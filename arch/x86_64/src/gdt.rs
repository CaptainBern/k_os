use core::mem;

use x86::dtables::{lgdt, DescriptorTablePointer};

use crate::{
    desc::{
        Access, CodeSegmentBits, DataSegmentBits, DescriptorFlags, SystemDescriptor,
        UserDescriptor, UserDescriptorType,
    },
    linker,
};

static mut GDT: KernelGdt = KernelGdt {
    null: UserDescriptor::NULL,
    kernel_code: UserDescriptor::new(
        0,
        0,
        UserDescriptorType::Code(CodeSegmentBits::EX_ONLY),
        Access::KERNEL_USR,
        DescriptorFlags::L,
    ),
    kernel_data: UserDescriptor::new(
        0,
        0,
        UserDescriptorType::Data(DataSegmentBits::READ_WRITE),
        Access::KERNEL_USR,
        DescriptorFlags::L,
    ),
    // Disable 32-bit ring3 code.
    user_code_32: UserDescriptor::new(
        0,
        0,
        UserDescriptorType::Code(CodeSegmentBits::empty()),
        Access::empty(),
        DescriptorFlags::empty(),
    ),
    user_code_64: UserDescriptor::new(
        0,
        0,
        UserDescriptorType::Code(CodeSegmentBits::EX_ONLY),
        Access::from_bits_truncate(Access::P.bits() | Access::S.bits() | Access::DPL_3.bits()),
        DescriptorFlags::L,
    ),
    user_data: UserDescriptor::new(
        0,
        0,
        UserDescriptorType::Data(DataSegmentBits::READ_WRITE),
        Access::from_bits_truncate(Access::P.bits() | Access::S.bits() | Access::DPL_3.bits()),
        DescriptorFlags::L,
    ),
    kernel_tss: [SystemDescriptor::NULL; linker::MAX_CPUS],
};

static mut GDT_PTR: DescriptorTablePointer<KernelGdt> = DescriptorTablePointer {
    limit: (mem::size_of::<KernelGdt>() - 1) as u16,
    base: unsafe { &GDT as *const _ },
};

/// The kernel GDT.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct KernelGdt {
    pub null: UserDescriptor,
    pub kernel_code: UserDescriptor,
    pub kernel_data: UserDescriptor,
    pub user_code_32: UserDescriptor,
    pub user_code_64: UserDescriptor,
    pub user_data: UserDescriptor,
    pub kernel_tss: [SystemDescriptor; linker::MAX_CPUS],
}

/// Load the kernel GDT.
pub unsafe fn init() {
    lgdt(&GDT_PTR);
}
