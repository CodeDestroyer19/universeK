//! Enhanced VGA text mode driver
//! Extends the basic VGA buffer implementation with more features

use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use crate::errors::KernelError;
use crate::serial_println;

// VGA text buffer constants
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const VGA_BUFFER_ADDR: usize = 0xb8000;

// VGA colors
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
    saved_positions: [(usize, usize); 10], // Store cursor positions for later restore
}

impl Writer {
    fn new() -> Self {
        Self {
            column_position: 0,
            row_position: 0,
            color_code: ColorCode::new(Color::White, Color::Black),
            buffer: unsafe { &mut *(VGA_BUFFER_ADDR as *mut Buffer) },
            saved_positions: [(0, 0); 10],
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\r' => self.carriage_return(),
            b'\t' => self.write_tab(),
            b'\x08' => self.backspace(), // Backspace
            // Handle printable ASCII
            0x20..=0x7e => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = self.row_position;
                let col = self.column_position;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });
                self.column_position += 1;
            }
            _ => {} // Ignore other bytes
        }
    }

    fn write_tab(&mut self) {
        // Insert spaces to reach next tab stop (every 8 columns)
        let spaces = 8 - (self.column_position % 8);
        for _ in 0..spaces {
            self.write_byte(b' ');
        }
    }

    fn backspace(&mut self) {
        if self.column_position > 0 {
            self.column_position -= 1;
            self.buffer.chars[self.row_position][self.column_position].write(ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            });
        } else if self.row_position > 0 {
            // Move to the end of the previous line
            self.row_position -= 1;
            self.column_position = BUFFER_WIDTH - 1;
            // Clear the last character
            self.buffer.chars[self.row_position][self.column_position].write(ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            });
        }
    }

    fn carriage_return(&mut self) {
        self.column_position = 0;
    }

    fn new_line(&mut self) {
        self.column_position = 0;
        if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            // Scroll the buffer up
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
            // Clear the last row
            self.clear_row(BUFFER_HEIGHT - 1);
        }
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.column_position = 0;
        self.row_position = 0;
    }

    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }

    pub fn cursor_position(&self) -> (usize, usize) {
        (self.row_position, self.column_position)
    }

    pub fn set_cursor_position(&mut self, row: usize, column: usize) {
        if row < BUFFER_HEIGHT && column < BUFFER_WIDTH {
            self.row_position = row;
            self.column_position = column;
        }
    }

    pub fn save_cursor(&mut self, slot: usize) {
        if slot < 10 {
            self.saved_positions[slot] = (self.row_position, self.column_position);
        }
    }

    pub fn restore_cursor(&mut self, slot: usize) {
        if slot < 10 {
            let (row, col) = self.saved_positions[slot];
            self.set_cursor_position(row, col);
        }
    }

    pub fn draw_box(&mut self, x: usize, y: usize, width: usize, height: usize) {
        // Save current position and color
        let saved_pos = (self.row_position, self.column_position);
        let saved_color = self.color_code;

        // Ensure box fits on screen
        let max_width = if x + width > BUFFER_WIDTH { BUFFER_WIDTH - x } else { width };
        let max_height = if y + height > BUFFER_HEIGHT { BUFFER_HEIGHT - y } else { height };

        // Box drawing characters
        let top_left = b'\xDA';     // ┌
        let top_right = b'\xBF';    // ┐
        let bottom_left = b'\xC0';  // └
        let bottom_right = b'\xD9'; // ┘
        let horizontal = b'\xC4';   // ─
        let vertical = b'\xB3';     // │

        // Draw top border
        self.set_cursor_position(y, x);
        self.write_byte(top_left);
        for i in 1..max_width-1 {
            self.set_cursor_position(y, x + i);
            self.write_byte(horizontal);
        }
        if max_width > 1 {
            self.set_cursor_position(y, x + max_width - 1);
            self.write_byte(top_right);
        }

        // Draw side borders
        for i in 1..max_height-1 {
            self.set_cursor_position(y + i, x);
            self.write_byte(vertical);
            if max_width > 1 {
                self.set_cursor_position(y + i, x + max_width - 1);
                self.write_byte(vertical);
            }
        }

        // Draw bottom border
        if max_height > 1 {
            self.set_cursor_position(y + max_height - 1, x);
            self.write_byte(bottom_left);
            for i in 1..max_width-1 {
                self.set_cursor_position(y + max_height - 1, x + i);
                self.write_byte(horizontal);
            }
            if max_width > 1 {
                self.set_cursor_position(y + max_height - 1, x + max_width - 1);
                self.write_byte(bottom_right);
            }
        }

        // Restore position and color
        self.row_position = saved_pos.0;
        self.column_position = saved_pos.1;
        self.color_code = saved_color;
    }

    pub fn draw_shadow(&mut self, x: usize, y: usize, width: usize, height: usize) {
        // Save current position and color
        let saved_pos = (self.row_position, self.column_position);
        let saved_color = self.color_code;

        // Set shadow color
        self.set_color(Color::Black, Color::DarkGray);

        // Draw shadow
        for row in y+1..y+height+1 {
            if row >= BUFFER_HEIGHT {
                break;
            }
            for col in x+2..x+width+2 {
                if col >= BUFFER_WIDTH {
                    break;
                }
                let char = self.buffer.chars[row][col].read();
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: char.ascii_character,
                    color_code: self.color_code,
                });
            }
        }

        // Restore position and color
        self.row_position = saved_pos.0;
        self.column_position = saved_pos.1;
        self.color_code = saved_color;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new());
}

