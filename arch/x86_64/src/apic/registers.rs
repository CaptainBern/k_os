//! x(2)APIC registers.
//!
//! Valid registers as defined in the Intel and AMD manuals (Vol. 3, 10.12.1.2
//! and Vol. 2, 16.3.2 respectively).
//!
//! An xAPIC register `x` can be converted to its x2APIC equivalent using:
//! `(x >> 4) + X2APIC_MSR_BASE`.

/// Base for x2APIC register access.
pub const X2APIC_MSR_BASE: u32 = 0x800;

/// Local APIC ID register.
pub const LOCAL_APIC_ID_REG: u32 = 0x20;

/// Local APIC version register.
pub const LOCAL_APIC_VERSION_REG: u32 = 0x30;

/// Task-priority register.
pub const TASK_PRIORITY_REG: u32 = 0x80;

/// Arbitration priority register. xAPIC only!
pub const ARBITRATION_PRIORITY_REG: u32 = 0x90;

/// Processor-priority register.
pub const PROCESSOR_PRIORITY_REG: u32 = 0xa0;

/// End-of-interrupt register.
pub const EOI_REG: u32 = 0xb0;

/// Remote-read register. xAPIC only!
pub const REMOTE_READ_REG: u32 = 0xc0;

/// Logical destination register.
pub const LOGICAL_DEST_REG: u32 = 0xd0;

/// Logical destination format register. xAPIC only!
pub const LOGICAL_DEST_FMT_REG: u32 = 0xe0;

/// Spurious interrupt vector register.
pub const SPURIOUS_INT_VECTOR_REG: u32 = 0xf0;

/// In-service register (bits 31:0).
pub const ISR0_REG: u32 = 0x100;

/// In-service register (bits 63:32).
pub const ISR1_REG: u32 = 0x110;

/// In-service register (bits 95:64).
pub const ISR2_REG: u32 = 0x120;

/// In-service register (bits 127:96).
pub const ISR3_REG: u32 = 0x130;

/// In-service register (bits 159:128).
pub const ISR4_REG: u32 = 0x140;

/// In-service register (bits 191:160).
pub const ISR5_REG: u32 = 0x150;

/// In-service register (bits 223:192).
pub const ISR6_REG: u32 = 0x160;

/// In-service register (bits 255:224).
pub const ISR7_REG: u32 = 0x170;

/// Triggermode register (bits 31:0).
pub const TMR0_REG: u32 = 0x180;

/// Triggermode register (bits 63:32).
pub const TMR1_REG: u32 = 0x190;

/// Triggermode register (bits 95:64).
pub const TMR2_REG: u32 = 0x1a0;

/// Triggermode register (bits 127:96).
pub const TMR3_REG: u32 = 0x1b0;

/// Triggermode register (bits 159:128).
pub const TMR4_REG: u32 = 0x1c0;

/// Triggermode register (bits 191:160).
pub const TMR5_REG: u32 = 0x1d0;

/// Triggermode register (bits 223:192).
pub const TMR6_REG: u32 = 0x1e0;

/// Triggermode register (bits 255:224).
pub const TMR7_REG: u32 = 0x1f0;

/// Interrupt request register (bits 31:0).
pub const IRR0_REG: u32 = 0x200;

/// Interrupt request  register (bits 63:32).
pub const IRR1_REG: u32 = 0x210;

/// Interrupt request  register (bits 95:64).
pub const IRR2_REG: u32 = 0x220;

/// Interrupt request  register (bits 127:96).
pub const IRR3_REG: u32 = 0x230;

/// Interrupt request  register (bits 159:128).
pub const IRR4_REG: u32 = 0x240;

/// Interrupt request  register (bits 191:160).
pub const IRR5_REG: u32 = 0x250;

/// Interrupt request  register (bits 223:192).
pub const IRR6_REG: u32 = 0x260;

/// Interrupt request  register (bits 255:224).
pub const IRR7_REG: u32 = 0x270;

