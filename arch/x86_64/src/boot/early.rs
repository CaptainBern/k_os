//! Early structures used for the transition from assembly to rust.

use core::mem;

use x86::dtables::DescriptorTablePointer;

use crate::{
    desc::{
        Access, CodeSegmentBits, DataSegmentBits, DescriptorFlags, UserDescriptor,
        UserDescriptorType,
    },
    linker,
    mm::paging::{PD, PDPT, PML4},
};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(packed)]
struct BootGdt {
    null: UserDescriptor,
    code: UserDescriptor,
    data: UserDescriptor,
}

#[repr(C, align(16))]
pub struct Stack(pub [u8; linker::STACK_SIZE]);

/// Early boot stack. When changing the size, it should also be changed
/// in `start.S`.
#[used]
#[no_mangle]
pub static mut BOOT_STACK: Stack = Stack([0u8; linker::STACK_SIZE]);

/// The level 4 page table, initialised in `head.S`. It contains two entries,
/// each pointing to [BOOT_PDPT]. The first entry is responsible for identity
/// mapping the first 4G of memory. The second entry, the last in the table,
/// maps the -2G virtual address space to the first 2G of the physical address
/// space.
#[used]
#[no_mangle]
static BOOT_PML4: PML4 = PML4::zero();

/// The level 3 page table, initialised in `head.S`. It contains six entries.
/// The first four entries point to each of the level 2 page directories in
/// [BOOT_PDS] respectively. The last and second-to-last entries point to the
/// first two page directories in [BOOT_PDS].
#[used]
#[no_mangle]
static BOOT_PDPT: PDPT = PDPT::zero();

/// The level 2 page directories, initialised in `head.S`. Each of the four
/// directories maps 1G of memory using 2M pages. The full 32bit address
/// space is mapped using these.
#[used]
#[no_mangle]
static BOOT_PDS: [PD; 4] = [PD::zero(), PD::zero(), PD::zero(), PD::zero()];

/// The early GDT. It contains the bare minimum to get us into longmode.
/// See `start.S`.
#[used]
#[no_mangle]
static BOOT_GDT: BootGdt = BootGdt {
    null: UserDescriptor::NULL,
    code: UserDescriptor::new(
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
        DescriptorFlags::from_bits_truncate(DescriptorFlags::G.bits() | DescriptorFlags::DB.bits()),
    ),
};

/// Early GDT pointer. See `start.S`.
#[used]
#[no_mangle]
static mut BOOT_GDT_PTR: DescriptorTablePointer<BootGdt> = DescriptorTablePointer {
    limit: (mem::size_of::<BootGdt>() - 1) as u16,
    base: &BOOT_GDT as *const _, // 1:1 mapped so this is fine.
};
