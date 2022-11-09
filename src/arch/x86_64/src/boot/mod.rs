use x86::bits64::paging::{PML4, PML4Entry, PDPT, PDPTEntry, PD, PDEntry};

use crate::include_asm;

include_asm!(
    "src/boot/header.S",
    "src/boot/start.S"
);

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
    [PDEntry(0); 512]
];

/// Continue the boot process.
/// 
/// At this point we're running in the higher half, but the pages are still
/// located in lower-level memory.
#[no_mangle]
extern "C" fn boot(mb_info: usize) {
    // The full 32bit address space is identity mapped, which simplifies the
    // parsing of the ACPI tables. After we're done parsing them, we can move
    // on and setup proper kernel pages.


    loop {}
}