/// Error status register.
pub const ERROR_STATUS_REG: u32 = 0x280;

/// LVT CMCI register.
pub const LVT_CMCI_REG: u32 = 0x2f0;

/// Interrupt command register (bits 31:0).
pub const ICR_LOW_REG: u32 = 0x300;

/// Interrupt command register (bits 63:32). xAPIC only!
pub const ICR_HIGH_REG: u32 = 0x310;

/// Local vector table Timer register.
pub const LVT_TIMER_REG: u32 = 0x320;

/// Local vector table Thermal Sensor register.
pub const LVT_THERM_SENSOR_REG: u32 = 0x330;

/// Local vector table Performance Monitoring Counter register.
pub const LVT_PERF_MON_COUNTER_REG: u32 = 0x340;

/// Local vector table LINT0 register.
pub const LVT_LINT0_REG: u32 = 0x350;

/// Local vector table LINT1 register.
pub const LVT_LINT1_REG: u32 = 0x360;

/// Local vector table Error register.
pub const LVT_ERROR_REG: u32 = 0x370;

/// Initial count register for the timer.
pub const TIMER_INIT_COUNT_REG: u32 = 0x380;

/// Current count register for the timer.
pub const TIMER_CURR_COUNT_REG: u32 = 0x390;

/// Divide configuration register for the timer.
pub const TIMER_DIVIDE_CONF_REG: u32 = 0x3e0;

/// Self IPI register. Only available for x2APIC.
pub const X2_SELF_IPI: u32 = 0x83f;

/// APIC version register.
///
/// The local APIC contains a hardwired version register. This register
/// can be used to identify the APIC version. The register also specifies
/// the number of entries in the Local Vector Table for a specific
/// implementation.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Version {
    bits: u32,
}

impl Version {
    /// Return the version of the local APIC.
    ///
    /// A version of `0x00` indicates a 82489DX discrete APIC. A version
    /// between `0x10` and `0x15` indicates an integrated APIC. Every other
    /// value is reserved.
    pub fn version(&self) -> u8 {
        (self.bits & 0xff) as u8
    }

    /// Returns the maximum number of LVT entrues minus one.
    pub fn max_lvt_entry(&self) -> u8 {
        ((self.bits >> 8) & 0xff) as u8
    }

    /// Indicates whether software can inhibit the broadcast of EOI message by
    /// setting bit 12 of the Spurious Interrupt Vector Register.
    pub fn eoi_broadcast_supression(&self) -> bool {
        ((self.bits >> 24) & 1) == 1
    }

    pub const unsafe fn from_bits_unchecked(bits: u32) -> Self {
        Version { bits }
    }
}

/// The local APIC records errors detected during interrupt handling in the Error Status
/// Register (ESR).
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ErrorStatus {
    bits: u32,
}

impl ErrorStatus {
    /// Set when the local APIC detects a checksum error for a message that it sent
    /// on the APIC bus. Used only on P6 family and Pentium processors. Reserved on AMD.
    pub const fn send_checksum_error(&self) -> bool {
        (self.bits & 1) == 1
    }

    /// Set when the local APIC detects a checksum error for a message that it received
    /// on the APIC bus. Used only on P6 family and Pentium processors. Reserved on AMD.
    pub const fn receive_checksum_error(&self) -> bool {
        ((self.bits >> 1) & 1) == 1
    }

    /// Set when the local APIC detects that a message it sent was not accepted by any APIC
    /// on the APIC bus. For Intel, used only on P6 family and Pentium processors.
    pub const fn send_accept_error(&self) -> bool {
        ((self.bits >> 2) & 1) == 1
    }

    /// Set when the local APIC detects that a message it received was not accepted by any
    /// APIC on the APIC bus, including itself. For Intel, used only on P6 family and Pentium
    /// processors.
    pub const fn receive_accept_error(&self) -> bool {
        ((self.bits >> 3) & 1) == 1
    }

