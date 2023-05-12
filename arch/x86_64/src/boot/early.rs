//! Early structures used for the transition from assembly to rust are kept
//! here. They are not `pub` on purpose, since they become unavailable once
//! we're on the proper kernel pages.
//!
//! `start.S` will initialise the boot page tables defined here. The boot tables
//! identity map the lower physical memory (4G), and map the first 2G of memory
//! to virtual address -2G (from the top of the virtual address space). The
//! reason for this is that the Rust code is compiled to run at the high
//! virtual address, while the assembly runs in 'physical' mode. Rust variables
//! annotated with `#[link_section = ".phys.*"]` are linked to their physical
//! address and can be used directly by the assembly stub (instead of having to
//! calculate their physical address at runtime) but functions, while directly
//! 'callable' from within the assembly stub, do rely on Rust APIs linked at a
//! high virtual address. So, in order to get to Rust, we need to setup 'early'
//! pagetables that identity-map the low physical memory (for the transition
//! from assembly to Rust), and mappings for the high virtual memory.
//!
//! Once we're in high virtual memory, we need to switch to a new GDT and top
//! page table (see `src/mm.rs`).

use x86::dtables::DescriptorTablePointer;

use crate::{
    desc::{
        Access, CodeSegmentBits, DataSegmentBits, DescriptorFlags, UserDescriptor,
        UserDescriptorType,
    },
    mm::paging::{PD, PDE, PDPT, PDPTE, PML4, PML4E},
};

/// Early boot stack. When changing the size, it should also be changed
/// in `start.S`.
#[no_mangle]
static mut BOOT_STACK: [u8; 0x4000] = [0; 0x4000];

#[allow(dead_code)] // Used by `start.S`
#[derive(Debug, Clone, Copy)]
#[repr(packed)]
struct EarlyGdt {
    null: UserDescriptor,
    code: UserDescriptor,
    data: UserDescriptor,
}

/// The early GDT. It contains the bare minimum to get us into longmode.
/// See `start.S`.
#[used]
#[no_mangle]
#[link_section = ".data.boot"]
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

/// Early GDT pointer. See `start.S`.
#[used]
#[no_mangle]
#[link_section = ".data.boot"]
static mut BOOT_GDT_PTR: DescriptorTablePointer<EarlyGdt> = DescriptorTablePointer {
    limit: 23,                   // (8 * 3) -1
    base: &BOOT_GDT as *const _, // 1:1 mapped so this is fine.
};

/// The level 4 page table, initialised in `head.S`. It contains two entries,
/// each pointing to [BOOT_PDPT]. The first entry is responsible for identity
/// mapping the first 4G of memory. The second entry, the last in the table,
/// maps the -2G virtual address space to the first 2G of the physical address
/// space.
#[used]
#[no_mangle]
#[link_section = ".bss.boot"]
static BOOT_PML4: PML4 = PML4::zero();

/// The level 3 page table, initialised in `head.S`. It contains six entries.
/// The first four entries point to each of the level 2 page directories in
/// [BOOT_PDS] respectively. The last and second-to-last entries point to the
/// first two page directories in [BOOT_PDS].
#[used]
#[no_mangle]
#[link_section = ".bss.boot"]
static BOOT_PDPT: PDPT = PDPT::zero();

/// The level 2 page directories, initialised in `head.S`. Each of the four
/// directories maps 1G of memory using 2M pages. The full 32bit address
/// space is mapped using these.
#[used]
#[no_mangle]
#[link_section = ".bss.boot"]
static BOOT_PDS: [PD; 4] = [PD::zero(), PD::zero(), PD::zero(), PD::zero()];
