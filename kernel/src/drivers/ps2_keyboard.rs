//! PS/2 keyboard driver
//! Handles keyboard input via the PS/2 controller

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
const PS2_DISABLE_FIRST_PORT: u8 = 0xAD;
const PS2_ENABLE_FIRST_PORT: u8 = 0xAE;
const PS2_TEST_CONTROLLER: u8 = 0xAA;
const PS2_TEST_FIRST_PORT: u8 = 0xAB;

// PS/2 device commands
const PS2_RESET_DEVICE: u8 = 0xFF;
const PS2_ENABLE_SCANNING: u8 = 0xF4;
const PS2_SET_DEFAULTS: u8 = 0xF6;

// Keyboard status flags
const KB_OUTPUT_FULL: u8 = 1 << 0;
const KB_INPUT_FULL: u8 = 1 << 1;

// Keyboard scan code set 1 (XT) - key pressed codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum KeyCode {
    Escape = 0x01,
    Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9, Key0,
    Minus, Equals, Backspace,
    Tab,
    Q, W, E, R, T, Y, U, I, O, P,
    LeftBracket, RightBracket, Enter,
    LeftControl,
    A, S, D, F, G, H, J, K, L,
    Semicolon, Apostrophe, Backtick,
    LeftShift,
    Backslash,
    Z, X, C, V, B, N, M,
    Comma, Period, Slash,
    RightShift,
    Keypad_Multiply,
    LeftAlt, Space, CapsLock,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10,
    NumLock, ScrollLock,
    Keypad_7, Keypad_8, Keypad_9, Keypad_Minus,
    Keypad_4, Keypad_5, Keypad_6, Keypad_Plus,
    Keypad_1, Keypad_2, Keypad_3, Keypad_0, Keypad_Decimal,
    // Extended keys and special keys
    Unknown = 0xFF
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub state: KeyState,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

// Global keyboard state
lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard::new());
}

static SHIFT_PRESSED: AtomicBool = AtomicBool::new(false);
static CTRL_PRESSED: AtomicBool = AtomicBool::new(false);
static ALT_PRESSED: AtomicBool = AtomicBool::new(false);

pub struct Keyboard {
    data_port: Port<u8>,
    status_port: PortReadOnly<u8>,
    command_port: PortWriteOnly<u8>,
    event_queue: VecDeque<KeyEvent>,
}

impl Keyboard {
    fn new() -> Self {
        Self {
            data_port: Port::new(PS2_DATA_PORT),
            status_port: PortReadOnly::new(PS2_STATUS_PORT),
            command_port: PortWriteOnly::new(PS2_COMMAND_PORT),
            event_queue: VecDeque::with_capacity(16),
        }
    }

    fn init(&mut self) -> Result<(), KernelError> {
        // Test PS/2 controller
        unsafe {
            self.command_port.write(PS2_TEST_CONTROLLER);
            if self.wait_for_data() != 0x55 {
                serial_println!("PS/2 controller test failed");
                return Err(KernelError::DeviceError(DeviceError::InitFailed));
            }

            // Test keyboard port
            self.command_port.write(PS2_TEST_FIRST_PORT);
            if self.wait_for_data() != 0x00 {
                serial_println!("PS/2 keyboard port test failed");
                return Err(KernelError::DeviceError(DeviceError::InitFailed));
            }

            // Enable keyboard port
            self.command_port.write(PS2_ENABLE_FIRST_PORT);

            // Reset keyboard
            self.data_port.write(PS2_RESET_DEVICE);
            if self.wait_for_data() != 0xAA {
                serial_println!("Keyboard reset failed");
                return Err(KernelError::DeviceError(DeviceError::InitFailed));
            }

            // Set default parameters
            self.data_port.write(PS2_SET_DEFAULTS);
            if self.wait_for_data() != 0xFA {
                serial_println!("Failed to set keyboard defaults");
                return Err(KernelError::DeviceError(DeviceError::InitFailed));
            }

            // Enable scanning
            self.data_port.write(PS2_ENABLE_SCANNING);
            if self.wait_for_data() != 0xFA {
                serial_println!("Failed to enable keyboard scanning");
                return Err(KernelError::DeviceError(DeviceError::InitFailed));
            }

            Ok(())
        }
    }

    fn send_command(&mut self, command: u8) -> Result<(), KernelError> {
        let mut timeout = 10000;
        unsafe {
            while timeout > 0 {
                if self.status_port.read() & KB_INPUT_FULL == 0 {
                    self.data_port.write(command);
                    return Ok(());
                }
                timeout -= 1;
            }
        }
        Err(KernelError::DeviceError(DeviceError::Timeout))
    }