    /// Set when the local APIC detects an attempt to send an IPI with the lowest-priority
    /// delivery mode and the local APIC does not support the sending of such IPIs.
    /// This bit is used on some Intel Core and Xeon processors. Reserved on AMD.
    pub const fn redirectable_ipi(&self) -> bool {
        ((self.bits >> 4) & 1) == 1
    }

    /// Set when the local APIC detects an illegal vector (one in the range 0 to 15) in the
    /// message that it is sending. This occurs as the result of a write to the ICR (in both
    /// xAPIC and x2APIC modes) or to the SELF IPI register (for x2APIC mode) with an illegal
    /// vector.
    pub const fn send_illegal_vector(&self) -> bool {
        ((self.bits >> 5) & 1) == 1
    }

    /// Set when the local APIC detects an illegal vector (one in the range 0 to 15) in an
    /// interrupt message it receives or in an interrupt generated locally from the local
    /// vector table or via a SELF IPI. Such interrupts are not delivered to the processor;
    /// the local APIC will never set an IRR bit in the range 0 to 15.
    pub const fn receive_illegal_vector(&self) -> bool {
        ((self.bits >> 6) & 1) == 1
    }

    /// Set when the local APIC is in xAPIC mpode and software attempts to access a register
    /// that is reserved in the processor's local-APIC register address-space. Used only on
    /// Intel Core, Intel Atom, Pentium 4, Intel Xeon, P6 family processors, and AMD.
    ///
    /// In x2APIC mode, software accesses the APIC registers using the RDMSR and WRMSR instructions.
    /// Use of one of these instructions to access a reserved register cause a general-protection
    /// exception. They do not set the 'Illegal Register Access' bit.
    pub const fn illegal_register_access(&self) -> bool {
        ((self.bits >> 7) & 1) == 1
    }

    pub const unsafe fn from_bits_unchecked(bits: u32) -> Self {
        ErrorStatus { bits }
    }

    pub const fn bits(&self) -> u32 {
        self.bits
    }
}

/// Interrupt delivery mode.
///
/// Specifies the type of interrupt to be sent to the processor. On AMD this
/// is known as 'message type'. Some delivery modes will only operate as
/// intended when used in conjunction with a specific trigger mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum DeliveryMode {
    /// Deliver the interrupt specified in the vector field.
    Fixed = 0b000,

    /// The IPI delivers an interrupt to the local APIC executing at the lowest
    /// priority of all of the local APICs that match the destination logical
    /// ID specified in the destination field of the ICR.
    ///
    /// This mode can only be used for ICR. In addition, this functionality
    /// is model specific, and should generally be avoided by BIOS or operating
    /// system software.
    LowestPriority = 0b001,

    /// Deliver an SMI interrupt to the processor core through the processor's
    /// local SMI signal path. When using this mode, the vector field should be
    /// set to 0.
    SMI = 0b010,

    /// AMD provides a Remote-read functionality. Since intel does not, we opt
    /// to not implement it.
    _Reserved = 0b011,

    /// Delivers an NMI interrupt to the processor. The vector field is
    /// ignored.
    NMI = 0b100,

    /// Delivers an INIT request to the processor core, which causes the
    /// processor to perform an INIT. When using this mode, the vector field
    /// should be set to 0. In the INIT state, the target APIC is responsive
    /// only to the STARTUP IPI. All other interrupts (including SMI and NMI)
    /// are held pending until the STARTUP IPI has been accepted.
    ///
    /// This mode is not supported for the LVT thermal sensor, LVT performance
    /// monitor, and LVT CMCI registers.
    INIT = 0b101,

    /// The IPI delivers a start-up request (SIPI) to the target local APIC(s)
    /// specified in the destination field, causing the CPU core to start
    /// processing the platform firmware boot-strap routine whose address is
    /// specified by the vector field.
    ///
    /// This mode can only be used for ICR!
    StartUp = 0b110,

    /// Causes the processor to respond to the interrupt as if the interrupt
    /// originated in an externally connected (8259A-compatible) interrupt
    /// controller. The APIC architecture only supports one ExtINT source in a
    /// system, usually contained in the compatibility bridge. Only one
    /// processor in the system should have an LVT entry configured to use the
    /// ExtINT delivery mode.
    ///
    /// This mode is not supported for the LVT thermal sensor, LVT performance
    /// monitor, LVT CMCI registers, and ICR.
    ExtINT = 0b111,
}

