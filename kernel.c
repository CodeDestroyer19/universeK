#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include "kernel.h"
#include "idt.h"
#include "pic.h"
#include "cursor.h"
#include "src/memory/memory.h"
#include "filesystem.h"
#include "src/libc/string.h"
#include "src/drivers/driver.h"
#include "src/drivers/mouse.h"
#include "src/interrupts/interrupt.h"
#include "debug/debug.h"
#include "src/drivers/vga/vga.h"
#include "window.h"
#include "io/io.h"
#include "drivers/keyboard/keyboard.h"
#include "terminal.h"
#include "interrupts/irq.h"
#include "src/interrupts/timer.h"

#define PORT 0x3f8   /* COM1 */

// Remove old serial port functions and replace with new ones
void init_serial() {
    port_write_byte(PORT + 1, 0x00);    // Disable all interrupts
    port_write_byte(PORT + 3, 0x80);    // Enable DLAB (set baud rate divisor)
    port_write_byte(PORT + 0, 0x03);    // Set divisor to 3 (lo byte) 38400 baud
    port_write_byte(PORT + 1, 0x00);    //                  (hi byte)
    port_write_byte(PORT + 3, 0x03);    // 8 bits, no parity, one stop bit
    port_write_byte(PORT + 2, 0xC7);    // Enable FIFO, clear them, with 14-byte threshold
    port_write_byte(PORT + 4, 0x0B);    // IRQs enabled, RTS/DSR set
}

int is_transmit_empty() {
    return port_read_byte(PORT + 5) & 0x20;
}

void write_serial(char a) {
    while (is_transmit_empty() == 0);
    port_write_byte(PORT, a);
}

void write_serial_string(const char* str) {
    for (size_t i = 0; str[i] != '\0'; i++) {
        write_serial(str[i]);
    }
}

// VGA text mode color constants
enum vga_color {
    VGA_COLOR_BLACK = 0,
    VGA_COLOR_BLUE = 1,
    VGA_COLOR_GREEN = 2,
    VGA_COLOR_CYAN = 3,
    VGA_COLOR_RED = 4,
    VGA_COLOR_MAGENTA = 5,
    VGA_COLOR_BROWN = 6,
    VGA_COLOR_LIGHT_GREY = 7,
    VGA_COLOR_DARK_GREY = 8,
    VGA_COLOR_LIGHT_BLUE = 9,
    VGA_COLOR_LIGHT_GREEN = 10,
    VGA_COLOR_LIGHT_CYAN = 11,
    VGA_COLOR_LIGHT_RED = 12,
    VGA_COLOR_LIGHT_MAGENTA = 13,
    VGA_COLOR_LIGHT_BROWN = 14,
    VGA_COLOR_WHITE = 15,
};

static inline uint8_t vga_entry_color(enum vga_color fg, enum vga_color bg) {
    return fg | bg << 4;
}

static inline uint16_t vga_entry(unsigned char uc, uint8_t color) {
    return (uint16_t) uc | (uint16_t) color << 8;
}

// Text mode dimensions
#define TEXT_MODE_WIDTH 80
#define TEXT_MODE_HEIGHT 25

// Terminal dimensions
#define TERM_WIDTH 80
#define TERM_HEIGHT 25
#define TERM_BUFFER_SIZE (TERM_WIDTH * TERM_HEIGHT)

// Global variables for text mode
static char term_buffer[TERM_BUFFER_SIZE];

// GUI windows
static Window* terminal_window = NULL;
static Window* info_window = NULL;

size_t terminal_row;
size_t terminal_column;
uint8_t terminal_color;
uint16_t* terminal_buffer;

size_t get_terminal_row(void) {
    return terminal_row;
}

size_t get_terminal_column(void) {
    return terminal_column;
}

void terminal_initialize(void) {
    terminal_row = 0;
    terminal_column = 0;
    terminal_color = vga_entry_color(VGA_COLOR_LIGHT_GREY, VGA_COLOR_BLACK);
    terminal_buffer = (uint16_t*) 0xB8000;
    for (size_t y = 0; y < TEXT_MODE_HEIGHT; y++) {
        for (size_t x = 0; x < TEXT_MODE_WIDTH; x++) {
            const size_t index = y * TEXT_MODE_WIDTH + x;
            terminal_buffer[index] = vga_entry(' ', terminal_color);
        }
    }
    update_cursor(0, 0, TEXT_MODE_WIDTH);
}

void terminal_setcolor(uint8_t color) {
    terminal_color = color;
}