    fn wait_for_data(&mut self) -> u8 {
        let mut timeout = 10000;
        unsafe {
            while timeout > 0 {
                if self.status_port.read() & KB_OUTPUT_FULL != 0 {
                    return self.data_port.read();
                }
                timeout -= 1;
            }
        }
        0 // Timeout occurred
    }

    fn handle_scancode(&mut self, scancode: u8) {
        // Determine if this is a key press or release
        let is_release = scancode & 0x80 != 0;
        let key_code = scancode & 0x7F;
        
        // Convert raw scancode to our KeyCode enum
        let key = match key_code {
            0x01 => KeyCode::Escape,
            0x02 => KeyCode::Key1,
            0x03 => KeyCode::Key2,
            0x04 => KeyCode::Key3,
            0x05 => KeyCode::Key4,
            0x06 => KeyCode::Key5,
            0x07 => KeyCode::Key6,
            0x08 => KeyCode::Key7,
            0x09 => KeyCode::Key8,
            0x0A => KeyCode::Key9,
            0x0B => KeyCode::Key0,
            0x0C => KeyCode::Minus,
            0x0D => KeyCode::Equals,
            0x0E => KeyCode::Backspace,
            0x0F => KeyCode::Tab,
            0x10 => KeyCode::Q,
            0x11 => KeyCode::W,
            0x12 => KeyCode::E,
            0x13 => KeyCode::R,
            0x14 => KeyCode::T,
            0x15 => KeyCode::Y,
            0x16 => KeyCode::U,
            0x17 => KeyCode::I,
            0x18 => KeyCode::O,
            0x19 => KeyCode::P,
            0x1A => KeyCode::LeftBracket,
            0x1B => KeyCode::RightBracket,
            0x1C => KeyCode::Enter,
            0x1D => KeyCode::LeftControl,
            0x1E => KeyCode::A,
            0x1F => KeyCode::S,
            0x20 => KeyCode::D,
            0x21 => KeyCode::F,
            0x22 => KeyCode::G,
            0x23 => KeyCode::H,
            0x24 => KeyCode::J,
            0x25 => KeyCode::K,
            0x26 => KeyCode::L,
            0x27 => KeyCode::Semicolon,
            0x28 => KeyCode::Apostrophe,
            0x29 => KeyCode::Backtick,
            0x2A => KeyCode::LeftShift,
            0x2B => KeyCode::Backslash,
            0x2C => KeyCode::Z,
            0x2D => KeyCode::X,
            0x2E => KeyCode::C,
            0x2F => KeyCode::V,
            0x30 => KeyCode::B,
            0x31 => KeyCode::N,
            0x32 => KeyCode::M,
            0x33 => KeyCode::Comma,
            0x34 => KeyCode::Period,
            0x35 => KeyCode::Slash,
            0x36 => KeyCode::RightShift,
            0x37 => KeyCode::Keypad_Multiply,
            0x38 => KeyCode::LeftAlt,
            0x39 => KeyCode::Space,
            _ => KeyCode::Unknown,
        };

        // Update modifier key states
        match key {
            KeyCode::LeftShift | KeyCode::RightShift => {
                SHIFT_PRESSED.store(!is_release, Ordering::SeqCst);
            }
            KeyCode::LeftControl => {
                CTRL_PRESSED.store(!is_release, Ordering::SeqCst);
            }
            KeyCode::LeftAlt => {
                ALT_PRESSED.store(!is_release, Ordering::SeqCst);
            }
            _ => {}
        }

        // Create key event
        let event = KeyEvent {
            code: key,
            state: if is_release { KeyState::Released } else { KeyState::Pressed },
            shift: SHIFT_PRESSED.load(Ordering::SeqCst),
            ctrl: CTRL_PRESSED.load(Ordering::SeqCst),
            alt: ALT_PRESSED.load(Ordering::SeqCst),
        };

        // Add to event queue
        if self.event_queue.len() < 16 {
            self.event_queue.push_back(event);
        }
        
        // Print debug info
        if !is_release {
            let c = match key {
                KeyCode::A => 'a',
                KeyCode::B => 'b',
                KeyCode::C => 'c',
                KeyCode::D => 'd',
                KeyCode::E => 'e',
                KeyCode::F => 'f',
                KeyCode::G => 'g',
                KeyCode::H => 'h',
                KeyCode::I => 'i',
                KeyCode::J => 'j',
                KeyCode::K => 'k',
                KeyCode::L => 'l',
                KeyCode::M => 'm',
                KeyCode::N => 'n',
                KeyCode::O => 'o',
                KeyCode::P => 'p',
                KeyCode::Q => 'q',
                KeyCode::R => 'r',
                KeyCode::S => 's',
                KeyCode::T => 't',
                KeyCode::U => 'u',
                KeyCode::V => 'v',
                KeyCode::W => 'w',
                KeyCode::X => 'x',
                KeyCode::Y => 'y',
                KeyCode::Z => 'z',
                KeyCode::Key1 => '1',
                KeyCode::Key2 => '2',
                KeyCode::Key3 => '3',
                KeyCode::Key4 => '4',
                KeyCode::Key5 => '5',
                KeyCode::Key6 => '6',
                KeyCode::Key7 => '7',
                KeyCode::Key8 => '8',
                KeyCode::Key9 => '9',
                KeyCode::Key0 => '0',
                KeyCode::Enter => '\n',
                KeyCode::Space => ' ',
                _ => '?',
            };
            serial_println!("Key pressed: {:?} ({})", key, c);
        }
    }
}