/// Indicates the interrupt delivery status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum DeliveryStatus {
    /// There is currently no activity for this interrupt source, or the
    /// previous interrupt from this source was delivered to the processor core
    /// and accepted.
    Idle = 0,

    /// Indicates that an interrupt from this source has been delivered to the
    /// processor core but has not yet been accepted.
    SendPending = 1,
}

/// The trigger mode for the local LINT0 and LINT1 pins.
///
/// This flag is only used when the delivery mode is Fixed. When the delivery
/// mode is SMI, NMI or INIT, the trigger mode is always edge sensitive. When
/// the delivery mode is ExtINT, the trigger mode is always level sensitive.
/// The timer and error interrupts are always treated as edge sensitive.
///
/// Software should always set the trigger mode in the LVT LINT1 register to
/// edge. Level-sensitive interrupts are not supported for LINT1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum TriggerMode {
    Edge = 0,
    Level = 1,
}

/// The valid modes for the Timer register.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum TimerMode {
    /// Oneshot mode. The countdown value is stored in an initial-count register.
    OneShot = 0b00,

    /// Periodic mode. The interval value us stored in an initial-count register.
    Periodic = 0b01,

    /// TSC deadline mode. Program the target value in iA32_TSC_DEADLINE MSR.
    TscDeadline = 0b10,

    /// Reserved for future use.
    _Reserved = 0b11,
}

/// Valid divisors for the divide configuration register (See [DivideConfiguration]).
///
/// The divisor specifies the the value of the CPU core clock divisor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Divisor {
    By2 = 0b0_0_00,
    By4 = 0b0_0_001,
    By8 = 0b0_0_10,
    By16 = 0b0_0_11,
    By32 = 0b1_0_00,
    By64 = 0b1_0_01,
    By128 = 0b1_0_10,
    By1 = 0b1_0_11,
}

/// Divisor configuration register.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct DivideConfiguration {
    bits: u32,
}

impl DivideConfiguration {
    pub const fn new(divisor: Divisor) -> Self {
        DivideConfiguration {
            bits: divisor as u32,
        }
    }

    pub const fn divisor(&self) -> Divisor {
        match self.bits & 0b1011 {
            0b0_0_00 => Divisor::By2,
            0b0_0_001 => Divisor::By4,
            0b0_0_10 => Divisor::By8,
            0b0_0_11 => Divisor::By16,
            0b1_0_00 => Divisor::By32,
            0b1_0_01 => Divisor::By64,
            0b1_0_10 => Divisor::By128,
            0b1_0_11 => Divisor::By1,
            _ => unreachable!(),
        }
    }

    pub fn set_divisor(&mut self, divisor: Divisor) {
        self.bits &= !(0b1111);
        self.bits |= divisor as u32
    }

    pub const unsafe fn from_bits_unchecked(bits: u32) -> Self {
        DivideConfiguration { bits }
    }

    pub const fn bits(&self) -> u32 {
        self.bits
    }
}

