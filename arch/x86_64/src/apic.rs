use core::ptr;

use spin::Once;
use x86::msr::{rdmsr, wrmsr, IA32_APIC_BASE};

use crate::{
    apic::registers::{DivideConfiguration, Timer, LVT_TIMER_REG, TIMER_DIVIDE_CONF_REG},
    cpu::cpuid,
    linker,
};

use self::registers::{
    DeliveryMode, DeliveryStatus, DestinationMode, DestinationShorthand, Divisor, Error,
    ErrorStatus, Icr, IcrHigh, IcrLow, Level, TimerMode, TriggerMode, EOI_REG, ERROR_STATUS_REG,
    ICR_HIGH_REG, ICR_LOW_REG, LOCAL_APIC_ID_REG, LVT_ERROR_REG, TIMER_INIT_COUNT_REG,
    X2APIC_MSR_BASE,
};

pub mod registers;

/// Local APIC.
///
/// This enum provides a way to program the local APIC, be it
/// the xAPIC or x2APIC.
///
/// TODO: not really happy with how reads and writes are implemented at the moment.
#[derive(Debug)]
pub enum LocalApic {
    /// Backed by xAPIC.
    ///
    /// We're using an MMIO to interface with the APIC registers.
    XApic(*mut [u32; 0x400]),

    /// Backed by x2APIC.
    ///
    /// In this mode, we interface with the APIC using RDMSR and WRMSR.
    X2Apic,
}

/// Safety: Local APIC access is CPU relative.
unsafe impl Sync for LocalApic {}

/// Safety: Local APIC access is CPU relative.
unsafe impl Send for LocalApic {}

impl LocalApic {
    /// Enable the local APIC.
    pub fn enable(&self) {
        unsafe {
            let mut base = rdmsr(IA32_APIC_BASE);

            // set the global 'EN' (or 'AE' on AMD) bit.
            base |= 1 << 11;
            wrmsr(IA32_APIC_BASE, base);

            // From AMD64 Architecture Programmer's Manual Vol. 2, 16.9:
            // 'the local APIC is placed into x2APIC mode by setting bit 10 in the
            // Local APIC base register. Before entering x2APIC mode, the local APIC
            // must first be enabled. System software can then place the local APIC
            // into x2APIC mode by executing a WRMSR with both AE=1 and EXTD=1.'
            if matches!(self, LocalApic::X2Apic) {
                base |= 1 << 10;
                wrmsr(IA32_APIC_BASE, base);
            }
        }
    }

    /// Disable the local APIC.
    pub fn disable(&self) {
        unsafe {
            let mut base = rdmsr(IA32_APIC_BASE);
            base &= !(1 << 11 & 1 << 10);
            wrmsr(IA32_APIC_BASE, base);
        }
    }

    /// Returns true is the current CPU is the Boot Strap Processor.
    pub fn is_bsp(&self) -> bool {
        unsafe { (rdmsr(IA32_APIC_BASE) & (1 << 8)) == (1 << 8) }
    }

    /// Returns the APIC ID of the current CPU.
    pub fn id(&self) -> u32 {
        let mut raw = unsafe { self.read(LOCAL_APIC_ID_REG) as u32 };

        if matches!(self, LocalApic::XApic(_)) {
            raw >>= 24;
        }

        raw
    }

    /// Setup the APIC Error LVT entry.
    pub fn setup_error(&self, vector: u8) {
        unsafe {
            self.write(LVT_ERROR_REG, Error::new(vector, false).bits() as u64);
            self.write(ERROR_STATUS_REG, 0);
        }
    }

    /// Setup the APIC timer.
    pub fn setup_timer(&self, vector: u8, masked: bool, mode: TimerMode, divisor: Divisor) {
        assert!(matches!(mode, TimerMode::OneShot) || matches!(mode, TimerMode::Periodic));

        let mut timer = Timer::new(vector, masked);
        timer.set_timer_mode(mode);

        let divisor = DivideConfiguration::new(divisor);

        unsafe {
            self.write(LVT_TIMER_REG, timer.bits() as u64);
            self.write(TIMER_DIVIDE_CONF_REG, divisor.bits() as u64);
        }
    }

    /// Start the APIC timer.
    ///
    /// To avoid race conditions, this function should not be called before
    /// [`setup_timer`](LocalApic::start_timer) has been called.
    pub fn start_timer(&self, init: u32) {
        unsafe {
            self.write(TIMER_INIT_COUNT_REG, init as u64);
        }
    }

    /// Stop the APIC timer.
    pub fn stop_timer(&self) {
        unsafe {
            self.write(TIMER_INIT_COUNT_REG, 0x0);
        }
    }

    /// Send an INIT IPI to the target APIC.
    ///
    /// This will reset the target into the INIT state and await a STARTUP IPI.
    pub fn ipi_init(&self, apic_id: u32) {
        let low = IcrLow::new(
            0,
            DeliveryMode::INIT,
            DestinationMode::Physical,
            Level::Assert,
            TriggerMode::Edge,
            DestinationShorthand::NoShorthand,
        );

        let high = if matches!(self, LocalApic::XApic(_)) {
            IcrHigh::new_xapic_destination(apic_id as u8)
        } else {
            IcrHigh::new_x2apic_destination(apic_id)
        };

        self.ipi(Icr::new(low, high));
    }

