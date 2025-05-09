 //! Programmable Interval Timer (PIT) driver
//! Provides system timing functions via the Intel 8254 PIT

use spin::Mutex;
use lazy_static::lazy_static;
use x86_64::instructions::port::{Port, PortWriteOnly};
use x86_64::structures::idt::InterruptStackFrame;
use crate::errors::KernelError;
use crate::serial_println;

// PIT ports
const PIT_CHANNEL0: u16 = 0x40;  // Channel 0 data port
const PIT_CHANNEL1: u16 = 0x41;  // Channel 1 data port
const PIT_CHANNEL2: u16 = 0x42;  // Channel 2 data port
const PIT_COMMAND: u16 = 0x43;   // Mode/Command register

// PIT commands
const PIT_CMD_CHANNEL0: u8 = 0x00;  // Select channel 0
const PIT_CMD_LATCH: u8 = 0x00;     // Latch count value command
const PIT_CMD_ACCESS_BOTH: u8 = 0x30;  // Access mode: low byte then high byte
const PIT_CMD_MODE2: u8 = 0x04;     // Mode 2: rate generator
const PIT_CMD_MODE3: u8 = 0x06;     // Mode 3: square wave generator

// PIT parameters
const PIT_FREQUENCY: u32 = 1193182;  // Base frequency (Hz)
static mut CURRENT_FREQUENCY: u32 = 0;
static mut MS_PER_TICK: u32 = 0;

// Tick counter
static mut TICKS: u64 = 0;

// PIT driver structure
struct PitDriver {
    command_port: PortWriteOnly<u8>,
    data_port: Port<u8>,
    initialized: bool,
}

impl PitDriver {
    fn new() -> Self {
        Self {
            command_port: PortWriteOnly::new(PIT_COMMAND),
            data_port: Port::new(PIT_CHANNEL0),
            initialized: false,
        }
    }
    
    fn init(&mut self, frequency: u32) -> Result<(), KernelError> {
        if frequency == 0 || frequency > 1193180 {
            return Err(KernelError::InvalidParameter);
        }

        // Calculate divisor
        let divisor = 1193180 / frequency;
        
        // Send command byte: channel 0, lobyte/hibyte, mode 3 (square wave)
        unsafe {
            self.command_port.write(0x36);
            
            // Send divisor
            let low = (divisor & 0xFF) as u8;
            let high = ((divisor >> 8) & 0xFF) as u8;
            self.data_port.write(low);
            self.data_port.write(high);
        }
        
        // TODO: Implement proper interrupt registration
        // For now, we'll skip registering the handler since we're in safe mode
        
        Ok(())
    }
    
    fn set_frequency(&mut self, frequency: u32) -> Result<(), KernelError> {
        // Calculate divisor
        let divisor = PIT_FREQUENCY / frequency;
        
        if divisor > 65535 {
            return Err(KernelError::InvalidParameter);
        }
        
        // Save the configured frequency and ms per tick
        unsafe {
            CURRENT_FREQUENCY = frequency;
            MS_PER_TICK = 1000 / frequency;
        }
        
        // Send the command byte
        let command = PIT_CMD_CHANNEL0 | PIT_CMD_ACCESS_BOTH | PIT_CMD_MODE3;
        unsafe {
            // Send command
            self.command_port.write(command);
            
            // Send divisor (low byte, then high byte)
            self.data_port.write((divisor & 0xFF) as u8);
            self.data_port.write(((divisor >> 8) & 0xFF) as u8);
        }
        
        Ok(())
    }
}

lazy_static! {
    static ref PIT: Mutex<PitDriver> = Mutex::new(PitDriver::new());
}

/// PIT interrupt handler - called on timer tick
pub extern "x86-interrupt" fn pit_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    // TODO: Implement proper PIC handling
    // For now, we'll skip sending EOI since we're in safe mode
}

/// Initialize the PIT with the given frequency
pub fn init(frequency: u32) -> Result<(), KernelError> {
    serial_println!("Initializing PIT (Programmable Interval Timer) at {} Hz", frequency);
    
    // Initialize the PIT with the specified frequency
    PIT.lock().init(frequency)?;
    
    // Set current frequency and timing values
    unsafe {
        CURRENT_FREQUENCY = frequency;
        MS_PER_TICK = 1000 / frequency;
    }
    
    Ok(())
}

/// Set the PIT frequency
pub fn set_frequency(frequency: u32) -> Result<(), KernelError> {
    PIT.lock().set_frequency(frequency)
}

/// Get the current system tick count
pub fn get_ticks() -> u64 {
    unsafe { TICKS }
}

/// Get system uptime in milliseconds
pub fn get_uptime_ms() -> u64 {
    unsafe { TICKS * MS_PER_TICK as u64 }
}

/// Sleep for a number of milliseconds
pub fn sleep_ms(ms: u32) {
    let start_ticks = get_ticks();
    let ticks_to_wait = (ms as u64 * unsafe { CURRENT_FREQUENCY as u64 }) / 1000;
    
    while get_ticks() - start_ticks < ticks_to_wait {
        // Use the CPU's HLT instruction to pause until the next interrupt
        x86_64::instructions::hlt();
    }
}

/// Sleep for a number of seconds
pub fn sleep(seconds: u32) {
    sleep_ms(seconds * 1000);
}