macro_rules! define_lvt {
    (
        @impl
        $name:ident
        delivery_mode
    ) => {
        impl $name {
            pub const fn delivery_mode(&self) -> DeliveryMode {
                match (self.bits >> 8) & 0b111 {
                    0b000 => DeliveryMode::Fixed,
                    0b001 => DeliveryMode::LowestPriority,
                    0b010 => DeliveryMode::SMI,
                    0b011 => DeliveryMode::_Reserved,
                    0b100 => DeliveryMode::NMI,
                    0b101 => DeliveryMode::INIT,
                    0b110 => DeliveryMode::StartUp,
                    0b111 => DeliveryMode::ExtINT,
                    _ => unreachable!(),
                }
            }

            pub fn set_delivery_mode(&mut self, mode: DeliveryMode) {
                self.bits &= !(0b111 << 8);
                self.bits |= (mode as u32) << 8;
            }
        }
    };

    (
        @impl
        $name:ident
        remote_irr
    ) => {
        impl $name {
            pub const fn remote_irr(&self) -> bool {
                ((self.bits >> 14) & 1) == 1
            }
        }
    };

    (
        @impl
        $name:ident
        timer_mode
    ) => {
        impl $name {
            pub const fn timer_mode(&self) -> TimerMode {
                match (self.bits >> 17) & 0b11 {
                    0b00 => TimerMode::OneShot,
                    0b01 => TimerMode::Periodic,
                    0b10 => TimerMode::TscDeadline,
                    _ => unreachable!(),
                }
            }

            pub fn set_timer_mode(&mut self, mode: TimerMode) {
                self.bits &= !(0b11 << 17);
                self.bits |= (mode as u32) << 17;
            }
        }
    };

    (
        @impl
        $name:ident
        trigger_mode
    ) => {
        impl $name {
            pub const fn trigger_mode(&self) -> TriggerMode {
                match (self.bits >> 15) & 1 {
                    0 => TriggerMode::Edge,
                    1 => TriggerMode::Level,
                    _ => unreachable!(),
                }
            }

            pub fn set_trigger_mode(&mut self, mode: TriggerMode) {
                self.bits &= !(1 << 15);
                self.bits |= (mode as u32) << 15;
            }
        }
    };

    (
        @struct
        $(#[$($meta:tt)*])*
        $name:ident $({
            $($fields:tt)*
        })?;
    ) => {
        $(
            #[$($meta)*]
        )*
        #[derive(Debug, Clone, Copy)]
        #[repr(transparent)]
        pub struct $name {
            bits: u32,
        }

        impl $name {
            pub const fn new(vector: u8, masked: bool) -> Self {
                Self {
                    bits: ((masked as u32) << 12) | vector as u32
                }
            }

            pub const fn vector(&self) -> u8 {
                (self.bits & 0xff) as u8
            }

            pub fn set_vector(&mut self, vector: u8) {
                self.bits &= !(0xff);
                self.bits |= vector as u32;
            }

            pub const fn delivery_status(&self) -> DeliveryStatus {
                match (self.bits >> 12) & 1 {
                    0 => DeliveryStatus::Idle,
                    1 => DeliveryStatus::SendPending,
                    _ => unreachable!()
                }
            }

            pub const fn masked(&self) -> bool {
                ((self.bits >> 16) & 1) == 1
            }

            pub fn set_masked(&mut self, masked: bool) {
                self.bits &= !(1 << 16);
                self.bits |= (masked as u32) << 16;
            }

            pub const unsafe fn from_bits_unchecked(bits: u32) -> Self {
                Self { bits }
            }

            pub const fn bits(&self) -> u32 {
                self.bits
            }
        }

        $(
            $(
                define_lvt! {
                    @impl
                    $name
                    $fields
                }
            )*
        )?
    };

    (
        $(
            $(#[$($meta:tt)*])*
            $name:ident $({
                $($fields:tt)*
            })?;
        )+
    ) => {
        $(
            define_lvt! {
                @struct
                $(
                    #[$($meta)*]
                )*
                $name $({
                    $($fields)*
                })?;
            }
        )+
    };
}

define_lvt! {
    #[doc = "
    Local APIC timer LVT register.

    This struct represents the local APIC timer LVT entry. Using the
    time requires programming the current-count, initial-count, and
    divide-configuration registers as well.

    This register determines the vector number that is delivered to the
    processor when the interrupt is triggered. The mask flag can be used
    to mask the interrupt.
    "]
    Timer {
        timer_mode
    };

    CMCI {
        delivery_mode
    };

    LINT0 {
        delivery_mode
        remote_irr
        trigger_mode
    };

    LINT1 {
        delivery_mode
        remote_irr
        trigger_mode
    };

    #[doc = "
    APIC error LVT register.

    Errors that are detected while handling interrupts cause an APIC error
    interrupt to be generated under control of the mask bit of the LVT
    error register.

    The error information is stored in the Error Status Register (see [ErrorStatus]).
    "]
    Error {};

    PerfMonCounter {
        delivery_mode
    };

    ThermalSensor {
        delivery_mode
    };
}

/// IPI destination mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum DestinationMode {
    /// Selects physical destination mode.
    Physical = 0,

    /// Selects logical destination mode.
    ///
    /// In this mode, the destination may be one or more local APICs with a
    /// common destination logical ID.
    Logical = 1,
}

/// For the INIT level de-assert delivery mode, [Level::Deassert] must be
/// used; for all other delivery mode, [Level::Assert] must be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Level {
    Deassert = 0,
    Assert = 1,
}

