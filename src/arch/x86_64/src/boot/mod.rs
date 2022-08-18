use core::arch::global_asm;

use x86::bits64::paging::{PML4, PDPT, PD, PML4Entry, PDPTEntry, PDEntry};

#[no_mangle]
static mut BOOT_PML4: PML4 = [PML4Entry(0); 512];

#[no_mangle]
static mut BOOT_PDPT: PDPT = [PDPTEntry(0); 512];

#[no_mangle]
static mut BOOT_PD: [PD; 4] = [
    [PDEntry(0); 512],
    [PDEntry(0); 512], 
    [PDEntry(0); 512], 
    [PDEntry(0); 512]
];

/**
 * A helper macro to include assembly files.
 */
macro_rules! include_asm {
    ($path:tt) => {
        global_asm!(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path)));
    };
}

include_asm!("src/boot/multiboot2.S");
include_asm!("src/boot/head.S");
include_asm!("src/boot/gdt.S");

#[no_mangle]
extern fn boot() {}