void terminal_putchar(char c) {
    if (c == '\n') {
        terminal_column = 0;
        if (++terminal_row == TEXT_MODE_HEIGHT) {
            // Scroll the screen
            for (size_t y = 0; y < TEXT_MODE_HEIGHT - 1; y++) {
                for (size_t x = 0; x < TEXT_MODE_WIDTH; x++) {
                    terminal_buffer[y * TEXT_MODE_WIDTH + x] = terminal_buffer[(y + 1) * TEXT_MODE_WIDTH + x];
                }
            }
            // Clear the last line
            for (size_t x = 0; x < TEXT_MODE_WIDTH; x++) {
                terminal_buffer[(TEXT_MODE_HEIGHT - 1) * TEXT_MODE_WIDTH + x] = vga_entry(' ', terminal_color);
            }
            terminal_row = TEXT_MODE_HEIGHT - 1;
        }
        update_cursor(terminal_column, terminal_row, TEXT_MODE_WIDTH);
        return;
    }
    
    if (c == '\b') {
        if (terminal_column > 0) {
            terminal_column--;
        } else if (terminal_row > 0) {
            terminal_row--;
            terminal_column = TEXT_MODE_WIDTH - 1;
        }
        const size_t index = terminal_row * TEXT_MODE_WIDTH + terminal_column;
        terminal_buffer[index] = vga_entry(' ', terminal_color);
        update_cursor(terminal_column, terminal_row, TEXT_MODE_WIDTH);
        return;
    }
    
    const size_t index = terminal_row * TEXT_MODE_WIDTH + terminal_column;
    terminal_buffer[index] = vga_entry(c, terminal_color);
    if (++terminal_column == TEXT_MODE_WIDTH) {
        terminal_column = 0;
        if (++terminal_row == TEXT_MODE_HEIGHT) {
            // Scroll the screen
            for (size_t y = 0; y < TEXT_MODE_HEIGHT - 1; y++) {
                for (size_t x = 0; x < TEXT_MODE_WIDTH; x++) {
                    terminal_buffer[y * TEXT_MODE_WIDTH + x] = terminal_buffer[(y + 1) * TEXT_MODE_WIDTH + x];
                }
            }
            // Clear the last line
            for (size_t x = 0; x < TEXT_MODE_WIDTH; x++) {
                terminal_buffer[(TEXT_MODE_HEIGHT - 1) * TEXT_MODE_WIDTH + x] = vga_entry(' ', terminal_color);
            }
            terminal_row = TEXT_MODE_HEIGHT - 1;
        }
    }
    update_cursor(terminal_column, terminal_row, TEXT_MODE_WIDTH);
}

void terminal_write(const char* data, size_t size) {
    for (size_t i = 0; i < size; i++)
        terminal_putchar(data[i]);
}

void terminal_writestring(const char* data) {
    for (size_t i = 0; data[i] != '\0'; i++)
        terminal_putchar(data[i]);
}