/// Indicates whether a shorthand notation is used to specify the destination
/// of the interrupt, and if so, which shorthand is used. Destination
/// shorthands are used in place of the 8-bit destination field, and can be
/// sent by software using a single write to the low doubleword of the ICR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum DestinationShorthand {
    /// No shorthand. The destination is specified in the destination field.
    NoShorthand = 0b00,

    /// The issuing APIC is the one and only destination of the IPI.
    Myself = 0b01,

    /// The IPI is sent to all processors in the system, including the
    /// processor sending the IPI.
    AllIncludingSelf = 0b10,

    /// The IPI is sent to all processors in the system with the exception of
    /// the processor sending the IPI.
    AllExludingSelf = 0b11,
}

/// The lower double-word of the ICR register.
///
/// A write to this word causes the IPI to be sent.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct IcrLow {
    bits: u32,
}

impl IcrLow {
    pub const fn new(
        vector: u8,
        delivery: DeliveryMode,
        destination: DestinationMode,
        level: Level,
        trigger: TriggerMode,
        shorthand: DestinationShorthand,
    ) -> Self {
        let mut bits = vector as u32;
        bits |= (delivery as u32) << 8;
        bits |= (destination as u32) << 11;
        bits |= (level as u32) << 14;
        bits |= (trigger as u32) << 15;
        bits |= (shorthand as u32) << 18;

        Self { bits }
    }

    pub const fn vector(&self) -> u8 {
        (self.bits & 0xff) as u8
    }

    pub fn set_vector(&mut self, vector: u8) {
        self.bits &= !(0xff);
        self.bits |= vector as u32;
    }

    pub const fn delivery_mode(&self) -> DeliveryMode {
        match (self.bits >> 8) & 0b111 {
            0b000 => DeliveryMode::Fixed,
            0b001 => DeliveryMode::LowestPriority,
            0b010 => DeliveryMode::SMI,
            0b011 => DeliveryMode::_Reserved,
            0b100 => DeliveryMode::NMI,
            0b101 => DeliveryMode::INIT,
            0b110 => DeliveryMode::StartUp,
            0b111 => DeliveryMode::ExtINT,
            _ => unreachable!(),
        }
    }

    pub fn set_delivery_mode(&mut self, mode: DeliveryMode) {
        self.bits &= !(0b111 << 8);
        self.bits |= (mode as u32) << 8;
    }

    pub const fn destination_mode(&self) -> DestinationMode {
        match (self.bits >> 11) & 1 {
            0 => DestinationMode::Physical,
            1 => DestinationMode::Logical,
            _ => unreachable!(),
        }
    }

    pub fn set_destination_mode(&mut self, mode: DestinationMode) {
        self.bits &= !(1 << 11);
        self.bits |= (mode as u32) << 11;
    }

    pub const fn delivery_status(&self) -> DeliveryStatus {
        match (self.bits >> 12) & 1 {
            0 => DeliveryStatus::Idle,
            1 => DeliveryStatus::SendPending,
            _ => unreachable!(),
        }
    }

