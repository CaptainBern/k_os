// Tracking issues:
// - https://github.com/rust-lang/rust/issues/29594
// - https://github.com/rust-lang/rust/issues/90957
#![feature(lang_items, thread_local, naked_functions, asm_const, asm_sym)]
#![no_main]
#![no_std]

mod asm;
mod boot;
mod gdt;
mod idt;
mod interrupts;
mod linker;
mod mm;
mod panic;
mod pic;
mod traps;

#[no_mangle]
static mut KERNEL_STACK: [u8; 0x4000] = [0; 0x4000];

#[derive(Debug)]
pub struct KernelArguments {}

fn kmain(args: &KernelArguments) {}
