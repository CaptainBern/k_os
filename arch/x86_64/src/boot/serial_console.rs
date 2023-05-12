use spin::Mutex;
use uart_16550::SerialPort;

pub const DEFAULT_PORT: u16 = 0x3f8;

pub static SERIAL_PORT: Mutex<SerialPort> = Mutex::new(unsafe { SerialPort::new(DEFAULT_PORT) });

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut port = $crate::boot::serial_console::SERIAL_PORT.lock();
        port.write_fmt(format_args!($($arg)*)).unwrap();
    }}
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}

/// Initialise the serial console.
pub fn init() {
    SERIAL_PORT.lock().init();
}
