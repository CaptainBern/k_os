// Tracking issues:
// - https://github.com/rust-lang/rust/issues/29594
// - https://github.com/rust-lang/rust/issues/90957
#![feature(lang_items)]
#![feature(thread_local)]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(alloc_layout_extra)]
#![feature(const_maybe_uninit_zeroed)]
#![feature(offset_of)]
#![feature(pointer_byte_offsets)]
#![feature(const_pointer_byte_offsets)]
#![no_main]
#![no_std]

use core::sync::atomic::{AtomicU32, Ordering};

use libacpi::AcpiTables;
use spin::Once;
use x86::irq;

extern crate acpi as libacpi;

pub mod apic;
pub mod asm;
pub mod boot;
pub mod cpu;
pub mod desc;
pub mod gdt;
pub mod idt;
pub mod ioapic;
pub mod linker;
pub mod mm;
pub mod panic;
pub mod percpu;
pub mod pic;
pub mod smp;
pub mod stacks;
pub mod thread;

/// The maximum number of memory region descriptors. Changing this value will change
/// the kernel memory footprint.
pub const MAX_MEM_REGIONS: usize = 32;

/// Used to retieve the BSP APIC ID.
static BSP_APIC_ID: AtomicU32 = AtomicU32::new(u32::MAX);

/// The number of CPUs in the system.
static NUM_CPUS: AtomicU32 = AtomicU32::new(1);

/// Global ACPI tables.
static ACPI_TABLES: Once<AcpiTables> = Once::new();

pub fn init(stack: u64, percpu_offset: u64) {
    unsafe {
        percpu::init(percpu_offset);
    }

    // Setup GDT
    unsafe {
        gdt::init(
            stack + linker::STACK_SIZE as u64,
            stacks::nmi_stack_top(),
            stacks::df_stack_top(),
            stacks::mc_stack_top(),
        );

        gdt::load();
    }

    // Enable the local APIC for this node.
    apic::local().enable();

    // If we are the BSP, we are responsible for setting up the IDT stacks.
    if apic::local().is_bsp() {
        idt::set_ist(2, gdt::NMI_IST_INDEX);
        idt::set_ist(8, gdt::DF_IST_INDEX);
        idt::set_ist(18, gdt::MC_IST_INDEX);

        BSP_APIC_ID.store(apic::local().id(), Ordering::Relaxed);
    }

    // TODO: smep/smap, syscalls, fpu, ...

    // Everything done, we're ready to handle interrupts.
    unsafe {
        irq::enable();
    }

    NUM_CPUS.fetch_add(1, Ordering::Relaxed);
}

/// Start the current node.
pub fn start() -> ! {
    println!("Running!");
    loop {}
}
