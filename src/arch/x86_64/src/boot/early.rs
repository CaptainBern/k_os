//! Early structures used for the transition from assembly to rust are kept
//! here.

use x86::{
    bits64::paging::{PDEntry, PDPTEntry, PML4Entry, PD, PDPT, PML4},
    dtables::DescriptorTablePointer,
};

use crate::desc::{
    Access, CodeSegmentBits, DataSegmentBits, DescriptorFlags, UserDescriptor, UserDescriptorType,
};

#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct EarlyGdt {
    pub null: UserDescriptor,
    pub code: UserDescriptor,
    pub data: UserDescriptor,
}

#[no_mangle]
#[link_section = ".phys.data"]
static BOOT_GDT: EarlyGdt = EarlyGdt {
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
        DescriptorFlags::G,
    ),
};

#[no_mangle]
#[link_section = ".phys.data"]
static mut BOOT_GDT_PTR: DescriptorTablePointer<EarlyGdt> = DescriptorTablePointer {
    limit: 23,
    base: &BOOT_GDT as *const _,
};

/// The level 4 page table, initialised in `head.S`. It contains two entries,
/// each pointing to [BOOT_PDPT]. The first entry is responsible for identity
/// mapping the first 4G of memory. The second entry, the last in the table,
/// maps the -2G virtual address space to the first 2G of the physical address
/// space.
#[no_mangle]
#[link_section = ".phys.bss"]
static BOOT_PML4: PML4 = [PML4Entry(0); 512];

/// The level 3 page table, initialised in `head.S`. It contains six entries.
/// The first four entries point to each of the level 2 page directories in
/// [BOOT_PDS] respectively. The last and second-to-last entries point to the
/// first two page directories in [BOOT_PDS].
#[no_mangle]
#[link_section = ".phys.bss"]
static BOOT_PDPT: PDPT = [PDPTEntry(0); 512];

/// The level 2 page directories, initialised in `head.S`. Each of the four
/// directories maps 1G of memory using 2M pages. The full 32bit address
/// space is mapped using these.
#[no_mangle]
#[link_section = ".phys.bss"]
static BOOT_PDS: [PD; 4] = [
    [PDEntry(0); 512],
    [PDEntry(0); 512],
    [PDEntry(0); 512],
    [PDEntry(0); 512],
];
