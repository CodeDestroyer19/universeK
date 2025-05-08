//! Shell implementation for UniverseK OS
//! Provides a simple command-line interface for the kernel

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::serial_println;
use crate::drivers::vga_enhanced::{self, Color};
use crate::drivers::ps2_keyboard::{KeyCode, KeyEvent, KeyState};
use crate::fs;
use crate::errors::KernelError;

/// Maximum number of command history entries
const MAX_HISTORY: usize = 10;

/// Shell state and configuration
pub struct Shell {
    /// Current command line
    input_buffer: String,
    /// Cursor position in the input buffer
    cursor_position: usize,
    /// Command history
    history: Vec<String>,
    /// Current position in history (when navigating with up/down arrows)
    history_position: usize,
    /// Current working directory
    current_dir: String,
    /// Shell prompt string
    prompt: String,
    /// Shell window position and size
    window_x: usize,
    window_y: usize,
    window_width: usize,
    window_height: usize,
}

impl Shell {
    /// Create a new shell instance
    pub fn new() -> Self {
        Self {
            input_buffer: String::new(),
            cursor_position: 0,
            history: Vec::new(),
            history_position: 0,
            current_dir: "/".to_string(),
            prompt: "$ ".to_string(),
            window_x: 1,
            window_y: 2,
            window_width: 78,
            window_height: 22,
        }
    }
    
