#include "drivers/keyboard/keyboard.h"
#include "io/io.h"
#include "debug/debug.h"
#include "interrupts/interrupt.h"
#include "interrupts/pic.h"
#include "interrupts/irq.h"

// Keyboard ports
#define KEYBOARD_DATA    0x60
#define KEYBOARD_STATUS  0x64
#define KEYBOARD_CMD    0x64

// Keyboard commands
#define KEYBOARD_CMD_RESET           0xFF
#define KEYBOARD_CMD_ENABLE         0xF4
#define KEYBOARD_CMD_SET_DEFAULTS   0xF6
#define KEYBOARD_CMD_DISABLE        0xF5
#define KEYBOARD_CMD_SET_LEDS       0xED

// Keyboard responses
#define KEYBOARD_RES_ACK           0xFA
#define KEYBOARD_RES_RESEND       0xFE
#define KEYBOARD_RES_ERROR        0xFC
#define KEYBOARD_RES_SELF_TEST_OK 0xAA

// Keyboard status bits
#define KEYBOARD_STATUS_OUTPUT_FULL  0x01
#define KEYBOARD_STATUS_INPUT_FULL   0x02

// Maximum number of keyboard event handlers
#define MAX_KEYBOARD_HANDLERS 8

// Keyboard event handler type
typedef void (*keyboard_handler_t)(keyboard_event_t* event);

// Keyboard state
static struct {
    bool num_lock;
    bool caps_lock;
    bool scroll_lock;
    bool shift;
    bool ctrl;
    bool alt;
    keyboard_handler_t handlers[MAX_KEYBOARD_HANDLERS];
    int num_handlers;
} keyboard_state = {0};

