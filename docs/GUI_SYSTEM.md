# UniverseK OS GUI System Documentation

This document explains the architecture and usage of the GUI system in UniverseK OS.

## Overview

The UniverseK OS GUI system provides a Windows-like graphical user interface with a desktop, taskbar, windows, and applications. It's implemented in the `kernel/src/gui` directory and consists of several key components:

- **Desktop**: Manages the desktop environment, icons, and taskbar
- **Windows**: Handles window creation, drawing, and management
- **Applications**: Defines the application framework and default applications
- **Events**: Processes keyboard and mouse input events

## Architecture

The GUI system follows a modular design with these main components:

```
kernel/src/gui/
├── mod.rs         # Main GUI module with initialization
├── desktop.rs     # Desktop environment
├── window.rs      # Window management
├── app.rs         # Application framework
└── events.rs      # Input event handling
```

### Initialization Flow

1. `gui::init()` in `mod.rs` initializes the GUI subsystem
2. `desktop::init()` initializes the desktop environment
3. `app::register_default_apps()` registers the default applications
4. `gui::run()` starts the main GUI loop

### Desktop Management

The desktop (`desktop.rs`) manages:

- The desktop background and visual elements
- The taskbar with START button
- Desktop icons for launching applications
- Open windows and their z-order
- Window focus and activation

### Window System

The window system (`window.rs`) provides:

- Window creation and destruction
- Window drawing with borders and title bars
- Input handling within windows
- Content display and scrolling

### Application Framework

The application system (`app.rs`) includes:

- An application registration system
- Desktop icon representation
- Window creation for applications
- Default applications (Terminal, About, Files, Settings)

### Event Handling

The event system (`events.rs`) manages:

- Mouse events (movement, clicks)
- Keyboard events (keystrokes, shortcuts)
- Event dispatching to the appropriate windows

## Core Components

### Desktop

The `Desktop` struct in `desktop.rs` is the central component managing the GUI:

```rust
pub struct Desktop {
    icons: Vec<AppIcon>,
    windows: Vec<WindowHandle>,
    active_window: Option<usize>,
    mouse_x: usize,
    mouse_y: usize,
    start_menu_open: bool,
    taskbar_height: usize,
    exit_requested: bool,
}
```

Key methods:

- `add_icon()`: Adds an application icon to the desktop
- `add_window()`: Adds a window to the desktop
- `handle_mouse_click()`: Processes mouse clicks
- `draw()`: Draws the entire desktop

### Window

The `Window` struct in `window.rs` represents a window:

```rust
pub struct Window {
    title: String,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    content: String,
    input_buffer: String,
    input_callback: Option<InputCallback>,
    accepts_input: bool,
}
```

Key methods:

- `draw()`: Draws the window
- `add_text()`: Adds text to the window content
- `handle_key()`: Processes keyboard input
- `handle_click()`: Processes mouse clicks

### Application Icon

The `AppIcon` struct in `app.rs` represents an application:

```rust
pub struct AppIcon {
    pub name: String,
    pub create_fn: Option<AppCreateFn>,
}
```

Each application provides a creation function that returns a `WindowHandle`.

## VGA Interface

The GUI system relies on the enhanced VGA driver (`drivers/vga_enhanced.rs`) which provides:

- Text mode with customizable colors
- Character writing at specific positions
- Box drawing and UI elements
- Reading characters from screen positions for mouse interactions

## Extending the GUI System

### Creating a New Application

To create a new application:

1. Add a new function in `app.rs`:

```rust
fn create_calculator_app() -> Result<WindowHandle, KernelError> {
    let window_handle = create_window("Calculator", 20, 10, 30, 20);
    let window_handle_for_closure = window_handle.clone();

    {
        let mut window = window_handle.lock();
        window.add_text("Calculator App\n\n");
        window.add_text("Enter calculation:\n");

        // Set up input handling
        window.enable_input(Box::new(move |input| {
            let mut window = window_handle_for_closure.lock();
            // Parse and calculate input
            let result = calculate_expression(input);
            window.add_text(&format!("Result: {}\n", result));
            Ok(())
        }));
    }

    Ok(window_handle)
}

// Helper function for calculator
fn calculate_expression(expr: &str) -> i32 {
    // Simplified example - actual implementation would be more complex
    let parts: Vec<&str> = expr.split('+').collect();
    if parts.len() == 2 {
        if let (Ok(a), Ok(b)) = (parts[0].trim().parse::<i32>(), parts[1].trim().parse::<i32>()) {
            return a + b;
        }
    }
    0 // Default for invalid input
}
```

2. Register the application in `register_default_apps()`:

```rust
pub fn register_default_apps() -> Result<(), KernelError> {
    // Existing apps...
    desktop::add_icon(AppIcon::new("Calculator", Box::new(create_calculator_app)))?;
    Ok(())
}
```

### Adding a Window Control

To add a custom control to a window:

