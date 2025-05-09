//! Window module for UniverseK OS GUI
//! Implements window management for the graphical user interface

use crate::drivers::vga_enhanced::{self, Color};
use crate::serial_println;
use crate::errors::KernelError;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

/// Colors used for windows
pub const WINDOW_TITLE_ACTIVE: Color = Color::Blue;
pub const WINDOW_TITLE_INACTIVE: Color = Color::DarkGray;
pub const WINDOW_TEXT: Color = Color::White;
pub const WINDOW_BACKGROUND: Color = Color::LightGray;
pub const WINDOW_BORDER: Color = Color::White;

/// Window handle for shared access to windows
pub type WindowHandle = Arc<Mutex<Window>>;

/// Callback for handling input in a window
pub type InputCallback = Box<dyn Fn(&str) -> Result<(), KernelError> + Send>;

/// A window in the GUI
pub struct Window {
    /// Window title
    title: String,
    /// Window position
    x: usize,
    y: usize,
    /// Window size
    width: usize,
    height: usize,
    /// Window content buffer
    content: String,
    /// Input buffer (for shell-like windows)
    input_buffer: String,
    /// Input callback
    input_callback: Option<InputCallback>,
    /// Whether this window accepts input
    accepts_input: bool,
}

impl Window {
    /// Create a new window
    pub fn new(title: &str, x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            title: title.to_string(),
            x,
            y,
            width: width.max(10), // Minimum size
            height: height.max(5),
            content: String::new(),
            input_buffer: String::new(),
            input_callback: None,
            accepts_input: false,
        }
    }
    
    /// Enable input handling for this window
    pub fn enable_input(&mut self, callback: InputCallback) {
        self.accepts_input = true;
        self.input_callback = Some(callback);
    }
    
    /// Add text to the window's content
    pub fn add_text(&mut self, text: &str) {
        self.content.push_str(text);
        
        // Limit content size
        if self.content.len() > 1000 {
            self.content = self.content[self.content.len() - 1000..].to_string();
        }
    }
    
    /// Clear the window's content
    pub fn clear(&mut self) {
        self.content.clear();
    }
    
    /// Draw the window
    pub fn draw(&self, is_active: bool) -> Result<(), KernelError> {
        // Draw window border and background
        let title_color = if is_active { WINDOW_TITLE_ACTIVE } else { WINDOW_TITLE_INACTIVE };
        
        // Top border with title
        for i in 0..self.width {
            let c = if i == 0 {
                '╔' // Top-left corner
            } else if i == self.width - 1 {
                '╗' // Top-right corner
            } else {
                '═' // Horizontal border
            };
            vga_enhanced::write_at(self.y, self.x + i, &c.to_string(), WINDOW_TEXT, title_color);
        }
        
        // Draw title
        let title = if self.title.len() > self.width - 4 {
            // Truncate title if it's too long
            let mut truncated = self.title[0..self.width - 7].to_string();
            truncated.push_str("...");
            truncated
        } else {
            self.title.clone()
        };
        
        vga_enhanced::write_at(self.y, self.x + 2, &title, WINDOW_TEXT, title_color);
        
        // Draw close button
        vga_enhanced::write_at(self.y, self.x + self.width - 2, "X", Color::White, Color::Red);
        
        // Side borders and content area
        for i in 1..self.height - 1 {
            // Left border
            vga_enhanced::write_at(self.y + i, self.x, "║", WINDOW_BORDER, WINDOW_BACKGROUND);
            // Right border
            vga_enhanced::write_at(self.y + i, self.x + self.width - 1, "║", WINDOW_BORDER, WINDOW_BACKGROUND);
            
            // Window background
            for j in 1..self.width - 1 {
                vga_enhanced::write_at(self.y + i, self.x + j, " ", WINDOW_TEXT, WINDOW_BACKGROUND);
            }
        }
        
        // Bottom border
        for i in 0..self.width {
            let c = if i == 0 {
                '╚' // Bottom-left corner
            } else if i == self.width - 1 {
                '╝' // Bottom-right corner
            } else {
                '═' // Horizontal border
            };
            vga_enhanced::write_at(self.y + self.height - 1, self.x + i, &c.to_string(), WINDOW_BORDER, WINDOW_BACKGROUND);
        }
        
        // Draw content
        self.draw_content()?;
        
        // Draw input buffer if window accepts input
        if self.accepts_input {
            self.draw_input_line()?;
        }
        
        Ok(())
    }
    
    /// Draw the window's content
    fn draw_content(&self) -> Result<(), KernelError> {
        // Simple content drawing - just split by newlines
        let lines: Vec<&str> = self.content.split('\n').collect();
        let available_height = self.height - 3; // Account for borders and input line
        
        // Draw only the last N lines that fit
        let start_line = if lines.len() > available_height {
            lines.len() - available_height
        } else {
            0
        };
        
        for (i, line) in lines[start_line..].iter().enumerate() {
            if i >= available_height {
                break;
            }
            
            // Truncate line if it's too long
            let display_line = if line.len() > self.width - 2 {
                &line[0..self.width - 5]
            } else {
                line
            };
            
            vga_enhanced::write_at(self.y + 1 + i, self.x + 1, display_line, WINDOW_TEXT, WINDOW_BACKGROUND);
        }
        
        Ok(())
    }
    
    /// Draw the input line if this window accepts input
    fn draw_input_line(&self) -> Result<(), KernelError> {
        if self.accepts_input {
            let y = self.y + self.height - 2;
            
            // Draw input prompt
            vga_enhanced::write_at(y, self.x + 1, "> ", Color::Green, WINDOW_BACKGROUND);
            
            // Draw input buffer
            let buffer_display = if self.input_buffer.len() > self.width - 4 {
                &self.input_buffer[self.input_buffer.len() - (self.width - 4)..]
            } else {
                &self.input_buffer
            };
            
            vga_enhanced::write_at(y, self.x + 3, buffer_display, WINDOW_TEXT, WINDOW_BACKGROUND);
            
            // Draw cursor
            let cursor_pos = self.x + 3 + buffer_display.len();
            if cursor_pos < self.x + self.width - 1 {
                vga_enhanced::write_at(y, cursor_pos, "_", WINDOW_TEXT, WINDOW_BACKGROUND);
            }
        }
        
        Ok(())
    }
    
    /// Handle keyboard input
    pub fn handle_key(&mut self, key: char) -> Result<(), KernelError> {
        if !self.accepts_input {
            return Ok(());
        }
        
        match key {
            '\n' => {
                // Process input
                let input = self.input_buffer.clone();
                
                // Add the input line to the content first
                self.add_text(&format!("> {}\n", input));
                
                // Clear input buffer before calling callback
                self.input_buffer.clear();
                
                // Call callback if available
                if let Some(ref callback) = self.input_callback {
                    // Now call the callback with the input
                    callback(&input)?;
                }
            },
            '\x08' => {
                // Backspace
                if !self.input_buffer.is_empty() {
                    self.input_buffer.pop();
                }
            },
            c if c.is_ascii_graphic() || c == ' ' => {
                // Add character to input
                self.input_buffer.push(c);
            },
            _ => {}
        }
        
        Ok(())
    }
    
    /// Handle a mouse click
    pub fn handle_click(&mut self, x: usize, y: usize) -> Result<(), KernelError> {
        // For now, just focus the window (done by the desktop)
        // Could handle scrolling, buttons, etc. here
        Ok(())
    }
    
    /// Check if a point is inside this window
    pub fn contains_point(&self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.x + self.width && 
        y >= self.y && y < self.y + self.height
    }
    
    /// Check if a point is on the close button
    pub fn is_on_close_button(&self, x: usize, y: usize) -> bool {
        y == self.y && x == self.x + self.width - 2
    }
}

/// Create a new window with a handle
pub fn create_window(title: &str, x: usize, y: usize, width: usize, height: usize) -> WindowHandle {
    let window = Window::new(title, x, y, width, height);
    Arc::new(Mutex::new(window))
} 