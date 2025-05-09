// kernel/src/interrupts/mod.rs
//! Interrupt handling and management
//! This module contains the Interrupt Descriptor Table setup and interrupt handlers.

pub mod pic; // Make the PIC controller module available
pub mod apic; // Add APIC support

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::{println, serial_print, serial_println, hlt_loop};
use crate::gdt;
use lazy_static::lazy_static;
use pic::InterruptIndex;
use core::sync::atomic::{AtomicUsize, Ordering};

// Re-export PIC controller for convenience
pub use pic::PIC_CONTROLLER;

// A counter to track the number of timer interrupts
static TIMER_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Safely write a single character to the serial port (COM1)
/// This function checks if the transmitter is ready before writing
unsafe fn safe_serial_write(c: u8) {
    let com1_lsr_port: *mut u8 = 0x3FD as *mut u8;
    if (*com1_lsr_port & 0x20) != 0 {  // Check if transmitter holding register is empty
        let com1_data_port: *mut u8 = 0x3F8 as *mut u8;
        *com1_data_port = c;
    }
}

lazy_static! {
    // Create an IDT instance. It must be 'static because the CPU needs
    // to access it indefinitely after we load it.
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // Set up the Breakpoint handler (#BP, vector 3)
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        
        // Set up the Double Fault handler (#DF, vector 8) with a dedicated stack
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        
        // Add Page Fault handler (#PF, vector 14)
        idt.page_fault.set_handler_fn(page_fault_handler);
        
        // Add PIC interrupt handlers
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::Mouse.as_usize()].set_handler_fn(crate::drivers::ps2_mouse::mouse_interrupt_handler);
        
        // Add APIC timer handler (uses the same vector as PIC timer)
        // This allows us to handle timer interrupts whether they come from PIC or APIC
        if apic::is_apic_available() {
            idt[apic::APIC_TIMER_VECTOR as usize].set_handler_fn(apic_timer_handler);
        }
        
        idt
    };
}

/// Initializes the interrupt system
/// 
/// This function:
/// 1. Loads the Interrupt Descriptor Table (IDT)
/// 2. Initializes the 8259 Programmable Interrupt Controllers (PICs)
/// 3. Masks all interrupts (they will be selectively enabled later)
/// 4. If available, initializes the APIC
pub fn init() {
    serial_println!("Interrupt: Loading IDT");
    IDT.load();
    serial_println!("Interrupt: IDT loaded successfully");
    
    // Initialize and mask the PICs
    unsafe {
        serial_println!("Interrupt: Initializing PICs");
        
        // First disable CPU interrupts during initialization
        x86_64::instructions::interrupts::disable();
        serial_println!("Interrupt: CPU interrupts disabled during initialization");
        
        // Initialize PICs
        pic::PIC_CONTROLLER.initialize();
        serial_println!("Interrupt: PICs initialized, all IRQs masked");
        
        // Make doubly sure all interrupts are masked
        pic::PIC_CONTROLLER.configure_irqs(0xFF, 0xFF);
        serial_println!("Interrupt: Double-checked IRQ masking");
    }
    
    // Try to initialize APIC if available
    serial_println!("Interrupt: Checking for APIC");
    if apic::init() {
        serial_println!("Interrupt: APIC initialized successfully");
    } else {
        serial_println!("Interrupt: APIC not available or initialization failed; falling back to legacy PIC");
    }
    
    serial_println!("Interrupt: System initialized (CPU interrupts still disabled)");
}

/// Configure interrupts for normal operation
/// (e.g., unmask timer, keyboard)
pub fn configure_for_operation() {
    serial_println!("DEBUG: interrupts::configure_for_operation - Start");
    // For now, we'll just mask all interrupts to ensure stability
    // We can re-enable them once we have a stable system
    unsafe {
        serial_println!("DEBUG: interrupts::configure_for_operation - Masking ALL interrupts for stability");
        pic::PIC_CONTROLLER.configure_irqs(0b11111111, 0b11111111); // All masked
    }
    serial_println!("DEBUG: interrupts::configure_for_operation - End (all IRQs masked)");
}

// --- Exception Handlers ---

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> ! 
{
    serial_print!("!!! DOUBLE FAULT !!!\nStack frame: {:#?}\n", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    serial_print!("EXCEPTION: PAGE FAULT\n");
    serial_print!("Accessed Address: {:?}\n", Cr2::read());
    serial_print!("Error Code: {:?}\n", error_code);
    serial_print!("Stack frame: {:#?}\n", stack_frame);
    hlt_loop(); 
}

// --- Hardware Interrupt Handlers ---

// PIC Timer interrupt handler
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Ultra-minimal handler as a last resort
    unsafe {
        // 1. Directly write EOI to master PIC command port
        let master_cmd_port: *mut u8 = 0x20 as *mut u8;
        *master_cmd_port = 0x20; // EOI command directly to PIC
        
        // 2. No other operations that could potentially fail
        // Note: We're deliberately avoiding using the more complex EOI methods
        //       since they might be the source of the issue
    }
}

// APIC Timer interrupt handler
extern "x86-interrupt" fn apic_timer_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        // Write 'A' to show APIC timer interrupts
        safe_serial_write(b'A');
        
        // Send EOI to APIC
        apic::send_eoi();
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        // Write a direct indicator that the keyboard handler is starting
        safe_serial_write(b'K');
        safe_serial_write(b'1');
        
        // Read scancode directly, with minimal operations
        let keyboard_port = 0x60 as *mut u8;
        let scancode: u8 = *keyboard_port;
        
        // Simple debugging - write scancode to COM1 as hex digits
        let hex_chars = b"0123456789ABCDEF";
        safe_serial_write(hex_chars[(scancode >> 4) as usize]);
        safe_serial_write(hex_chars[(scancode & 0xF) as usize]);
        
        // Minimal processing - just call direct handler
        crate::drivers::ps2_keyboard::direct_handle_scancode(scancode);
        
        // Write a direct indicator before EOI
        safe_serial_write(b'E');
        
        // Send EOI directly to the PIC
        let pic_cmd = 0x20 as *mut u8;
        *pic_cmd = 0x20; // EOI command
        
        // Final indicator that handler completed
        safe_serial_write(b'2');
    }
} 