// Global interface functions

/// Clear the screen
pub fn clear_screen() {
    serial_println!("DEBUG: vga_enhanced::clear_screen - Clearing screen");
    let mut writer = WRITER.lock();
    writer.clear_screen();
    serial_println!("DEBUG: vga_enhanced::clear_screen - Screen cleared");
}

/// Set the text colors
pub fn set_color(foreground: Color, background: Color) {
    WRITER.lock().set_color(foreground, background);
}

/// Get current cursor position
pub fn get_cursor_position() -> (usize, usize) {
    WRITER.lock().cursor_position()
}

/// Set cursor position
pub fn set_cursor_position(row: usize, column: usize) {
    serial_println!("DEBUG: vga_enhanced::set_cursor_position - Setting to row={}, col={}", row, column);
    let mut writer = WRITER.lock();
    writer.set_cursor_position(row, column);
}

/// Draw a box on the screen
pub fn draw_box(x: usize, y: usize, width: usize, height: usize) {
    WRITER.lock().draw_box(x, y, width, height);
}

/// Draw a shadowed box
pub fn draw_shadowed_box(x: usize, y: usize, width: usize, height: usize) {
    serial_println!("DEBUG: vga_enhanced::draw_shadowed_box - Drawing box at x={}, y={}, width={}, height={}", 
        x, y, width, height);
    
    // Bounds checking to prevent panics
    if x >= BUFFER_WIDTH || y >= BUFFER_HEIGHT {
        serial_println!("DEBUG: vga_enhanced::draw_shadowed_box - Starting position out of bounds");
        return;
    }
    
    if x + width > BUFFER_WIDTH || y + height > BUFFER_HEIGHT {
        serial_println!("DEBUG: vga_enhanced::draw_shadowed_box - Box extends beyond screen boundaries");
        // Adjust width and height to fit screen
        let adj_width = core::cmp::min(BUFFER_WIDTH - x, width);
        let adj_height = core::cmp::min(BUFFER_HEIGHT - y, height);
        
        serial_println!("DEBUG: vga_enhanced::draw_shadowed_box - Adjusted to width={}, height={}", 
            adj_width, adj_height);
        
        let mut writer = WRITER.lock();
        writer.draw_box(x, y, adj_width, adj_height);
        writer.draw_shadow(x, y, adj_width, adj_height);
    } else {
        let mut writer = WRITER.lock();
        writer.draw_box(x, y, width, height);
        writer.draw_shadow(x, y, width, height);
    }
    
    serial_println!("DEBUG: vga_enhanced::draw_shadowed_box - Box drawn successfully");
}

/// Write a string at a specific position with specific colors
pub fn write_at(row: usize, column: usize, s: &str, fg: Color, bg: Color) {    
    // Bounds checking
    if row >= BUFFER_HEIGHT {
        serial_println!("DEBUG: vga_enhanced::write_at - Row {} out of bounds (max {})", row, BUFFER_HEIGHT-1);
        return;
    }
    
    let mut writer = WRITER.lock();
    let saved_position = writer.cursor_position();
    let saved_color = writer.color_code;
    
    writer.set_color(fg, bg);
    writer.set_cursor_position(row, column);
    writer.write_string(s);
    
    // Restore previous state
    writer.color_code = saved_color;
    writer.set_cursor_position(saved_position.0, saved_position.1);
}

/// Create a simple message box with a message
pub fn message_box(title: &str, message: &str) {
    // Calculate box dimensions
    let width = message.len() + 4;
    let x = (BUFFER_WIDTH - width) / 2;
    let y = BUFFER_HEIGHT / 3;
    
    // Draw box with shadow
    draw_shadowed_box(x, y, width, 5);
    
    // Draw title
    let title_x = x + (width - title.len()) / 2;
    write_at(y, title_x, title, Color::White, Color::Blue);
    
    // Draw message
    write_at(y + 2, x + 2, message, Color::White, Color::Black);
}

/// Initialize the VGA driver
pub fn init() -> Result<(), KernelError> {
    serial_println!("Initializing enhanced VGA text mode driver");
    
    // Clear the screen
    clear_screen();
    
    // Draw a welcome message
    set_color(Color::White, Color::Blue);
    write_at(0, 0, "UniverseK OS - Enhanced VGA Driver", Color::White, Color::Blue);
    set_color(Color::LightGray, Color::Black);
    
    Ok(())
}

/// Set cursor position (alias for set_cursor_position for backward compatibility)
pub fn set_cursor(row: usize, column: usize) {
    set_cursor_position(row, column);
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
} 