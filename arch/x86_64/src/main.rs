// Tracking issues:
// - https://github.com/rust-lang/rust/issues/29594
// - https://github.com/rust-lang/rust/issues/90957
#![feature(lang_items)]
#![feature(thread_local)]
#![feature(naked_functions)]
#![feature(asm_const)]
#![no_main]
#![no_std]

use acpi::TableKind;
use heapless::Vec;
use spin::Once;

pub mod apic;
pub mod asm;
pub mod boot;
pub mod cpu;
pub mod desc;
pub mod gdt;
pub mod interrupts;
pub mod linker;
pub mod mm;
pub mod mp;
pub mod panic;
pub mod percpu;
pub mod pic;

/// The maximum number of memory region descriptors. The list of memory
/// descriptors is statically allocated, so changing this value will change
/// the kernel memory footprint.
pub const MAX_MEM_REGIONS: usize = 32;

/// Boot information, initialised by `boot.rs`.
pub static BOOT_INFO: Once<BootInfo> = Once::new();

/// The boot information, as provided by the bootloader.
/// TODO: modules
#[derive(Debug)]
pub struct BootInfo {
    /// The memory map as provided by the BIOS.
    pub mem_descriptors: Vec<mm::desc::MemoryDescriptor, MAX_MEM_REGIONS>,

    /// The ACPI tables (either normal or extended).
    pub acpi_tables: acpi::AcpiTables<'static>,
}

/// Start the current CPU.
fn startup_cpu() -> ! {
    loop {}
}

/// Startup the system.
///
/// We're still on our boot stack and initial pages. The boot information also still
/// refers to physical memory.
pub fn startup_system(boot_info: BootInfo) -> ! {
    // Store bootinfo.
    BOOT_INFO.call_once(|| boot_info);

    println!("BSP: {}", cpu::is_bsc());

    // TODO:
    // - setup proper kernel memory + remap ACPI shit yo!
    // - gdt/tss
    // - bootcode for APs
    // - start other cores

    unsafe {
        let madt = BOOT_INFO
            .get_unchecked()
            .acpi_tables
            .iter()
            .find_map(|table| {
                if let TableKind::Madt(madt) = table {
                    Some(madt)
                } else {
                    None
                }
            })
            .expect("Failed to find MADT");
        let lapic_address = madt.local_interrupt_controller_address;

        println!("LAPIC address: {:#018x}", lapic_address);

        pic::remap(0x20, 0x28);
        pic::disable();
    }
    interrupts::init_early_idt();

    unsafe {
        cpu::init_cpu();
        gdt::init();
        mm::init_early(BOOT_INFO.get_unchecked());
    }

    unsafe {
        println!("Retrieving current per-cpu");
        let current = percpu::current();
        println!("current: {:?}", current);
    }

    cpu::start_cpu()
}