/// Keyboard interrupt handler - called when a key is pressed/released
pub extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    unsafe {
        let scancode = Port::<u8>::new(PS2_DATA_PORT).read();
        KEYBOARD.lock().handle_scancode(scancode);
        
        // Send EOI to PIC
        crate::interrupts::pic::PIC_CONTROLLER.lock().notify_end_of_interrupt(
            crate::interrupts::pic::InterruptIndex::Keyboard.as_u8()
        );
    }
}

/// Initialize the PS/2 keyboard
pub fn init() -> Result<(), KernelError> {
    serial_println!("DEBUG: Initializing PS/2 keyboard");
    
    // Initialize keyboard hardware
    KEYBOARD.lock().init()?;
    
    // We don't need to register the interrupt handler here because
    // it's already set up in the IDT initialization in interrupts/mod.rs
    serial_println!("DEBUG: PS/2 keyboard initialized successfully");
    
    Ok(())
}

/// Get the next keyboard event, if any
pub fn get_event() -> Option<KeyEvent> {
    // Removed SAFE MODE direct port reading logic.
    // Relies on interrupt handler populating the queue.
    
    // Try to get an event from the queue
    let event = KEYBOARD.lock().event_queue.pop_front();
    
    // Log if we're returning an event
    if let Some(ref e) = event {
        serial_println!("DEBUG: Keyboard returning event: code={:?}, state={:?}, shift={}, ctrl={}, alt={}", 
            e.code, e.state, e.shift, e.ctrl, e.alt);
    }
    
    event
}

/// Wait for a key press and return it
pub fn wait_for_key() -> KeyEvent {
    loop {
        if let Some(event) = get_event() {
            if event.state == KeyState::Pressed {
                return event;
            }
        }
        x86_64::instructions::hlt();
    }
}

/// Handle a scancode from the keyboard interrupt handler
/// This function is called directly from the interrupt handler
pub fn direct_handle_scancode(scancode: u8) {
    // Write directly to COM1 for debugging
    unsafe {
        let com1_data_port: *mut u8 = 0x3F8 as *mut u8;
        *com1_data_port = b'D'; // D for direct handle
    }
    
    // Use a minimal approach to handle the scancode
    // Skip locking the keyboard for now to avoid potential deadlocks
    let is_release = scancode & 0x80 != 0;
    let key_code = scancode & 0x7F;
    
    // Update modifier key states directly without locking
    match key_code {
        0x2A | 0x36 => { // LeftShift or RightShift
            SHIFT_PRESSED.store(!is_release, Ordering::SeqCst);
        },
        0x1D => { // LeftControl
            CTRL_PRESSED.store(!is_release, Ordering::SeqCst);
        },
        0x38 => { // LeftAlt
            ALT_PRESSED.store(!is_release, Ordering::SeqCst);
        },
        _ => {}
    }
    
    // Write end marker to COM1
    unsafe {
        let com1_data_port: *mut u8 = 0x3F8 as *mut u8;
        *com1_data_port = b'F'; // F for finished
    }
    
    // Only after the essential processing, try locking the keyboard
    // to update the event queue if we can do so without blocking
    let mut try_count = 0;
    while try_count < 3 {
        if let Some(mut kb) = KEYBOARD.try_lock() {
            // Successfully acquired the lock, process normally
            kb.handle_scancode(scancode);
            break;
        }
        try_count += 1;
    }
} 