// US keyboard layout
static const char keyboard_map[128] = {
    0,  27, '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\b',
    '\t', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n',
    0, 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`',
    0, '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', 0,
    '*', 0, ' ', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
};

// Wait for keyboard controller input buffer to be empty
static bool keyboard_wait_input(uint32_t timeout) {
    while (timeout--) {
        if (!(port_read_byte(KEYBOARD_STATUS) & KEYBOARD_STATUS_INPUT_FULL)) {
            return true;
        }
        io_wait();
    }
    DEBUG_ERROR("KB", "Timeout waiting for input buffer");
    return false;
}

// Wait for keyboard controller output buffer to be full
static bool keyboard_wait_output(uint32_t timeout) {
    while (timeout--) {
        if (port_read_byte(KEYBOARD_STATUS) & KEYBOARD_STATUS_OUTPUT_FULL) {
            return true;
        }
        io_wait();
    }
    DEBUG_ERROR("KB", "Timeout waiting for output buffer");
    return false;
}

// Mark keyboard_send_cmd as used to avoid warning
static bool keyboard_send_cmd(uint8_t cmd) __attribute__((used));
static bool keyboard_send_cmd(uint8_t cmd) {
    if (!keyboard_wait_input(1000)) {
        return false;
    }
    port_write_byte(KEYBOARD_CMD, cmd);
    io_wait();
    return true;
}

// Send data to keyboard
static bool keyboard_send_data(uint8_t data) {
    if (!keyboard_wait_input(1000)) {
        return false;
    }
    port_write_byte(KEYBOARD_DATA, data);
    io_wait();
    return true;
}

// Read data from keyboard
static int keyboard_read_data(void) {
    if (!keyboard_wait_output(1000)) {
        return -1;
    }
    io_wait();
    return port_read_byte(KEYBOARD_DATA);
}

// Update keyboard LEDs
static void keyboard_update_leds(void) {
    uint8_t leds = 0;
    if (keyboard_state.num_lock) leds |= 2;
    if (keyboard_state.caps_lock) leds |= 4;
    if (keyboard_state.scroll_lock) leds |= 1;
    
    keyboard_send_data(KEYBOARD_CMD_SET_LEDS);
    keyboard_read_data();  // Read ACK
    keyboard_send_data(leds);
    keyboard_read_data();  // Read ACK
}

// Process keyboard scancode
static void keyboard_process_scancode(uint8_t scancode) {
    keyboard_event_t event = {0};
    event.scancode = scancode;
    
    // Key release
    if (scancode & 0x80) {
        event.pressed = false;
        scancode &= 0x7F;
        
        switch (scancode) {
            case 0x2A: case 0x36: keyboard_state.shift = false; break;
            case 0x1D: keyboard_state.ctrl = false; break;
            case 0x38: keyboard_state.alt = false; break;
        }
    }
    // Key press
    else {
        event.pressed = true;
        
        switch (scancode) {
            case 0x2A: case 0x36: keyboard_state.shift = true; break;
            case 0x1D: keyboard_state.ctrl = true; break;
            case 0x38: keyboard_state.alt = true; break;
            case 0x45: keyboard_state.num_lock = !keyboard_state.num_lock; keyboard_update_leds(); break;
            case 0x3A: keyboard_state.caps_lock = !keyboard_state.caps_lock; keyboard_update_leds(); break;
            case 0x46: keyboard_state.scroll_lock = !keyboard_state.scroll_lock; keyboard_update_leds(); break;
        }
    }
    
    // Set modifiers
    event.shift = keyboard_state.shift;
    event.ctrl = keyboard_state.ctrl;
    event.alt = keyboard_state.alt;
    event.caps_lock = keyboard_state.caps_lock;
    event.num_lock = keyboard_state.num_lock;
    event.scroll_lock = keyboard_state.scroll_lock;
    
    // Map to ASCII
    if (scancode < 128) {
        event.key = keyboard_map[scancode];
        if (event.shift) {
            if (event.key >= 'a' && event.key <= 'z') {
                event.key -= 32;
            }
            // TODO: Add more shift mappings
        }
    }
    
    // Call handlers
    for (int i = 0; i < keyboard_state.num_handlers; i++) {
        if (keyboard_state.handlers[i]) {
            keyboard_state.handlers[i](&event);
        }
    }
}

// Keyboard interrupt handler
static void keyboard_interrupt(struct interrupt_context* context) {
    (void)context;
    
    // Read scancode
    uint8_t scancode = port_read_byte(KEYBOARD_DATA);
    DEBUG_TRACE("KB", "Scancode: 0x%02x", scancode);
    
    // Process scancode
    keyboard_process_scancode(scancode);
    
    // Send EOI
    pic_send_eoi(1);
}

status_t keyboard_init(void) {
    DEBUG_INFO("KB", "Initializing keyboard");
    
    // Reset keyboard state
    keyboard_state.num_handlers = 0;
    keyboard_state.num_lock = false;
    keyboard_state.caps_lock = false;
    keyboard_state.scroll_lock = false;
    keyboard_state.shift = false;
    keyboard_state.ctrl = false;
    keyboard_state.alt = false;
    
    // Reset keyboard
    DEBUG_INFO("KB", "Resetting keyboard");
    keyboard_send_data(KEYBOARD_CMD_RESET);
    int response = keyboard_read_data();
    if (response != KEYBOARD_RES_ACK) {
        DEBUG_ERROR("KB", "Keyboard reset failed (no ACK): 0x%02x", response);
        return STATUS_DEVICE_ERROR;
    }
    response = keyboard_read_data();
    if (response != KEYBOARD_RES_SELF_TEST_OK) {
        DEBUG_ERROR("KB", "Keyboard self test failed: 0x%02x", response);
        return STATUS_DEVICE_ERROR;
    }
    
    // Set default configuration
    DEBUG_INFO("KB", "Setting keyboard defaults");
    keyboard_send_data(KEYBOARD_CMD_SET_DEFAULTS);
    if (keyboard_read_data() != KEYBOARD_RES_ACK) {
        DEBUG_ERROR("KB", "Failed to set keyboard defaults");
        return STATUS_DEVICE_ERROR;
    }
    
    // Enable scanning
    DEBUG_INFO("KB", "Enabling keyboard");
    keyboard_send_data(KEYBOARD_CMD_ENABLE);
    if (keyboard_read_data() != KEYBOARD_RES_ACK) {
        DEBUG_ERROR("KB", "Failed to enable keyboard");
        return STATUS_DEVICE_ERROR;
    }
    
    // Register interrupt handler
    DEBUG_INFO("KB", "Installing keyboard interrupt handler");
    status_t status = interrupt_register_handler(IRQ_BASE + 1, keyboard_interrupt);
    if (status != STATUS_SUCCESS) {
        DEBUG_ERROR("KB", "Failed to register keyboard interrupt handler");
        return status;
    }
    
    // Unmask keyboard IRQ
    pic_unmask_irq(1);
    
    DEBUG_INFO("KB", "Keyboard initialized");
    return STATUS_SUCCESS;
}

status_t keyboard_register_handler(keyboard_handler_t handler) {
    if (!handler) {
        return STATUS_INVALID_PARAM;
    }
    
    if (keyboard_state.num_handlers >= MAX_KEYBOARD_HANDLERS) {
        return STATUS_BUSY;
    }
    
    keyboard_state.handlers[keyboard_state.num_handlers++] = handler;
    return STATUS_SUCCESS;
}

void keyboard_unregister_handler(keyboard_handler_t handler) {
    for (int i = 0; i < keyboard_state.num_handlers; i++) {
        if (keyboard_state.handlers[i] == handler) {
            // Shift remaining handlers down
            for (int j = i; j < keyboard_state.num_handlers - 1; j++) {
                keyboard_state.handlers[j] = keyboard_state.handlers[j + 1];
            }
            keyboard_state.num_handlers--;
            break;
        }
    }
} 