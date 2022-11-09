#![feature(lang_items)]
#![no_main]
#![no_std]

mod asm;
mod boot;
mod mm;
mod panic;

#[no_mangle]
static mut KERNEL_STACK: [u8; 0x4000] = [0; 0x4000];