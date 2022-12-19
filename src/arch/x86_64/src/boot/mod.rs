use multiboot2::load;
use x86::int;

use crate::{include_asm, interrupts, pic, println};

pub mod early;
pub mod serial_console;

include_asm!("src/boot/header.S", "src/boot/start.S");

/// Continue the boot process.
///
/// At this point we're running in the higher half, but the pages are still
/// located in lower-level memory.
#[no_mangle]
extern "C" fn boot(multiboot_info_ptr: usize) -> ! {
    serial_console::init();
    // TODO: setup some form of error handling/messaging
    // TODO: IDT
    // TODO: GDT
    // TODO: CPUID value cache shit
    // TODO: per-cpu shit

    // TODO: Init thread_local_storage

    // disable PIC
    unsafe {
        pic::remap(0x20, 0x28);
        pic::disable();
    }

    // Initialise the early interrupt handlers asap.
    unsafe {
        println!("Initialising interrupts");
        interrupts::init_early_idt();
    }

    println!("Triggering breakpoint");
    unsafe {
        // int!(3);
    }
    println!("Resumed from breakpoint");

    // Try parsing the multiboot information structure, it should be identity
    // mapped.
    let multiboot_info = unsafe { load(multiboot_info_ptr).unwrap() };

    println!("{:?}", multiboot_info);

    // PLAN:
    // - fix paging
    //  * allocate per-cpu storage
    //  * fix-up permissions
    //  * TODO: figure out alignment + kernel memory map
    // - GS/FS BASE
    //  * store current cpu ID in %gs
    //  * store current TLS ptr in %fs
    // - GDT/IDT per CPU

    loop {}
}
