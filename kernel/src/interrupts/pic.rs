// kernel/src/interrupts/pic.rs
//! PIC (Programmable Interrupt Controller) management
//! This module provides a clean interface for initializing and configuring the 8259A PICs.

use pic8259::ChainedPics;
use spin::Mutex;
use crate::serial_println;

/// The base vector offset for PIC1 (master)
pub const PIC_1_OFFSET: u8 = 32;
/// The base vector offset for PIC2 (slave)
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Represents interrupt vectors corresponding to PIC IRQs
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,      // IRQ 0
    Keyboard = PIC_1_OFFSET + 1, // IRQ 1
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

/// Manages the 8259 Programmable Interrupt Controllers
pub struct PicController {
    pics: Mutex<ChainedPics>,
    initialized: bool,
}

impl PicController {
    /// Create a new PIC controller (not initialized yet)
    pub const fn new() -> Self {
        Self {
            pics: Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) }),
            initialized: false,
        }
    }

    /// Initialize the PICs and mask all interrupts
    /// This is the primary initialization function that should be called once during boot
    pub fn initialize(&mut self) {
        serial_println!("PIC: Starting initialization sequence");
        
        // Use direct port I/O for maximum control over the initialization sequence
        use x86_64::instructions::port::Port;
        
        // PIC1 (Master) ports
        let mut master_cmd: Port<u8> = Port::new(0x20);
        let mut master_data: Port<u8> = Port::new(0x21);
        
        // PIC2 (Slave) ports
        let mut slave_cmd: Port<u8> = Port::new(0xA0);
        let mut slave_data: Port<u8> = Port::new(0xA1);
        
        // Function to add a small delay between I/O operations
        // The x86_64 crate doesn't seem to have io_wait, so we implement it manually
        fn io_wait() {
            // Write to the POST port (0x80) which should take long enough for PIC operations
            let mut wait_port: Port<u8> = Port::new(0x80);
            unsafe { wait_port.write(0) };
        }
        
        // The initialization command sequence for each PIC
        unsafe {
            // Start by masking all interrupts
            master_data.write(0xFF);
            io_wait();
            slave_data.write(0xFF);
            io_wait();
            
            // ICW1: Start initialization sequence in cascade mode
            master_cmd.write(0x11);
            io_wait();
            slave_cmd.write(0x11);
            io_wait();
            
            // ICW2: Set vector offsets
            master_data.write(PIC_1_OFFSET);
            io_wait();
            slave_data.write(PIC_2_OFFSET);
            io_wait();
            
            // ICW3: Tell Master PIC that there is a slave PIC at IRQ2 (0000 0100)
            master_data.write(4);
            io_wait();
            
            // ICW3: Tell Slave PIC its cascade identity (0000 0010)
            slave_data.write(2);
            io_wait();
            
            // ICW4: Set 8086 mode (not Auto EOI)
            master_data.write(0x01);
            io_wait();
            slave_data.write(0x01);
            io_wait();
            
            // Make sure all interrupts are masked
            master_data.write(0xFF);
            io_wait();
            slave_data.write(0xFF);
            io_wait();
        }
        
        serial_println!("PIC: Direct initialization complete, all IRQs masked");
        self.initialized = true;
    }

    /// Configure IRQ masks to enable specific interrupts
    /// 
    /// # Safety
    /// This function should only be called after initialize() and before enabling
    /// CPU interrupts with STI. The PICs should be in a properly initialized state.
    pub unsafe fn configure_irqs(&self, primary_mask: u8, secondary_mask: u8) {
        if !self.initialized {
            serial_println!("WARNING: Attempting to configure PIC IRQs before initialization!");
            return;
        }
        
        serial_println!("PIC: Setting IRQ masks: Primary={:#08b}, Secondary={:#08b}", 
                       primary_mask, secondary_mask);
        self.pics.lock().write_masks(primary_mask, secondary_mask);
    }

    /// Enable only the timer interrupt (IRQ0)
    /// A convenience method for the common case of wanting just the timer
    /// 
    /// # Safety
    /// Same safety requirements as configure_irqs
    pub unsafe fn enable_timer_only(&self) {
        // 0xFE = 1111 1110 - Only IRQ0 (Timer) unmasked
        // 0xFF = 1111 1111 - All IRQs on secondary PIC masked
        self.configure_irqs(0xFE, 0xFF);
    }

    /// Notify end of interrupt for the specified IRQ
    /// 
    /// # Safety
    /// This should only be called from an interrupt handler for the specified IRQ
    pub unsafe fn end_of_interrupt(&self, interrupt_id: u8) {
        // Use direct port I/O for maximum control
        use x86_64::instructions::port::Port;
        
        // Convert interrupt vector back to IRQ number
        let irq = interrupt_id - PIC_1_OFFSET;
        
        // Send the EOI command to the appropriate PIC(s)
        if irq >= 8 {
            // For IRQs 8-15, must send EOI to both slave and master PICs
            let mut slave_cmd: Port<u8> = Port::new(0xA0);
            let mut master_cmd: Port<u8> = Port::new(0x20);
            
            // Send EOI to slave PIC
            slave_cmd.write(0x20);
            
            // Also send EOI to master PIC (for the cascade IRQ)
            master_cmd.write(0x20);
        } else {
            // For IRQs 0-7, only send EOI to master PIC
            let mut master_cmd: Port<u8> = Port::new(0x20);
            master_cmd.write(0x20);
        }
    }
}

// Create a global instance of the PIC controller
pub static mut PIC_CONTROLLER: PicController = PicController::new(); 