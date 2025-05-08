#![no_std] // Don't link the Rust standard library
#![no_main] // Disable all Rust-level entry points
#![feature(custom_test_frameworks)] // Enable custom test framework
#![test_runner(kernel::test_runner)] // Use our custom test runner
#![reexport_test_harness_main = "test_main"] // Rename the test main function

use core::panic::PanicInfo;
use kernel::println; // Use the println from our library crate
use bootloader::{BootInfo, entry_point};

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC: {}", info);
    loop {}
}

// Define the entry point using the bootloader crate's macro
// This automatically handles the correct function signature
entry_point!(kernel_main);

// The function called by the bootloader
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Kernel kernel_main entered...");
    kernel::init(boot_info); // Call the main initialization function

    println!("Kernel init returned? Should not happen!");
    kernel::hlt_loop(); // Use the halt loop from the kernel library
}
