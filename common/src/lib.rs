#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]

// Common definitions for testing and error handling
pub fn test_common() -> u32 {
    42
}