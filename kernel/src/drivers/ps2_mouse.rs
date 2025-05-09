//! PS/2 mouse driver
//! Handles mouse input via the PS/2 controller

use core::sync::atomic::{AtomicBool, Ordering};
use alloc::collections::VecDeque;
use spin::Mutex;
use lazy_static::lazy_static;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};
use x86_64::structures::idt::InterruptStackFrame;
use crate::errors::{KernelError, DeviceError};
use crate::serial_println;

// PS/2 controller ports
const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;
const PS2_COMMAND_PORT: u16 = 0x64;

// PS/2 controller commands
const PS2_READ_CONFIG: u8 = 0x20;
const PS2_WRITE_CONFIG: u8 = 0x60;
const PS2_DISABLE_SECOND_PORT: u8 = 0xA7;
const PS2_ENABLE_SECOND_PORT: u8 = 0xA8;
const PS2_TEST_SECOND_PORT: u8 = 0xA9;
const PS2_WRITE_SECOND_PORT: u8 = 0xD4;

// PS/2 device commands
const PS2_RESET_DEVICE: u8 = 0xFF;
const PS2_ENABLE_REPORTING: u8 = 0xF4;
const PS2_SET_DEFAULTS: u8 = 0xF6;
const PS2_SET_SAMPLE_RATE: u8 = 0xF3;
const PS2_GET_DEVICE_ID: u8 = 0xF2;
const PS2_SET_RESOLUTION: u8 = 0xE8;

// PS/2 device responses
const PS2_ACK: u8 = 0xFA;
const PS2_RESEND: u8 = 0xFE;

// Keyboard status flags
const PS2_OUTPUT_FULL: u8 = 1 << 0;
const PS2_INPUT_FULL: u8 = 1 << 1;

// Mouse flags
const MOUSE_LEFT_BUTTON: u8 = 0x01;
const MOUSE_RIGHT_BUTTON: u8 = 0x02;
const MOUSE_MIDDLE_BUTTON: u8 = 0x04;
const MOUSE_X_SIGN: u8 = 0x10;
const MOUSE_Y_SIGN: u8 = 0x20;
const MOUSE_X_OVERFLOW: u8 = 0x40;
const MOUSE_Y_OVERFLOW: u8 = 0x80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseState {
    pub x: i16,
    pub y: i16,
    pub buttons: u8,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            buttons: 0,
        }
    }
    
    pub fn left_button(&self) -> bool {
        (self.buttons & MOUSE_LEFT_BUTTON) != 0
    }
    
    pub fn right_button(&self) -> bool {
        (self.buttons & MOUSE_RIGHT_BUTTON) != 0
    }
    
    pub fn middle_button(&self) -> bool {
        (self.buttons & MOUSE_MIDDLE_BUTTON) != 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MouseButtons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
}

impl MouseButtons {
    pub fn new() -> Self {
        Self {
            left: false,
            right: false,
            middle: false,
        }
    }
    
