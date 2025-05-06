#include <stdint.h>
#include "kernel.h"
//#include "cursor.h"  // Not needed for GUI
#include "interrupt.h"
#include "driver.h"
#include "window.h"
#include "debug.h"

#define KEYBOARD_DATA_PORT 0x60
#define KEYBOARD_STATUS_PORT 0x64
#define KEYBOARD_COMMAND_PORT 0x64
// #define BUFFER_SIZE 256  // Not used in GUI

// Keyboard controller commands
#define KEYBOARD_CMD_ENABLE 0xAE
#define KEYBOARD_CMD_RESET 0xFF
#define KEYBOARD_CMD_SET_DEFAULTS 0xF6
#define KEYBOARD_CMD_ENABLE_SCANNING 0xF4

// Add serial debugging function declaration
extern void write_serial_string(const char* str);

// static char input_buffer[BUFFER_SIZE];
// static size_t buffer_pos = 0;

// Basic US keyboard layout
unsigned char keyboard_map[128] = {
    0,  27, '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\b',
    '\t', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n',
    0, 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`',
    0, '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', 0,
    '*', 0, ' ', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
};

// Wait for keyboard controller input buffer to be empty
static void keyboard_wait_input(void) {
    uint32_t timeout = 100000;
    while (timeout-- && (inb(KEYBOARD_STATUS_PORT) & 0x02)) {
        io_wait();
    }
    if (timeout == 0) {
        DEBUG_ERROR("KEYBOARD", "Timeout waiting for input buffer");
    }
}

// Wait for keyboard controller output buffer to be full
static void keyboard_wait_output(void) {
    uint32_t timeout = 100000;
    while (timeout-- && !(inb(KEYBOARD_STATUS_PORT) & 0x01)) {
        io_wait();
    }
    if (timeout == 0) {
        DEBUG_ERROR("KEYBOARD", "Timeout waiting for output buffer");
    }
}

// Send command to keyboard controller
static void keyboard_send_cmd(uint8_t cmd) {
    keyboard_wait_input();
    outb(KEYBOARD_COMMAND_PORT, cmd);
    io_wait();
}

// Send data to keyboard
static void keyboard_send_data(uint8_t data) {
    keyboard_wait_input();
    outb(KEYBOARD_DATA_PORT, data);
    io_wait();
}

// Read keyboard response
static uint8_t keyboard_read_data(void) {
    keyboard_wait_output();
    io_wait();
    return inb(KEYBOARD_DATA_PORT);
}

// Keyboard IRQ handler forwards key to windowing system
static void keyboard_irq_handler(struct regs* r) {
    (void)r;
    uint8_t scancode = inb(KEYBOARD_DATA_PORT);
    
    // Debug: log scancode
    DEBUG_INFO_HEX("KEYBOARD", "Scancode received", scancode);
    
    if (scancode < 128) {
        char c = keyboard_map[scancode];
        if (c) {
            DEBUG_INFO_HEX("KEYBOARD", "Mapped to character", c);
            window_handle_key(c);
        }
    }
}

void init_keyboard(void) {
    DEBUG_INFO("KEYBOARD", "Starting keyboard initialization");
    
    // Disable keyboard first
    keyboard_send_cmd(0xAD);
    io_wait();
    
    // Flush output buffer
    while (inb(KEYBOARD_STATUS_PORT) & 0x01) {
        inb(KEYBOARD_DATA_PORT);
        io_wait();
    }
    
    // Reset keyboard
    DEBUG_INFO("KEYBOARD", "Resetting keyboard");
    keyboard_send_data(KEYBOARD_CMD_RESET);
    if (keyboard_read_data() != 0xFA) {
        DEBUG_ERROR("KEYBOARD", "Keyboard reset failed - no ACK");
        return;
    }
    if (keyboard_read_data() != 0xAA) {
        DEBUG_ERROR("KEYBOARD", "Keyboard reset failed - self test failed");
        return;
    }
    
    // Set default configuration
    DEBUG_INFO("KEYBOARD", "Setting keyboard defaults");
    keyboard_send_data(KEYBOARD_CMD_SET_DEFAULTS);
    if (keyboard_read_data() != 0xFA) {
        DEBUG_ERROR("KEYBOARD", "Failed to set keyboard defaults");
        return;
    }
    
    // Enable scanning
    DEBUG_INFO("KEYBOARD", "Enabling keyboard scanning");
    keyboard_send_data(KEYBOARD_CMD_ENABLE_SCANNING);
    if (keyboard_read_data() != 0xFA) {
        DEBUG_ERROR("KEYBOARD", "Failed to enable keyboard scanning");
        return;
    }
    
    // Re-enable keyboard
    keyboard_send_cmd(0xAE);
    io_wait();
    
    // Register keyboard IRQ handler
    DEBUG_INFO("KEYBOARD", "Installing keyboard IRQ handler");
    irq_install_handler(1, keyboard_irq_handler);
    
    // Register keyboard driver
    static struct driver keyboard_driver = {
        .name = "ps2_keyboard",
        .type = DRIVER_TYPE_CHAR,
        .init = NULL,
        .read = NULL,
        .write = NULL,
        .ioctl = NULL,
        .cleanup = NULL
    };
    register_driver(&keyboard_driver);
    
    DEBUG_INFO("KEYBOARD", "Keyboard initialization complete");
}