void handle_command(const char* cmd) {
    if (strcmp(cmd, "help") == 0) {
        terminal_writestring("Available commands:\n");
        terminal_writestring("  help     - Show this help message\n");
        terminal_writestring("  clear    - Clear the screen\n");
        terminal_writestring("  about    - About UniverseK OS\n");
        terminal_writestring("  ls       - List files\n");
        terminal_writestring("  touch    - Create a new file\n");
        terminal_writestring("  rm       - Delete a file\n");
        terminal_writestring("  write    - Write text to a file\n");
        terminal_writestring("  cat      - Display file contents\n");
        terminal_writestring("  meminfo  - Display memory information\n");
        terminal_writestring("  drivers  - List installed drivers\n");
        terminal_writestring("  mouse    - Show mouse position\n");
    }
    else if (strcmp(cmd, "clear") == 0) {
        terminal_initialize();
    }
    else if (strcmp(cmd, "about") == 0) {
        terminal_writestring("UniverseK - A simple operating system\n");
        terminal_writestring("Version 0.2.0\n");
        terminal_writestring("Features: Memory Management, Simple Filesystem\n");
    }
    else if (strcmp(cmd, "ls") == 0) {
        fs_list();
    }
    else if (strncmp(cmd, "touch ", 6) == 0) {
        const char* filename = cmd + 6;
        if (fs_create(filename) >= 0) {
            terminal_writestring("File created: ");
            terminal_writestring(filename);
            terminal_writestring("\n");
        } else {
            terminal_writestring("Error: Could not create file\n");
        }
    }
    else if (strncmp(cmd, "rm ", 3) == 0) {
        const char* filename = cmd + 3;
        int found = 0;
        
        // Find and delete the file by name
        for (int i = 0; i < MAX_FILES; i++) {
            // We'll need to add a way to get file info
            if (fs_delete(i) == 0) {
                found = 1;
                terminal_writestring("File deleted: ");
                terminal_writestring(filename);
                terminal_writestring("\n");
                break;
            }
        }
        
        if (!found) {
            terminal_writestring("Error: File not found: ");
            terminal_writestring(filename);
            terminal_writestring("\n");
        }
    }
    else if (strncmp(cmd, "write ", 6) == 0) {
        char* space = strchr(cmd + 6, ' ');
        if (space) {
            *space = '\0';
            const char* filename = cmd + 6;
            const char* content = space + 1;
            int found = 0;
            
            // Find file by name and write content
            for (int i = 0; i < MAX_FILES; i++) {
                // We'll need to add a way to check file names
                if (fs_write(i, (const uint8_t*)content, strlen(content)) >= 0) {
                    found = 1;
                    terminal_writestring("Content written to file: ");
                    terminal_writestring(filename);
                    terminal_writestring("\n");
                    break;
                }
            }
            
            if (!found) {
                terminal_writestring("Error: Could not write to file: ");
                terminal_writestring(filename);
                terminal_writestring("\n");
            }
        } else {
            terminal_writestring("Usage: write <filename> <content>\n");
        }
    }
    else if (strncmp(cmd, "cat ", 4) == 0) {
        const char* filename = cmd + 4;
        uint8_t buffer[4096];
        int found = 0;
        
        // Find and read file by name
        for (int i = 0; i < MAX_FILES; i++) {
            // We'll need to add a way to check file names
            int bytes = fs_read(i, buffer, sizeof(buffer));
            if (bytes > 0) {
                buffer[bytes] = '\0';
                terminal_writestring("Contents of ");
                terminal_writestring(filename);
                terminal_writestring(":\n");
                terminal_writestring((const char*)buffer);
                terminal_writestring("\n");
                found = 1;
                break;
            }
        }
        
        if (!found) {
            terminal_writestring("Error: File not found: ");
            terminal_writestring(filename);
            terminal_writestring("\n");
        }
    }
    else if (strcmp(cmd, "meminfo") == 0) {
        terminal_writestring("Memory Information:\n");
        terminal_writestring("Heap Start: 0x400000\n");
        terminal_writestring("Heap Size: 4MB\n");
        // You could add more detailed memory statistics here
    }
    else if (strcmp(cmd, "drivers") == 0) {
        list_drivers();
    }
    else if (strcmp(cmd, "mouse") == 0) {
        int x, y;
        get_mouse_position(&x, &y);
        terminal_writestring("Mouse position: (");
        
        // Convert x to string
        char x_str[8];
        int x_pos = 0;
        int x_val = x;
        do {
            x_str[x_pos++] = '0' + (x_val % 10);
            x_val /= 10;
        } while (x_val > 0);
        // Reverse the string
        for (int i = 0; i < x_pos / 2; i++) {
            char temp = x_str[i];
            x_str[i] = x_str[x_pos - 1 - i];
            x_str[x_pos - 1 - i] = temp;
        }
        x_str[x_pos] = '\0';
        
        terminal_writestring(x_str);
        terminal_writestring(", ");
        
        // Convert y to string
        char y_str[8];
        int y_pos = 0;
        int y_val = y;
        do {
            y_str[y_pos++] = '0' + (y_val % 10);
            y_val /= 10;
        } while (y_val > 0);
        // Reverse the string
        for (int i = 0; i < y_pos / 2; i++) {
            char temp = y_str[i];
            y_str[i] = y_str[y_pos - 1 - i];
            y_str[y_pos - 1 - i] = temp;
        }
        y_str[y_pos] = '\0';
        
        terminal_writestring(y_str);
        terminal_writestring(")\n");
    }
    else if (cmd[0] != '\0') {
        terminal_writestring("Unknown command: ");
        terminal_writestring(cmd);
        terminal_writestring("\nType 'help' for available commands\n");
    }
}

// Keyboard event handler
static void keyboard_handler(keyboard_event_t* event) {
    if (!event->pressed) return; // Only handle key presses
    
    // Get the focused window
    Window* focused = get_focused_window();
    if (focused && focused->type == WINDOW_TYPE_TERMINAL) {
        // Add character to terminal buffer
        if (event->key) {
            terminal_input_char(focused, event->key);
        }
    }
}

void terminal_draw_handler(Window* win) {
    (void)win; // Suppress unused parameter warning
    // Draw terminal contents from term_buffer
    for (size_t i = 0; i < TERM_BUFFER_SIZE && term_buffer[i]; i++) {
        window_putchar(win, (i % TERM_WIDTH) * 8, (i / TERM_WIDTH) * 9, term_buffer[i], VGA_WHITE);
    }
}

