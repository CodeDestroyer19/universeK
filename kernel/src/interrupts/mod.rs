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

// Re-export PIC controller for convenience
pub use pic::PIC_CONTROLLER;

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
        // Get a direct reference in this limited initialization scope
        pic::PIC_CONTROLLER.initialize();
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
    // Check if we should use APIC or PIC
    if apic::is_apic_available() {
        serial_println!("DEBUG: interrupts::configure_for_operation - Using APIC (currently disabled)");
        // TODO: Configure APIC timer, IPIs etc.
        // apic::enable_timer(10_000_000); // Example: Enable timer
    } else {
        serial_println!("DEBUG: interrupts::configure_for_operation - Using Legacy PIC");
        // Unmask only the timer and keyboard IRQs for now
        // PIC IRQs: 0=Timer, 1=Keyboard
        unsafe {
            // pic::PIC_CONTROLLER.configure_irqs(0b11111100, 0b11111111); // Mask all except Timer (0) and Keyboard (1)
            serial_println!("DEBUG: interrupts::configure_for_operation - Enabling Timer IRQ only (IRQ 0)");
            pic::PIC_CONTROLLER.enable_timer_only(); // Enables only IRQ 0
        }
    }
    serial_println!("DEBUG: interrupts::configure_for_operation - End");
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
    // Ultra minimal handler - just print a dot to show timer interrupts
    unsafe {
        // Write directly to COM1 port (0x3F8) - a single '.' character
        let com1_data_port: *mut u8 = 0x3F8 as *mut u8;
        *com1_data_port = b'.';
        
        // Send EOI to PIC directly
        let master_cmd_port: *mut u8 = 0x20 as *mut u8;
        *master_cmd_port = 0x20; // EOI command
    }
}

// APIC Timer interrupt handler
extern "x86-interrupt" fn apic_timer_handler(_stack_frame: InterruptStackFrame) {
    // Do the same as the APIC timer handler but don't call it directly
    unsafe {
        // Write directly to COM1 port for debugging
        let com1_data_port: *mut u8 = 0x3F8 as *mut u8;
        *com1_data_port = b'A'; // 'A' for APIC timer
        
        // Send EOI to APIC
        apic::send_eoi();
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = 
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
                HandleControl::Ignore)
            );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60); // PS/2 keyboard data port

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => serial_print!("{}", character),
                DecodedKey::RawKey(key) => serial_print!("{:?}", key),
            }
        }
    }

    // Send EOI
    unsafe {
        pic::PIC_CONTROLLER.end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
} 