    pub fn from_bits(bits: u8) -> Self {
        Self {
            left: (bits & MOUSE_LEFT_BUTTON) != 0,
            right: (bits & MOUSE_RIGHT_BUTTON) != 0,
            middle: (bits & MOUSE_MIDDLE_BUTTON) != 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    pub x: i16,
    pub y: i16,
    pub dx: i8,
    pub dy: i8,
    pub buttons: MouseButtons,
}

// Global mouse state
lazy_static! {
    static ref MOUSE: Mutex<Mouse> = Mutex::new(Mouse::new());
}

static MOUSE_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub struct Mouse {
    data_port: Port<u8>,
    status_port: PortReadOnly<u8>,
    command_port: PortWriteOnly<u8>,
    event_queue: VecDeque<MouseEvent>,
    state: MouseState,
    packet: [u8; 3],
    packet_index: usize,
}

impl Mouse {
    fn new() -> Self {
        Self {
            data_port: Port::new(PS2_DATA_PORT),
            status_port: PortReadOnly::new(PS2_STATUS_PORT),
            command_port: PortWriteOnly::new(PS2_COMMAND_PORT),
            event_queue: VecDeque::with_capacity(16),
            state: MouseState::new(),
            packet: [0; 3],
            packet_index: 0,
        }
    }

    fn init(&mut self) -> Result<(), KernelError> {
        serial_println!("Initializing PS/2 mouse");
        
        unsafe {
            // Disable the PS/2 port during initialization
            self.command_port.write(PS2_DISABLE_SECOND_PORT);
            
            // Get current controller configuration
            self.command_port.write(PS2_READ_CONFIG);
            let config = self.data_port.read();
            
            // Enable IRQ12 (mouse interrupt)
            let new_config = config | 0x02;
            self.command_port.write(PS2_WRITE_CONFIG);
            self.data_port.write(new_config);
            
            // Test mouse port
            self.command_port.write(PS2_TEST_SECOND_PORT);
            if self.wait_for_data() != 0x00 {
                serial_println!("PS/2 mouse port test failed");
                return Err(KernelError::DeviceError(DeviceError::InitFailed));
            }
            
            // Enable mouse port
            self.command_port.write(PS2_ENABLE_SECOND_PORT);
            
            // Reset the mouse
            self.send_command(PS2_RESET_DEVICE)?;
            if self.wait_for_data() != PS2_ACK {
                serial_println!("Mouse reset command failed");
                return Err(KernelError::DeviceError(DeviceError::InitFailed));
            }
            
            // Wait for self-test completion
            let response = self.wait_for_data();
            if response != 0xAA {
                serial_println!("Mouse self-test failed: {:02x}", response);
                return Err(KernelError::DeviceError(DeviceError::InitFailed));
            }
            
            // Get device ID
            self.send_command(PS2_GET_DEVICE_ID)?;
            if self.wait_for_data() != PS2_ACK {
                serial_println!("Mouse get device ID command failed");
                return Err(KernelError::DeviceError(DeviceError::InitFailed));
            }
            
            let device_id = self.wait_for_data();
            serial_println!("Mouse device ID: {:02x}", device_id);
            
            // Set sample rate to 100 samples/sec
            self.send_command(PS2_SET_SAMPLE_RATE)?;
            if self.wait_for_data() != PS2_ACK {
                serial_println!("Mouse set sample rate command failed");
                return Err(KernelError::DeviceError(DeviceError::InitFailed));
            }
            
            self.data_port.write(100);
            if self.wait_for_data() != PS2_ACK {
                serial_println!("Mouse set sample rate value failed");
                return Err(KernelError::DeviceError(DeviceError::InitFailed));
            }
            
            // Enable data reporting
            self.send_command(PS2_ENABLE_REPORTING)?;
            if self.wait_for_data() != PS2_ACK {
                serial_println!("Mouse enable reporting command failed");
                return Err(KernelError::DeviceError(DeviceError::InitFailed));
            }
        }
        
        serial_println!("PS/2 mouse initialized successfully");
        MOUSE_INITIALIZED.store(true, Ordering::SeqCst);
        Ok(())
    }
    
    fn send_command(&mut self, command: u8) -> Result<(), KernelError> {
        let mut timeout = 10000;
        unsafe {
            // Wait until we can send a command to the PS/2 controller
            while timeout > 0 {
                if self.status_port.read() & PS2_INPUT_FULL == 0 {
                    // Send the "write to second PS/2 port" command
                    self.command_port.write(PS2_WRITE_SECOND_PORT);
                    
                    // Wait again for the controller to process that command
                    timeout = 10000;
                    while timeout > 0 {
                        if self.status_port.read() & PS2_INPUT_FULL == 0 {
                            // Now send the actual mouse command
                            self.data_port.write(command);
                            return Ok(());
                        }
                        timeout -= 1;
                    }
                    return Err(KernelError::DeviceTimeout);
                }
                timeout -= 1;
            }
        }
        Err(KernelError::DeviceTimeout)
    }
    
    fn wait_for_data(&mut self) -> u8 {
        let mut timeout = 100000;
        unsafe {
            while timeout > 0 {
                if self.status_port.read() & PS2_OUTPUT_FULL != 0 {
                    return self.data_port.read();
                }
                timeout -= 1;
            }
        }
        0 // Timeout occurred
    }
    
    fn handle_packet(&mut self) {
        // Extract movement and button information from the packet
        let buttons = self.packet[0] & 0x07;
        
        // Process X movement
        let mut dx = self.packet[1] as i8;
        if self.packet[0] & MOUSE_X_SIGN != 0 {
            // X sign bit is set (negative movement)
            if dx > 0 {
                dx = -128 + (dx as i16 - 128) as i8;
            }
        }
        
        // Process Y movement (inverted for screen coordinates)
        let mut dy = -(self.packet[2] as i8);
        if self.packet[0] & MOUSE_Y_SIGN != 0 {
            // Y sign bit is set (negative movement)
            if dy < 0 {
                dy = -(dy as i16 as i8);
            } else {
                dy = -dy;
            }
        }
        
        // Update mouse state
        self.state.buttons = buttons;
        self.state.x = (self.state.x + dx as i16).max(0).min(640);
        self.state.y = (self.state.y + dy as i16).max(0).min(400);
        
        // Create a mouse event
        let event = MouseEvent {
            x: self.state.x,
            y: self.state.y,
            dx,
            dy,
            buttons: MouseButtons::from_bits(buttons),
        };
        
        // Add to the event queue if there's space
        if self.event_queue.len() < 16 {
            self.event_queue.push_back(event);
        }
        
        serial_println!("Mouse: x={}, y={}, buttons={:01b}", self.state.x, self.state.y, self.state.buttons);
    }
    
    fn handle_data(&mut self, data: u8) {
        self.packet[self.packet_index] = data;
        self.packet_index += 1;
        
        if self.packet_index >= 3 {
            self.handle_packet();
            self.packet_index = 0;
        }
    }
}

/// Mouse interrupt handler - called when mouse data is available
pub extern "x86-interrupt" fn mouse_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    if MOUSE_INITIALIZED.load(Ordering::SeqCst) {
        unsafe {
            let data = Port::<u8>::new(PS2_DATA_PORT).read();
            MOUSE.lock().handle_data(data);
            
            // Send End of Interrupt to the PIC
            crate::interrupts::pic::PIC_CONTROLLER.end_of_interrupt(
                crate::interrupts::pic::InterruptIndex::Mouse.as_u8()
            );
        }
    }
}

/// Initialize the PS/2 mouse
pub fn init() -> Result<(), KernelError> {
    // Initialize mouse
    MOUSE.lock().init()?;
    
    // Register mouse interrupt handler
    // TODO: Implement proper interrupt registration
    // For now, we'll skip registering the handler since we're in safe mode
    
    // Mark mouse as initialized
    MOUSE_INITIALIZED.store(true, Ordering::SeqCst);
    
    Ok(())
}

/// Get the next mouse event, if any
pub fn get_event() -> Option<MouseEvent> {
    if !MOUSE_INITIALIZED.load(Ordering::SeqCst) {
        return None;
    }
    
    // Get an event from the queue
    MOUSE.lock().event_queue.pop_front()
}

/// Get the current mouse state
pub fn get_state() -> MouseState {
    MOUSE.lock().state
}

/// Draw a simple cursor at the current mouse position
pub fn draw_cursor() {
    if MOUSE_INITIALIZED.load(Ordering::SeqCst) {
        let state = get_state();
        crate::drivers::vga_enhanced::write_at(
            state.y as usize, 
            state.x as usize,
            "X",
            if state.left_button() {
                crate::drivers::vga_enhanced::Color::Red
            } else {
                crate::drivers::vga_enhanced::Color::White
            },
            crate::drivers::vga_enhanced::Color::Black
        );
    }
}