    /// Set the shell prompt
    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt = prompt.to_string();
    }
    
    /// Initialize the shell
    pub fn init(&mut self) {
        serial_println!("DEBUG: Shell.init() - Starting shell initialization");
        
        // Try to clear the screen, catch any panics
        serial_println!("DEBUG: Shell.init() - Clearing screen");
        self.clear_screen();
        
        // Display welcome message
        serial_println!("DEBUG: Shell.init() - Displaying welcome message");
        self.display_welcome();
        
        // Draw initial prompt
        serial_println!("DEBUG: Shell.init() - Drawing initial prompt");
        self.draw_prompt();
        
        // Log initialization success
        serial_println!("DEBUG: Shell.init() - Shell initialization complete");
    }
    
    /// Clear the shell screen
    pub fn clear_screen(&self) {
        serial_println!("DEBUG: Shell.clear_screen() - Clearing VGA screen");
        // Check if VGA is working
        let test_msg = "Testing VGA";
        
        // Try to write something first to check if VGA is responsive
        vga_enhanced::write_at(0, 0, test_msg, Color::White, Color::Black);
        
        // Now proceed with normal screen setup
        vga_enhanced::clear_screen();
        
        // Draw title bar
        for i in 0..80 {
            vga_enhanced::write_at(0, i, " ", Color::White, Color::Blue);
        }
        
        // Draw title and border
        vga_enhanced::write_at(0, 2, " UniverseK OS Terminal ", Color::White, Color::Blue);
        vga_enhanced::write_at(0, 68, " [ESC] Exit ", Color::White, Color::Blue);
        
        // Draw border around terminal area
        vga_enhanced::draw_shadowed_box(1, 1, 78, 22);
        serial_println!("DEBUG: Shell.clear_screen() - Screen cleared successfully");
    }
    
    /// Display welcome message
    fn display_welcome(&self) {
        let welcome_text = concat!(
            "Welcome to UniverseK OS Terminal\n",
            "Type 'help' for a list of available commands.\n",
        );
        
        vga_enhanced::write_at(2, 2, welcome_text, Color::LightGreen, Color::Black);
    }
    
    /// Draw the command prompt
    fn draw_prompt(&self) {
        // Format the prompt with current directory
        let full_prompt = format!("{}:{}{}", "user", self.current_dir, self.prompt);
        vga_enhanced::write_at(self.window_height - 2, 2, &full_prompt, 
                             Color::LightCyan, Color::Black);
    }
    
    /// Handle keyboard input
    pub fn handle_key(&mut self, key_event: KeyEvent) -> bool {
        // Only process key down events
        if key_event.state != KeyState::Pressed {
            return false;
        }
        
        match key_event.code {
            // Handle special keys
            KeyCode::Escape => return true, // Signal to exit shell
            KeyCode::Enter => {
                self.execute_command();
                return false;
            },
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.input_buffer.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                    self.redraw_input_line();
                }
                return false;
            },
            KeyCode::LeftBracket if key_event.ctrl => { // Use Ctrl+[ as left arrow
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.update_cursor();
                }
                return false;
            },
            KeyCode::RightBracket if key_event.ctrl => { // Use Ctrl+] as right arrow
                if self.cursor_position < self.input_buffer.len() {
                    self.cursor_position += 1;
                    self.update_cursor();
                }
                return false;
            },
            KeyCode::P if key_event.ctrl => { // Use Ctrl+P as up arrow
                self.navigate_history_up();
                return false;
            },
            KeyCode::N if key_event.ctrl => { // Use Ctrl+N as down arrow
                self.navigate_history_down();
                return false;
            },
            // Handle regular keys (convert to ASCII/Unicode)
            _ => {
                if let Some(c) = self.key_to_char(key_event) {
                    self.input_buffer.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                    self.redraw_input_line();
                }
                return false;
            },
        }
    }
    
    /// Convert a key event to a character
    fn key_to_char(&self, key_event: KeyEvent) -> Option<char> {
        let key_code = key_event.code;
        let shift = key_event.shift;
        
        match key_code {
            KeyCode::Key1 => Some(if shift { '!' } else { '1' }),
            KeyCode::Key2 => Some(if shift { '@' } else { '2' }),
            KeyCode::Key3 => Some(if shift { '#' } else { '3' }),
            KeyCode::Key4 => Some(if shift { '$' } else { '4' }),
            KeyCode::Key5 => Some(if shift { '%' } else { '5' }),
            KeyCode::Key6 => Some(if shift { '^' } else { '6' }),
            KeyCode::Key7 => Some(if shift { '&' } else { '7' }),
            KeyCode::Key8 => Some(if shift { '*' } else { '8' }),
            KeyCode::Key9 => Some(if shift { '(' } else { '9' }),
            KeyCode::Key0 => Some(if shift { ')' } else { '0' }),
            KeyCode::A => Some(if shift { 'A' } else { 'a' }),
            KeyCode::B => Some(if shift { 'B' } else { 'b' }),
            KeyCode::C => Some(if shift { 'C' } else { 'c' }),
            KeyCode::D => Some(if shift { 'D' } else { 'd' }),
            KeyCode::E => Some(if shift { 'E' } else { 'e' }),
            KeyCode::F => Some(if shift { 'F' } else { 'f' }),
            KeyCode::G => Some(if shift { 'G' } else { 'g' }),
            KeyCode::H => Some(if shift { 'H' } else { 'h' }),
            KeyCode::I => Some(if shift { 'I' } else { 'i' }),
            KeyCode::J => Some(if shift { 'J' } else { 'j' }),
            KeyCode::K => Some(if shift { 'K' } else { 'k' }),
            KeyCode::L => Some(if shift { 'L' } else { 'l' }),
            KeyCode::M => Some(if shift { 'M' } else { 'm' }),
            KeyCode::N => Some(if shift { 'N' } else { 'n' }),
            KeyCode::O => Some(if shift { 'O' } else { 'o' }),
            KeyCode::P => Some(if shift { 'P' } else { 'p' }),
            KeyCode::Q => Some(if shift { 'Q' } else { 'q' }),
            KeyCode::R => Some(if shift { 'R' } else { 'r' }),
            KeyCode::S => Some(if shift { 'S' } else { 's' }),
            KeyCode::T => Some(if shift { 'T' } else { 't' }),
            KeyCode::U => Some(if shift { 'U' } else { 'u' }),
            KeyCode::V => Some(if shift { 'V' } else { 'v' }),
            KeyCode::W => Some(if shift { 'W' } else { 'w' }),
            KeyCode::X => Some(if shift { 'X' } else { 'x' }),
            KeyCode::Y => Some(if shift { 'Y' } else { 'y' }),
            KeyCode::Z => Some(if shift { 'Z' } else { 'z' }),
            KeyCode::Space => Some(' '),
            KeyCode::Minus => Some(if shift { '_' } else { '-' }),
            KeyCode::Equals => Some(if shift { '+' } else { '=' }),
            KeyCode::LeftBracket => Some(if shift { '{' } else { '[' }),
            KeyCode::RightBracket => Some(if shift { '}' } else { ']' }),
            KeyCode::Backslash => Some(if shift { '|' } else { '\\' }),
            KeyCode::Semicolon => Some(if shift { ':' } else { ';' }),
            KeyCode::Apostrophe => Some(if shift { '"' } else { '\'' }),
            KeyCode::Backtick => Some(if shift { '~' } else { '`' }),
            KeyCode::Comma => Some(if shift { '<' } else { ',' }),
            KeyCode::Period => Some(if shift { '>' } else { '.' }),
            KeyCode::Slash => Some(if shift { '?' } else { '/' }),
            _ => None,
        }
    }
    
    /// Redraw the input line (current command being typed)
    fn redraw_input_line(&self) {
        // Clear the input line first
        for i in 0..self.window_width - 2 {
            vga_enhanced::write_at(self.window_height - 2, 2 + i, " ", 
                                 Color::White, Color::Black);
        }
        
        // Draw the prompt
        self.draw_prompt();
        
        // Draw the current input
        let prompt_len = format!("{}:{}{}", "user", self.current_dir, self.prompt).len();
        vga_enhanced::write_at(self.window_height - 2, 2 + prompt_len, 
                             &self.input_buffer, Color::White, Color::Black);
        
        // Position the cursor
        self.update_cursor();
    }
    
    /// Update the cursor position
    fn update_cursor(&self) {
        let prompt_len = format!("{}:{}{}", "user", self.current_dir, self.prompt).len();
        vga_enhanced::set_cursor_position(self.window_height - 2, 2 + prompt_len + self.cursor_position);
    }
    
    /// Navigate command history upward (older commands)
    fn navigate_history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        
        if self.history_position < self.history.len() {
            self.history_position += 1;
            let index = self.history.len() - self.history_position;
            self.input_buffer = self.history[index].clone();
            self.cursor_position = self.input_buffer.len();
            self.redraw_input_line();
        }
    }
    
    /// Navigate command history downward (newer commands)
    fn navigate_history_down(&mut self) {
        if self.history_position > 0 {
            self.history_position -= 1;
            
            if self.history_position == 0 {
                // Back to empty line at bottom of history
                self.input_buffer.clear();
            } else {
                let index = self.history.len() - self.history_position;
                self.input_buffer = self.history[index].clone();
            }
            
            self.cursor_position = self.input_buffer.len();
            self.redraw_input_line();
        }
    }
    
    /// Execute the current command
    fn execute_command(&mut self) {
        // Add the command to output area with prompt
        let prompt = format!("{}:{}{}", "user", self.current_dir, self.prompt);
        let input_copy = self.input_buffer.clone();
        self.output_line(&format!("{}{}", prompt, input_copy));
        
        // Add to history if not empty and not the same as the last command
        if !self.input_buffer.is_empty() {
            if self.history.is_empty() || self.history.last().unwrap() != &self.input_buffer {
                self.history.push(self.input_buffer.clone());
                // Trim history if it gets too long
                if self.history.len() > MAX_HISTORY {
                    self.history.remove(0);
                }
            }
            self.history_position = 0;
        }
        
        // Process command
        let command = self.input_buffer.trim().to_string();
        if !command.is_empty() {
            let result = self.process_command(&command);
            if let Err(e) = result {
                self.output_line(&format!("Error: {:?}", e));
            }
        }
        
        // Clear input and redraw prompt
        self.input_buffer.clear();
        self.cursor_position = 0;
        self.redraw_input_line();
    }
    
    /// Process a command and execute it
    fn process_command(&mut self, command: &str) -> Result<(), KernelError> {
        // Split command and arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }
        
        let cmd = parts[0];
        let args = &parts[1..];
        
        match cmd {
            "help" => self.cmd_help(),
            "echo" => self.cmd_echo(args),
            "ls" | "dir" => self.cmd_ls(args),
            "cd" => self.cmd_cd(args),
            "cat" => self.cmd_cat(args),
            "cls" | "clear" => self.cmd_clear(),
            "pwd" => self.cmd_pwd(),
            "touch" | "mkfile" => self.cmd_touch(args),
            "mkdir" => self.cmd_mkdir(args),
            "rm" => self.cmd_rm(args),
            "reboot" => self.cmd_reboot(),
            "version" => self.cmd_version(),
            _ => {
                self.output_line(&format!("Unknown command: {}", cmd));
                Ok(())
            }
        }
    }
    
    /// Output a line of text in the shell
    fn output_line(&mut self, text: &str) {
        // Scroll the screen up to make room for new output
        // TODO: Implement proper scrolling
        
        // For now, just clear and redraw everything
        self.clear_screen();
        self.display_welcome();
        
        // Add the new line
        vga_enhanced::write_at(self.window_height - 4, 2, text, 
                             Color::White, Color::Black);
        
        // Redraw the prompt and input
        self.redraw_input_line();
    }
    
    // Command implementations
    
    /// Display help information
    fn cmd_help(&mut self) -> Result<(), KernelError> {
        let help_text = concat!(
            "Available commands:\n",
            "  help       - Display this help message\n",
            "  echo [msg] - Display a message\n",
            "  ls [dir]   - List directory contents\n",
            "  cd [dir]   - Change directory\n",
            "  pwd        - Print working directory\n",
            "  cat [file] - Display file contents\n",
            "  clear/cls  - Clear the screen\n",
            "  touch [f]  - Create a new file\n",
            "  mkdir [d]  - Create a new directory\n",
            "  rm [path]  - Remove a file or directory\n",
            "  reboot     - Restart the system\n",
            "  version    - Display OS version\n"
        );
        
        self.output_line(help_text);
        Ok(())
    }
    
    /// Echo command arguments
    fn cmd_echo(&mut self, args: &[&str]) -> Result<(), KernelError> {
        self.output_line(&args.join(" "));
        Ok(())
    }
    
    /// List directory contents
    fn cmd_ls(&mut self, args: &[&str]) -> Result<(), KernelError> {
        let path = if args.is_empty() {
            self.current_dir.clone()
        } else {
            self.resolve_path(args[0])
        };
        
        let vfs = fs::vfs::get_vfs_manager().ok_or(KernelError::NotInitialized)?;
        let entries = vfs.read_dir(&path)?;
        
        if entries.is_empty() {
            self.output_line("Directory is empty.");
        } else {
            for entry in entries {
                let type_indicator = match entry.node_type {
                    fs::vfs::NodeType::Directory => "/",
                    fs::vfs::NodeType::File => "",
                    _ => "?",
                };
                self.output_line(&format!("{}{}", entry.name, type_indicator));
            }
        }
        
        Ok(())
    }
    
    /// Change current directory
    fn cmd_cd(&mut self, args: &[&str]) -> Result<(), KernelError> {
        if args.is_empty() {
            self.current_dir = "/".to_string();
            return Ok(());
        }
        
        let new_path = self.resolve_path(args[0]);
        
        // Verify that the directory exists
        let vfs = fs::vfs::get_vfs_manager().ok_or(KernelError::NotInitialized)?;
        let metadata = vfs.metadata(&new_path)?;
        
        if let fs::vfs::NodeType::Directory = metadata.node_type {
            self.current_dir = new_path;
            Ok(())
        } else {
            Err(KernelError::NotADirectory)
        }
    }
    
    /// Display file contents
    fn cmd_cat(&mut self, args: &[&str]) -> Result<(), KernelError> {
        if args.is_empty() {
            self.output_line("Usage: cat <file>");
            return Ok(());
        }
        
        let path = self.resolve_path(args[0]);
        
        // Read and display the file
        let mut buffer = [0u8; 256]; // Read in chunks
        let bytes_read = fs::direct_read_file(&path, &mut buffer)?;
        
        if bytes_read > 0 {
            let text = core::str::from_utf8(&buffer[0..bytes_read])
                .unwrap_or("(binary data)");
            self.output_line(text);
        } else {
            self.output_line("(empty file)");
        }
        
        Ok(())
    }
    
    /// Clear the screen
    fn cmd_clear(&mut self) -> Result<(), KernelError> {
        self.clear_screen();
        self.display_welcome();
        self.redraw_input_line();
        Ok(())
    }
    
    /// Print working directory
    fn cmd_pwd(&mut self) -> Result<(), KernelError> {
        let dir = self.current_dir.clone();
        self.output_line(&dir);
        Ok(())
    }
    
    /// Create a new file
    fn cmd_touch(&mut self, args: &[&str]) -> Result<(), KernelError> {
        if args.is_empty() {
            self.output_line("Usage: touch <filename>");
            return Ok(());
        }
        
        let path = self.resolve_path(args[0]);
        let vfs = fs::vfs::get_vfs_manager().ok_or(KernelError::NotInitialized)?;
        
        vfs.create_file(&path)?;
        self.output_line(&format!("Created file: {}", path));
        
        Ok(())
    }
    
    /// Create a new directory
    fn cmd_mkdir(&mut self, args: &[&str]) -> Result<(), KernelError> {
        if args.is_empty() {
            self.output_line("Usage: mkdir <directory>");
            return Ok(());
        }
        
        let path = self.resolve_path(args[0]);
        let vfs = fs::vfs::get_vfs_manager().ok_or(KernelError::NotInitialized)?;
        
        vfs.create_directory(&path)?;
        self.output_line(&format!("Created directory: {}", path));
        
        Ok(())
    }
    
    /// Remove a file or directory
    fn cmd_rm(&mut self, args: &[&str]) -> Result<(), KernelError> {
        if args.is_empty() {
            self.output_line("Usage: rm <path>");
            return Ok(());
        }
        
        let path = self.resolve_path(args[0]);
        let vfs = fs::vfs::get_vfs_manager().ok_or(KernelError::NotInitialized)?;
        
        vfs.remove(&path)?;
        self.output_line(&format!("Removed: {}", path));
        
        Ok(())
    }
    
    /// Reboot the system
    fn cmd_reboot(&mut self) -> Result<(), KernelError> {
        self.output_line("Rebooting...");
        
        // Wait a moment for the message to be seen
        for _ in 0..10_000_000 {
            // Spin to create visual delay
        }
        
        // Reboot using the 8042 keyboard controller
        unsafe {
            use x86_64::instructions::port::Port;
            let mut port = Port::new(0x64);
            port.write(0xFE as u8);
        }
        
        // Should not reach here, but just in case
        Ok(())
    }
    
    /// Display OS version information
    fn cmd_version(&mut self) -> Result<(), KernelError> {
        self.output_line("UniverseK OS v0.1.0");
        self.output_line("A minimal Unix-like OS for x86_64");
        Ok(())
    }
    
    /// Resolve a relative path to an absolute path
    fn resolve_path(&self, path: &str) -> String {
        if path.starts_with('/') {
            return path.to_string();
        }
        
        // Handle ".." to go up a directory
        if path == ".." {
            let parts: Vec<&str> = self.current_dir.split('/').collect();
            if parts.len() <= 2 {
                return "/".to_string();
            } else {
                return parts[..parts.len()-1].join("/");
            }
        }
        
        // Handle "." to stay in current directory
        if path == "." {
            return self.current_dir.clone();
        }
        
        // Handle relative path
        if self.current_dir.ends_with('/') {
            format!("{}{}", self.current_dir, path)
        } else {
            format!("{}/{}", self.current_dir, path)
        }
    }
}

