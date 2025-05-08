// kernel/src/serial.rs
use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;
use core::fmt;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) }; // Standard COM1 port
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // Disable interrupts to prevent deadlock if an interrupt handler
    // also tries to print
    interrupts::without_interrupts(|| {
        SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
    });
}

/// Extra-simple print function for use in interrupt handlers
/// This avoids complex macro expansion and minimizes the
/// chance of deadlocks in interrupt context
pub fn _print_simple(s: &str) {
    // We're already in an interrupt context, so we don't need
    // to disable interrupts here, but we do need to be careful
    // about lock acquisition
    if let Some(_serial) = SERIAL1.try_lock() {
        // Only print if we can get the lock without blocking
        for byte in s.bytes() {
            unsafe {
                // Direct write using the 0xF8 port (COM1 data register)
                core::ptr::write_volatile(0x3F8 as *mut u8, byte);
            }
        }
    }
    // Silently fail if we can't get the lock (better than deadlock)
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {{
        $crate::serial::_print(format_args!($($arg)*));
    }};
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
} 