```rust
// In window.rs, add a new method:
impl Window {
    pub fn add_button(&mut self, label: &str, row: usize, col: usize) -> usize {
        let button_id = self.next_control_id;
        self.next_control_id += 1;

        // Store button info
        self.controls.push(Control {
            id: button_id,
            kind: ControlKind::Button,
            label: label.to_string(),
            row,
            col,
            width: label.len() + 4,
            height: 3,
        });

        // Return button ID for callback registration
        button_id
    }
}
```

### Customizing the Desktop

To customize the desktop appearance:

```rust
// In desktop.rs, modify the draw_desktop function

fn draw_desktop() -> Result<(), KernelError> {
    // Set custom background color
    for y in 0..23 {  // Leave space for taskbar
        for x in 0..80 {
            vga_enhanced::write_at(y, x, " ", Color::White, Color::DarkBlue);
        }
    }

    // Draw custom desktop elements
    vga_enhanced::write_at(1, 2, "Welcome to UniverseK OS", Color::Yellow, Color::DarkBlue);

    // Draw a system information box
    vga_enhanced::draw_box(60, 1, 18, 6);
    vga_enhanced::write_at(2, 62, "System Info", Color::White, Color::DarkBlue);
    vga_enhanced::write_at(3, 62, "CPU: x86_64", Color::LightGray, Color::DarkBlue);
    vga_enhanced::write_at(4, 62, "Memory: 64MB", Color::LightGray, Color::DarkBlue);

    Ok(())
}
```

## Advanced Topics

### Input Handling Best Practices

When dealing with input in windows:

1. Always clone the window handle for use in closures:

```rust
let window_handle_for_closure = window_handle.clone();
```

2. Use appropriate borrowing in callbacks:

```rust
window.enable_input(Box::new(move |input| {
    let mut window = window_handle_for_closure.lock();
    // Process input...
    Ok(())
}));
```

3. Avoid borrow checker issues by releasing locks before callbacks:

```rust
// Add input to buffer first
window.add_text(&format!("> {}\n", input));
// Clear buffer before callback
window.input_buffer.clear();
// Now call the callback
callback(&input)?;
```

### Window Management

For custom window management:

1. Moving windows:

```rust
pub fn move_window(&mut self, dx: i32, dy: i32) {
    // Calculate new position with bounds checking
    let new_x = (self.x as i32 + dx).max(0) as usize;
    let new_y = (self.y as i32 + dy).max(0) as usize;

    // Update position
    self.x = new_x;
    self.y = new_y;
}
```

2. Implementing window minimizing:

```rust
pub fn minimize_window(&mut self, window_idx: usize) {
    if let Some(window) = self.windows.get(window_idx) {
        let mut window = window.lock();
        window.is_minimized = true;
    }
}
```

## Known Limitations

The current GUI system has some limitations to be aware of:

1. **Fixed Resolution**: Limited to 80x25 text mode resolution.
2. **No Graphics Mode**: Text-based UI only, no pixel-level graphics.
3. **Limited Controls**: Basic windows and text, without complex controls.
4. **No Drag-and-Drop**: Windows can't be moved by dragging.
5. **Input Focus**: Limited focus management between windows.

## Future Improvements

Planned improvements to the GUI system:

1. **Window Dragging**: Allow moving windows by dragging their title bars
2. **Dialog Boxes**: Add support for modal dialog boxes
3. **More Controls**: Add buttons, checkboxes, radio buttons, etc.
4. **Custom Themes**: Support for different color themes
5. **Graphics Mode**: Support for higher-resolution graphics mode

## Troubleshooting

### Common Problems

- **Window not rendering properly**: Check if the window's dimensions exceed the screen boundaries.
- **Input not working**: Ensure the window has `accepts_input` set to true and has an input callback registered.
- **Mouse clicks not detected**: Verify the mouse position calculation and event propagation.

### Debugging Tips

1. Use `serial_println!()` to debug GUI rendering and event handling.
2. Add visual indicators for mouse position and clicked areas.
3. Implement a temporary debug overlay to show window boundaries.

## Reference

### Color Constants

The GUI system uses these color constants from `vga_enhanced.rs`:

```rust
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
```

### Desktop Colors

```rust
pub const DESKTOP_BACKGROUND: Color = Color::Blue;
pub const DESKTOP_TEXT: Color = Color::White;
pub const TASKBAR_BACKGROUND: Color = Color::LightGray;
pub const TASKBAR_TEXT: Color = Color::Black;
pub const ICON_BACKGROUND: Color = Color::Cyan;
pub const ICON_TEXT: Color = Color::Black;
```

### Window Colors

```rust
pub const WINDOW_TITLE_ACTIVE: Color = Color::Blue;
pub const WINDOW_TITLE_INACTIVE: Color = Color::DarkGray;
pub const WINDOW_TEXT: Color = Color::White;
pub const WINDOW_BACKGROUND: Color = Color::LightGray;
pub const WINDOW_BORDER: Color = Color::White;
```
