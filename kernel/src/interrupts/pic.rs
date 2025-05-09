// kernel/src/interrupts/pic.rs
//! PIC (Programmable Interrupt Controller) management
//! This module provides a clean interface for initializing and configuring the 8259A PICs.

use spin::Mutex;
use crate::serial_println;
use x86_64::instructions::port::Port;

/// The PICs are configured to use these offsets for their IRQs
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Command and data ports for the PICs
const PIC_1_COMMAND: u16 = 0x20;
const PIC_1_DATA: u16 = 0x21;
const PIC_2_COMMAND: u16 = 0xA0;
const PIC_2_DATA: u16 = 0xA1;

/// PIC initialization command words
const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;

/// PIC end of interrupt command
const PIC_EOI: u8 = 0x20;

/// Represents interrupt vectors corresponding to PIC IRQs
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,      // IRQ 0
    Keyboard = PIC_1_OFFSET + 1, // IRQ 1
    // IRQs 2-7 on the master PIC
    
    // IRQs 8-15 on the slave PIC, add 8 to the offset
    Mouse = PIC_1_OFFSET + 12, // IRQ 12
    // Add other PIC interrupts as needed
}

impl InterruptIndex {
    /// Convert to the raw u8 interrupt vector
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Convert to usize for IDT indexing
    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

/// A structure representing the PIC controller
pub struct PicController {
    primary_offset: u8,
    secondary_offset: u8,
}

impl PicController {
    /// Creates a new PIC controller with the given offsets
    pub const fn new(primary_offset: u8, secondary_offset: u8) -> Self {
        Self {
            primary_offset,
            secondary_offset,
        }
    }

    /// Initializes the PICs with the configured offsets
    pub fn initialize(&mut self) {
        serial_println!("Initializing PICs with offsets: primary={}, secondary={}", 
            self.primary_offset, self.secondary_offset);

        // Save current masks
        let primary_mask: u8 = unsafe { Port::new(PIC_1_DATA).read() };
        let secondary_mask: u8 = unsafe { Port::new(PIC_2_DATA).read() };
        serial_println!("DEBUG: Saved PIC masks - Primary: {:08b}, Secondary: {:08b}", 
            primary_mask, secondary_mask);

        // Start initialization sequence
        unsafe {
            // ICW1: Start initialization sequence
            Port::new(PIC_1_COMMAND).write(ICW1_INIT | ICW1_ICW4);
            Port::new(PIC_2_COMMAND).write(ICW1_INIT | ICW1_ICW4);

            // ICW2: Set vector offsets
            Port::new(PIC_1_DATA).write(self.primary_offset);
            Port::new(PIC_2_DATA).write(self.secondary_offset);

            // ICW3: Tell PICs how they're cascaded
            Port::new(PIC_1_DATA).write(4u8); // Secondary PIC at IRQ2
            Port::new(PIC_2_DATA).write(2u8); // Cascade identity

            // ICW4: Set 8086 mode
            Port::new(PIC_1_DATA).write(ICW4_8086);
            Port::new(PIC_2_DATA).write(ICW4_8086);

            // Mask all interrupts initially
            Port::new(PIC_1_DATA).write(0xFFu8);
            Port::new(PIC_2_DATA).write(0xFFu8);
        }

        serial_println!("PIC initialization complete");
    }

    /// Configures which IRQs are enabled/disabled
    pub fn configure_irqs(&mut self, primary_mask: u8, secondary_mask: u8) {
        serial_println!("Configuring IRQs - Primary mask: {:08b}, Secondary mask: {:08b}", 
            primary_mask, secondary_mask);
            
        unsafe {
            // Ensure we're not enabling any interrupts that aren't properly set up
            let safe_primary_mask = primary_mask & 0xFCu8; // Only allow IRQ0 (timer) and IRQ1 (keyboard)
            let safe_secondary_mask = secondary_mask & 0xEFu8; // Only allow IRQ12 (mouse)
            
            // Write the masks
            Port::new(PIC_1_DATA).write(safe_primary_mask);
            Port::new(PIC_2_DATA).write(safe_secondary_mask);
            
            // Verify the masks were written correctly
            let verify_primary: u8 = unsafe { Port::new(PIC_1_DATA).read() };
            let verify_secondary: u8 = unsafe { Port::new(PIC_2_DATA).read() };
            
            serial_println!("IRQs configured - Primary: {:08b}, Secondary: {:08b}", 
                safe_primary_mask, safe_secondary_mask);
            serial_println!("Verified masks - Primary: {:08b}, Secondary: {:08b}", 
                verify_primary, verify_secondary);
        }
    }

    /// Sends an end of interrupt signal for the given IRQ
    pub fn notify_end_of_interrupt(&mut self, irq: u8) {
        if irq >= 8 {
            unsafe {
                Port::new(PIC_2_COMMAND).write(PIC_EOI);
            }
        }
        unsafe {
            Port::new(PIC_1_COMMAND).write(PIC_EOI);
        }
    }
}

/// Global PIC controller instance
pub static PIC_CONTROLLER: Mutex<PicController> = Mutex::new(PicController::new(PIC_1_OFFSET, PIC_2_OFFSET)); 