    pub const fn level(&self) -> Level {
        match (self.bits >> 14) & 1 {
            0 => Level::Deassert,
            1 => Level::Assert,
            _ => unreachable!(),
        }
    }

    pub fn set_level(&mut self, level: Level) {
        self.bits &= !(1 << 14);
        self.bits |= (level as u32) << 14;
    }

    pub const fn trigger_mode(&self) -> TriggerMode {
        match (self.bits >> 15) & 1 {
            0 => TriggerMode::Edge,
            1 => TriggerMode::Level,
            _ => unreachable!(),
        }
    }

    pub fn set_trigger_mode(&mut self, mode: TriggerMode) {
        self.bits &= !(1 << 15);
        self.bits |= (mode as u32) << 15;
    }

    pub const fn destination_shorthand(&self) -> DestinationShorthand {
        match (self.bits >> 18) & 0b11 {
            0b00 => DestinationShorthand::NoShorthand,
            0b01 => DestinationShorthand::Myself,
            0b10 => DestinationShorthand::AllIncludingSelf,
            0b11 => DestinationShorthand::AllExludingSelf,
            _ => unreachable!(),
        }
    }

    pub fn set_destination_shorthand(&mut self, shorthand: DestinationShorthand) {
        self.bits &= !(0b11 << 18);
        self.bits |= (shorthand as u32) << 18;
    }

    pub const unsafe fn from_bits_unchecked(bits: u32) -> Self {
        Self { bits }
    }

    pub const fn bits(&self) -> u32 {
        self.bits
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct IcrHigh {
    bits: u32,
}

impl IcrHigh {
    pub const fn new() -> Self {
        IcrHigh { bits: 0 }
    }

    pub const fn new_xapic_destination(dest: u8) -> Self {
        Self {
            bits: (dest as u32) << 24,
        }
    }

    pub const fn new_x2apic_destination(dest: u32) -> Self {
        Self { bits: dest }
    }

    pub const fn xapic_destination(&self) -> u8 {
        (self.bits >> 24) as u8
    }

    pub fn set_xapic_destination(&mut self, destination: u8) {
        self.bits = (destination as u32) << 24;
    }

    pub const fn x2apic_destination(&self) -> u32 {
        self.bits
    }

    pub fn set_x2apic_destination(&mut self, destination: u32) {
        self.bits = destination;
    }

    pub const unsafe fn from_bits_unchecked(bits: u32) -> Self {
        Self { bits }
    }

    pub const fn bits(&self) -> u32 {
        self.bits
    }
}

/// Interrupt command register.
///
/// The primary facility for issuing Inter-Processor Interrupts (IPIs) is by
/// the Interrupt Command Register. In xAPIC mode, this register is accessed
/// using two 32-bit registers (low and high). Writing to the low register
/// will cause the IPI to be sent. In x2APIC mode, the ICR is addressed as a
/// single RDMSR/WRMSR.
///
/// In xAPIC mode, AMD specifies bits 17:16 as a read-only Remote Read Status
/// field. This field indicates the remote-read status from another local APIC.
/// Both Intel and AMD mark these bits as reserved when running in x2APIC mode,
/// hence why we aren't providing an implementation for that field.
#[derive(Debug, Clone, Copy)]
pub struct Icr {
    pub high: IcrHigh,
    pub low: IcrLow,
}

impl Icr {
    pub const fn new(low: IcrLow, high: IcrHigh) -> Self {
        Self { low, high }
    }

    pub const unsafe fn from_bits64_unchecked(bits: u64) -> Self {
        Self::from_bits_unchecked((bits >> 32) as u32, (bits & 0xffffffff) as u32)
    }

    pub const unsafe fn from_bits_unchecked(high: u32, low: u32) -> Self {
        Icr {
            high: IcrHigh::from_bits_unchecked(high),
            low: IcrLow::from_bits_unchecked(low),
        }
    }

    pub const fn bits(&self) -> u64 {
        (self.high.bits() as u64) << 32 | self.low.bits() as u64
    }
}
