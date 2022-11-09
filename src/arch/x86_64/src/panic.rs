use core::panic::PanicInfo;

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn rust_eh_personality() {
}

#[lang = "panic_impl"]
#[no_mangle]
pub extern fn rust_begin_panic(panic_info: &PanicInfo) -> ! {
    loop {}
}