    /// Send a synchronization message to all local APICs in the system to set
    /// their arbitration IDs to the values of their APIC IDs.
    pub fn ipi_init_deassert(&self) {
        let low = IcrLow::new(
            0,
            DeliveryMode::INIT,
            DestinationMode::Physical, // destination mode doesn't matter.
            Level::Deassert,
            TriggerMode::Level,
            DestinationShorthand::AllIncludingSelf,
        );

        let high = IcrHigh::new();

        self.ipi(Icr::new(low, high));
    }

    /// Send a STARTUP IPI to the target APIC.
    ///
    /// After receiving the STARTUP, the target will begin executing the bootstrap
    /// routine located at `bootstrap * 4096`.
    pub fn ipi_startup(&self, apic_id: u32, bootstrap: u8) {
        let low = IcrLow::new(
            bootstrap,
            DeliveryMode::StartUp,
            DestinationMode::Physical,
            Level::Assert,
            TriggerMode::Edge,
            DestinationShorthand::NoShorthand,
        );
        let high = if matches!(self, LocalApic::XApic(_)) {
            IcrHigh::new_xapic_destination(apic_id as u8)
        } else {
            IcrHigh::new_x2apic_destination(apic_id)
        };

        self.ipi(Icr::new(low, high));
    }

    /// Send an IPI using the supplied ICR.
    ///
    /// Caller must make sure ICR is properly formatted.
    pub fn ipi(&self, icr: Icr) {
        match self {
            LocalApic::XApic(_) => {
                self.await_icr_send();
                self.write_icr(icr);
                self.await_icr_send();
            }
            LocalApic::X2Apic => self.write_icr(icr),
        }
    }

    /// Issue an end-of-interrupt.
    pub fn eoi(&self) {
        unsafe {
            self.write(EOI_REG, 0x0);
        }
    }

    /// Read the error status register.
    pub fn esr(&self) -> ErrorStatus {
        unsafe {
            self.write(ERROR_STATUS_REG, 0);
            ErrorStatus::from_bits_unchecked(self.read(ERROR_STATUS_REG) as u32)
        }
    }

    /// Block while the ICR is in the 'Send Pending' status.
    fn await_icr_send(&self) {
        while self.read_icr().low.delivery_status() == DeliveryStatus::SendPending {
            core::hint::spin_loop();
        }
    }

    /// Perform a raw write to the ICR register.
    fn write_icr(&self, icr: Icr) {
        match self {
            LocalApic::XApic(_) => unsafe {
                self.write(ICR_HIGH_REG, icr.high.bits() as u64);
                self.write(ICR_LOW_REG, icr.low.bits() as u64);
            },
            LocalApic::X2Apic => unsafe {
                self.write(ICR_LOW_REG, icr.bits());
            },
        }
    }

    /// Read the ICR register.
    pub fn read_icr(&self) -> Icr {
        match self {
            LocalApic::XApic(_) => unsafe {
                let low = self.read(ICR_LOW_REG) as u32;
                let high = self.read(ICR_HIGH_REG) as u32;
                Icr::from_bits_unchecked(low, high)
            },
            LocalApic::X2Apic => unsafe { Icr::from_bits64_unchecked(self.read(ICR_LOW_REG)) },
        }
    }

    /// Write to the given register, automatically translating it to x2APIC if
    /// necessary.
    unsafe fn write(&self, reg: u32, val: u64) {
        match self {
            LocalApic::XApic(_) => self.unchecked_write(reg, val),
            LocalApic::X2Apic => self.unchecked_write(X2APIC_MSR_BASE + (reg >> 4), val),
        }
    }

    /// Read the given register, automatically translating it to x2APIC if
    /// necessary.
    unsafe fn read(&self, reg: u32) -> u64 {
        match self {
            LocalApic::XApic(_) => self.unchecked_read(reg),
            LocalApic::X2Apic => self.unchecked_read(X2APIC_MSR_BASE + (reg >> 4)),
        }
    }

    /// Perform an unchecked write to the given register.
    pub unsafe fn unchecked_write(&self, reg: u32, val: u64) {
        match self {
            LocalApic::XApic(mmio) => {
                let ptr = (*mmio as *const u32).add(reg as usize) as *mut u32;
                ptr::write_volatile(ptr, val as u32)
            }
            LocalApic::X2Apic => {
                wrmsr(reg, val);
            }
        }
    }

    /// Perform an unchecked read from the given register.
    pub unsafe fn unchecked_read(&self, reg: u32) -> u64 {
        match self {
            LocalApic::XApic(mmio) => {
                let ptr = (*mmio as *const u32).add(reg as usize) as *const u32;
                ptr::read_volatile(ptr) as u64
            }
            LocalApic::X2Apic => rdmsr(reg),
        }
    }
}

/// Retrieve a reference to the local APIC.
///
/// This function may only be called *after* the APIC MMIO has been mapped (see [mm::map_apic]).
pub fn local() -> &'static LocalApic {
    static LOCAL: Once<LocalApic> = Once::new();

    LOCAL.call_once(|| {
        if cpuid().features.has_x2apic() {
            LocalApic::X2Apic
        } else {
            LocalApic::XApic(linker::LOCAL_APIC_ADDRESS as *mut [u32; 0x400])
        }
    })
}
