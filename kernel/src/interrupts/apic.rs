// kernel/src/interrupts/apic.rs
//! APIC (Advanced Programmable Interrupt Controller) support
//! Provides initialization for systems with APIC instead of legacy PIC

#[allow(dead_code)]

use core::ptr::{read_volatile, write_volatile};
use crate::serial_println;
use x86_64::registers::model_specific::Msr;
use x86_64::structures::idt::InterruptStackFrame;

// Local APIC registers (memory-mapped at the APIC base address)
const APIC_ID: u64 = 0x20;               // Local APIC ID Register
const APIC_VER: u64 = 0x30;              // Local APIC Version Register
const APIC_TPR: u64 = 0x80;              // Task Priority Register
const APIC_EOI: u64 = 0xB0;              // End of Interrupt Register
const APIC_LDR: u64 = 0xD0;              // Logical Destination Register
const APIC_DFR: u64 = 0xE0;              // Destination Format Register
const APIC_SPURIOUS: u64 = 0xF0;         // Spurious Interrupt Vector Register
const APIC_ICR_LOW: u64 = 0x300;         // Interrupt Command Register Low
const APIC_ICR_HIGH: u64 = 0x310;        // Interrupt Command Register High
const APIC_TIMER_LVT: u64 = 0x320;       // Timer Local Vector Table Entry
const APIC_THERMAL_LVT: u64 = 0x330;     // Thermal Local Vector Table Entry
const APIC_PERF_LVT: u64 = 0x340;        // Performance Local Vector Table Entry
const APIC_LINT0_LVT: u64 = 0x350;       // Local Interrupt 0 Vector Table Entry
const APIC_LINT1_LVT: u64 = 0x360;       // Local Interrupt 1 Vector Table Entry
const APIC_ERROR_LVT: u64 = 0x370;       // Error Vector Table Entry
const APIC_TIMER_ICR: u64 = 0x380;       // Timer Initial Count Register
const APIC_TIMER_CCR: u64 = 0x390;       // Timer Current Count Register
const APIC_TIMER_DCR: u64 = 0x3E0;       // Timer Divide Configuration Register

// IA32_APIC_BASE MSR (0x1B)
const IA32_APIC_BASE_MSR: u32 = 0x1B;
const IA32_APIC_BASE_MSR_BSP: u64 = 0x100;       // Is this the Bootstrap Processor?
const IA32_APIC_BASE_MSR_ENABLE: u64 = 0x800;    // APIC Global Enable bit
const IA32_APIC_BASE_MSR_ADDR_MASK: u64 = 0xFFFFFF000; // Base address mask

// APIC Timer modes
const APIC_TIMER_PERIODIC: u32 = 0x20000; // Timer mode: periodic

// Interrupt vector numbers
pub const APIC_TIMER_VECTOR: u8 = 0x20; // Vector 32 for the timer

// Track APIC state
static mut APIC_BASE_ADDR: u64 = 0;
static mut APIC_AVAILABLE: bool = false;

/// Detect if APIC is available
pub fn is_apic_available() -> bool {
    // TEMPORARY: Force to false to ensure we use the PIC path
    return false;
    
    // Original code:
    // unsafe { APIC_AVAILABLE }
}

/// Initialize the APIC system
pub fn init() -> bool {
    serial_println!("APIC: Checking for availability");
    
    // Use CPUID to check if APIC is available
    if !check_apic_available() {
        serial_println!("APIC: Not available on this CPU");
        unsafe { APIC_AVAILABLE = false; }
        return false;
    }
    
    // Get the APIC base address from MSR
    let apic_base = get_apic_base();
    unsafe { APIC_BASE_ADDR = apic_base; }
    
    serial_println!("APIC: Base physical address: {:#x}", apic_base);
    
    // SAFETY NOTE: This code assumes that the bootloader has identity-mapped
    // the APIC MMIO region. If it hasn't, we'll get page faults when trying to
    // access APIC registers, so let's add some safeguards.
    
    serial_println!("APIC: IMPORTANT: Skipping APIC setup to avoid page faults");
    serial_println!("APIC: Will fall back to legacy PIC for interrupts");
    
    // For now, mark APIC as not available to fall back to PIC
    unsafe { APIC_AVAILABLE = false; }
    return false;
    
    // Original code below - DISABLED TEMPORARILY
    /*
    // Enable the APIC
    enable_apic();
    
    // Set up the timer (disabled initially)
    configure_timer();
    
    unsafe { APIC_AVAILABLE = true; }
    serial_println!("APIC: Initialized successfully");
    true
    */
}

/// Check if APIC is available using CPUID
fn check_apic_available() -> bool {
    let result = unsafe {
        core::arch::x86_64::__cpuid(1)
    };
    
    // Bit 9 of EDX indicates APIC availability
    (result.edx & (1 << 9)) != 0
}

/// Get the APIC base address from MSR
fn get_apic_base() -> u64 {
    let msr = Msr::new(IA32_APIC_BASE_MSR);
    let value = unsafe { msr.read() };
    
    // Extract the base address (bits 12-51)
    value & IA32_APIC_BASE_MSR_ADDR_MASK
}

/// Enable the APIC
fn enable_apic() {
    let mut msr = Msr::new(IA32_APIC_BASE_MSR);
    let value = unsafe { msr.read() };
    
    // Set the enable bit
    unsafe { msr.write(value | IA32_APIC_BASE_MSR_ENABLE); }
    
    // Set up the Spurious Interrupt Vector Register
    // Enable APIC and set spurious vector to 0xFF
    write_apic_reg(APIC_SPURIOUS, 0x1FF);
}

/// Configure the APIC timer (initially disabled)
fn configure_timer() {
    // Set the timer's divide value register (divide by 16)
    write_apic_reg(APIC_TIMER_DCR, 0x3);
    
    // Set up the timer for one-shot mode, initially masked
    // Vector 32, masked (bit 16), one-shot mode
    write_apic_reg(APIC_TIMER_LVT, (APIC_TIMER_VECTOR as u32) | (1 << 16));
    
    // Initial count is 0 (disabled)
    write_apic_reg(APIC_TIMER_ICR, 0);
}

/// Enable the APIC timer with a specific initial count
pub fn enable_timer(initial_count: u32) {
    // Set initial count
    write_apic_reg(APIC_TIMER_ICR, initial_count);
    
    // Unmask the timer, periodic mode
    write_apic_reg(APIC_TIMER_LVT, (APIC_TIMER_VECTOR as u32) | APIC_TIMER_PERIODIC);
}

/// Send EOI (End Of Interrupt) to the APIC
pub fn send_eoi() {
    // Write any value to the EOI register
    write_apic_reg(APIC_EOI, 0);
}

/// Read from an APIC register
fn read_apic_reg(reg: u64) -> u32 {
    unsafe {
        let ptr = (APIC_BASE_ADDR + reg) as *const u32;
        read_volatile(ptr)
    }
}

/// Write to an APIC register
fn write_apic_reg(reg: u64, value: u32) {
    unsafe {
        let ptr = (APIC_BASE_ADDR + reg) as *mut u32;
        write_volatile(ptr, value);
    }
}

/// Timer interrupt handler for APIC timer
pub extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        // Write directly to COM1 port for debugging
        let com1_data_port: *mut u8 = 0x3F8 as *mut u8;
        *com1_data_port = b'A'; // 'A' for APIC timer
        
        // Send EOI to APIC
        send_eoi();
    }
} 