/// Global shell instance
static mut SHELL: Option<Shell> = None;

/// Initialize the shell subsystem
pub fn init() -> Result<(), KernelError> {
    serial_println!("DEBUG: Initializing shell subsystem");
    
    // Create a new shell instance
    unsafe {
        serial_println!("DEBUG: Creating new Shell instance");
        SHELL = Some(Shell::new());
        
        if let Some(shell) = SHELL.as_mut() {
            serial_println!("DEBUG: Calling shell.init()");
            shell.init();
            serial_println!("DEBUG: Shell instance initialized successfully");
        } else {
            serial_println!("ERROR: Failed to access shell instance after creation");
            return Err(KernelError::InitializationFailed);
        }
    }
    
    serial_println!("DEBUG: Shell subsystem initialization complete");
    Ok(())
}

/// Get a reference to the shell, if initialized
pub fn get_shell() -> Option<&'static mut Shell> {
    unsafe { SHELL.as_mut() }
}

/// Run the shell (blocking)
pub fn run() -> Result<(), KernelError> {
    serial_println!("DEBUG: Starting shell main loop");
    
    // Get shell instance
    let shell = unsafe {
        match SHELL.as_mut() {
            Some(s) => {
                serial_println!("DEBUG: Got shell instance");
                s
            },
            None => {
                serial_println!("ERROR: Shell not initialized, cannot run");
                return Err(KernelError::NotInitialized);
            }
        }
    };
    
    // Draw initial screen
    serial_println!("DEBUG: Drawing initial shell screen");
    shell.clear_screen();
    shell.display_welcome();
    shell.draw_prompt();
    
    // Main shell loop
    serial_println!("DEBUG: Entering shell input loop");
    let mut loop_count = 0;
    loop {
        // Poll for keyboard input
        if let Some(key_event) = crate::drivers::ps2_keyboard::get_event() {
            serial_println!("DEBUG: Shell received key event: code={:?}, state={:?}", 
                key_event.code, key_event.state);
            
            if shell.handle_key(key_event) {
                // Exit code (ESC key pressed)
                serial_println!("DEBUG: Shell exit requested (ESC key)");
                break;
            }
        }
        
        // Output periodic heartbeat to show we're still running
        loop_count += 1;
        if loop_count % 10_000_000 == 0 {
            serial_println!("DEBUG: Shell heartbeat: {} iterations", loop_count / 10_000_000);
        }
        
        // Small delay to reduce CPU usage
        for _ in 0..100 {
            // Spin
        }
    }
    
    serial_println!("DEBUG: Shell exited normally");
    Ok(())
} 