// Info window handler
void info_draw_handler(Window* win) {
    int y = 5;
    window_draw_text(win, 5, y, "System Information:", VGA_WHITE);
    y += 18;
    
    window_draw_text(win, 5, y, "Memory:", VGA_LIGHT_GRAY);
    y += 9;
    window_draw_text(win, 15, y, "Total: 4MB", VGA_WHITE);
    y += 9;
    
    window_draw_text(win, 5, y, "Filesystem:", VGA_LIGHT_GRAY);
    y += 9;
    window_draw_text(win, 15, y, "Max files: 256", VGA_WHITE);
    y += 9;
    window_draw_text(win, 15, y, "File size: 4KB", VGA_WHITE);
    y += 18;
    
    window_draw_text(win, 5, y, "Input devices:", VGA_LIGHT_GRAY);
    y += 9;
    window_draw_text(win, 15, y, "Keyboard: PS/2", VGA_WHITE);
    y += 9;
    window_draw_text(win, 15, y, "Mouse: PS/2", VGA_WHITE);
}

void kernel_main(void) {
    // Initialize serial port first for debugging
    init_serial();
    write_serial_string("Serial port initialized\n");
    
    // Initialize debug system
    debug_init();
    DEBUG_INFO("KERNEL", "Debug system initialized");
    
    // Initialize memory management
    DEBUG_INFO("KERNEL", "Initializing memory management");
    memory_init();
    DEBUG_INFO("KERNEL", "Memory management initialized");
    
    // Initialize filesystem
    DEBUG_INFO("KERNEL", "Initializing filesystem");
    fs_init();
    DEBUG_INFO("KERNEL", "Filesystem initialized");
    
    // Initialize IDT
    DEBUG_INFO("KERNEL", "Initializing IDT");
    idt_install();
    DEBUG_INFO("KERNEL", "IDT initialized");
    
    // Initialize IRQ system
    DEBUG_INFO("KERNEL", "Initializing IRQ system");
    irq_init();
    DEBUG_INFO("KERNEL", "IRQ system initialized");
    
    // Initialize PIC
    DEBUG_INFO("KERNEL", "Initializing PIC");
    pic_init();
    DEBUG_INFO("KERNEL", "PIC initialized");
    
    // Initialize timer
    DEBUG_INFO("KERNEL", "Initializing system timer");
    timer_init();
    DEBUG_INFO("KERNEL", "System timer initialized");
    
    // Initialize keyboard
    if (keyboard_init() != STATUS_SUCCESS) {
        DEBUG_ERROR("KERNEL", "Failed to initialize keyboard");
        return;
    }
    
    // Register keyboard handler
    if (keyboard_register_handler(keyboard_handler) != STATUS_SUCCESS) {
        DEBUG_ERROR("KERNEL", "Failed to register keyboard handler");
        return;
    }
    
    DEBUG_INFO("KERNEL", "System initialization complete");
    
    // Initialize mouse
    DEBUG_INFO("KERNEL", "Initializing mouse");
    init_mouse();
    DEBUG_INFO("KERNEL", "Mouse initialized");
    
    // Initialize VGA graphics
    DEBUG_INFO("KERNEL", "Initializing VGA graphics");
    vga_init();
    DEBUG_INFO("KERNEL", "VGA graphics initialized");
    
    // Initialize window system
    DEBUG_INFO("KERNEL", "Initializing window system");
    window_init();
    DEBUG_INFO("KERNEL", "Window system initialized");
    
    // Create info window
    info_window = window_create(320, 30, 200, 150, "System Info");
    if (info_window) {
        info_window->on_draw = info_draw_handler;
        window_clear(info_window, VGA_BLACK);
    }

    // Create terminal window
    terminal_window = window_create(10, 30, 300, 150, "Terminal");
    if (terminal_window) {
        terminal_window->type = WINDOW_TYPE_TERMINAL;
        terminal_init(terminal_window);
        window_clear(terminal_window, VGA_BLACK);
        window_draw_text(terminal_window, 0, 0, "Welcome to UniverseK!\nType 'help' for available commands.\n\n> ", VGA_LIGHT_GRAY);
        window_focus(terminal_window);
    }
    
    // Enable interrupts
    DEBUG_INFO("KERNEL", "Enabling interrupts");
    asm volatile("sti");
    DEBUG_INFO("KERNEL", "Interrupts enabled");
    
    // Main loop
    while (1) {
        // Update display
        window_draw_all();
        
        // Halt CPU until next interrupt
        asm volatile("hlt");
    }
} 