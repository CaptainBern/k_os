use core::mem;

use x86::{dtables::DescriptorTablePointer, fence::mfence};

use crate::{
    apic,
    desc::{
        Access, CodeSegmentBits, DataSegmentBits, DescriptorFlags, UserDescriptor,
        UserDescriptorType,
    },
};

const BOOTSTRAP_DATA_OFFSET: usize = 0x1000;
const BOOTSTRAP_GDT_PTR_OFFSET: usize = 0;
const BOOTSTRAP_GDT_OFFSET: usize = 10;
const BOOTSTRAP_KERNEL_TOP_OFFSET: usize = 42;
const BOOTSTRAP_STACK_OFFSET: usize = 50;
const BOOTSTRAP_PERCPU_OFFSET: usize = 58;

/// Bootstrap GDT, used to enable 32-bit protected mode.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct BootstrapGdt {
    null: UserDescriptor,
    code32: UserDescriptor,
    code64: UserDescriptor,
    data: UserDescriptor,
}

impl BootstrapGdt {
    pub const fn new() -> Self {
        Self {
            null: UserDescriptor::NULL,
            code32: UserDescriptor::new(
                0,
                0xfffff,
                UserDescriptorType::Code(CodeSegmentBits::EX_READ),
                Access::KERNEL_USR,
                DescriptorFlags::from_bits_truncate(DescriptorFlags::empty().bits()),
            ),
            code64: UserDescriptor::new(
                0,
                0xfffff,
                UserDescriptorType::Code(CodeSegmentBits::EX_READ),
                Access::KERNEL_USR,
                DescriptorFlags::L,
            ),
            data: UserDescriptor::new(
                0,
                0xfffff,
                UserDescriptorType::Data(DataSegmentBits::READ_WRITE),
                Access::KERNEL_USR,
                DescriptorFlags::from_bits_truncate(
                    DescriptorFlags::G.bits() | DescriptorFlags::DB.bits(),
                ),
            ),
        }
    }
}

/// Bootstrap used to boot APs.
///
/// This struct is tightly coupled with the code in [`boot/start16.S`].
#[derive(Debug)]
pub struct Bootstrap {
    virt: *mut u8,
    phys: u32,
}

impl Bootstrap {
    /// Setup a memory bootstrap region.
    ///
    /// The given region should be mapped below 1M in physical memory. `virt` is
    /// a virtual pointer to the bootstrap region. `phys` refers to the physical
    /// address at which the bootstrap is mapped. `kernel_top` refers to the
    /// physical address of the PML4 that should be used by the AP to switch into
    /// longmode.
    pub unsafe fn new(virt: *mut [u8; 0x2000], phys: u32, kernel_top: u64) -> Self {
        let bootstrap = Self {
            virt: virt as *mut u8,
            phys,
        };

        bootstrap.write_data(BOOTSTRAP_GDT_OFFSET, BootstrapGdt::new());
        bootstrap.write_data(BOOTSTRAP_KERNEL_TOP_OFFSET, kernel_top);
        bootstrap.write_data(
            BOOTSTRAP_GDT_PTR_OFFSET,
            DescriptorTablePointer {
                limit: (mem::size_of::<BootstrapGdt>() - 1) as u16,
                base: (phys as usize + BOOTSTRAP_DATA_OFFSET + BOOTSTRAP_GDT_OFFSET)
                    as *const BootstrapGdt,
            },
        );

        bootstrap
    }

    /// Internal function used to write a given value inside the data region.
    unsafe fn write_data<T>(&self, offset: usize, val: T) {
        assert!(mem::size_of::<T>() + offset <= 0x1000);
        (self
            .virt
            .byte_offset((BOOTSTRAP_DATA_OFFSET + offset) as isize) as *mut T)
            .write_unaligned(val);
    }

    /// Attempt to start the target AP with the given stack and percpu.
    ///
    /// # Safety
    /// Caller must make sure apic_id, stack, and percpu are valid and unique
    /// for each AP!
    pub unsafe fn try_start_ap(&self, apic_id: u32, stack: u64, percpu: u64) {
        self.write_data(BOOTSTRAP_STACK_OFFSET, stack);
        self.write_data(BOOTSTRAP_PERCPU_OFFSET, percpu);

        // Make sure the memory is synced between all CPUs.
        mfence();

        apic::local().ipi_init(apic_id);
        apic::local().ipi_startup(apic_id, (self.phys >> 12) as u8);
    }
}
