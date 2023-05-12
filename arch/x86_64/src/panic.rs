use core::panic::PanicInfo;

use crate::println;

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[lang = "panic_impl"]
#[no_mangle]
pub extern "C" fn rust_begin_panic(panic_info: &PanicInfo) -> ! {
    // TODO:
    // - unwind the stack
    // - use a proper logger and not println.
    println!("{:?}", panic_info);
    loop {}
}
