#include "mouse.h"
#include "driver.h"
#include "interrupt.h"
#include "debug.h"

#define MOUSE_PORT   0x60
#define MOUSE_STATUS 0x64
#define MOUSE_CMD    0x64
#define MOUSE_IRQ    12

static int mouse_x = 0;
static int mouse_y = 0;
static mouse_callback handler = NULL;
static uint8_t mouse_cycle = 0;
static uint8_t mouse_packet[4];

// Mouse command bytes
#define MOUSE_ENABLE 0xA8
#define MOUSE_GET_STATUS 0x20
#define MOUSE_SET_STATUS 0x60
#define MOUSE_DEFAULT 0xF6
#define MOUSE_ENABLE_PACKET 0xF4

// Wait for mouse input buffer to be clear
static void mouse_wait(uint8_t type) {
    uint32_t timeout = 100000;
    if (type == 0) {
        while (timeout--) {
            if ((inb(MOUSE_STATUS) & 1) == 1) {
                return;
            }
        }
        DEBUG_ERROR("MOUSE", "Timeout waiting for input buffer (read)");
    } else {
        while (timeout--) {
            if ((inb(MOUSE_STATUS) & 2) == 0) {
                return;
            }
        }
        DEBUG_ERROR("MOUSE", "Timeout waiting for input buffer (write)");
    }
}

// Write to mouse
static void mouse_write(uint8_t data) {
    DEBUG_VERBOSE_HEX("MOUSE", "Writing command", data);
    mouse_wait(1);
    outb(MOUSE_CMD, 0xD4);
    mouse_wait(1);
    outb(MOUSE_PORT, data);
}

// Read from mouse
static uint8_t mouse_read(void) {
    mouse_wait(0);
    uint8_t data = inb(MOUSE_PORT);
    DEBUG_VERBOSE_HEX("MOUSE", "Read data", data);
    return data;
}

// Mouse interrupt handler
static void mouse_handler(struct regs* r) {
    (void)r; // Unused parameter
    
    uint8_t status = inb(MOUSE_STATUS);
    if (!(status & 0x20)) {
        DEBUG_VERBOSE("MOUSE", "Spurious mouse interrupt");
        return; // Not mouse data
    }
    
    uint8_t data = mouse_read();
    
    switch(mouse_cycle) {
        case 0:
            mouse_packet[0] = data;
            if (data & 0x08) { // Check if this is the start of a packet
                DEBUG_VERBOSE_HEX("MOUSE", "Packet start received", data);
                mouse_cycle++;
            } else {
                DEBUG_WARN_HEX("MOUSE", "Invalid first byte", data);
            }
            break;
        case 1:
            mouse_packet[1] = data;
            DEBUG_VERBOSE_HEX("MOUSE", "X movement", data);
            mouse_cycle++;
            break;
        case 2:
            mouse_packet[2] = data;
            DEBUG_VERBOSE_HEX("MOUSE", "Y movement", data);
            
            // Process packet
            if (mouse_packet[0] & 0x10) mouse_packet[1] |= 0xFF00; // X sign bit
            if (mouse_packet[0] & 0x20) mouse_packet[2] |= 0xFF00; // Y sign bit
            
            // Update position
            mouse_x += (int16_t)mouse_packet[1];
            mouse_y -= (int16_t)mouse_packet[2]; // Y is inverted
            
            // Clamp position
            if (mouse_x < 0) mouse_x = 0;
            if (mouse_y < 0) mouse_y = 0;
            if (mouse_x > 79) mouse_x = 79;
            if (mouse_y > 24) mouse_y = 24;
            
            DEBUG_VERBOSE_HEX("MOUSE", "New X position", mouse_x);
            DEBUG_VERBOSE_HEX("MOUSE", "New Y position", mouse_y);
            
            // Call handler if registered
            if (handler) {
                struct mouse_packet packet = {
                    .buttons = mouse_packet[0] & 0x07,
                    .x = mouse_x,
                    .y = mouse_y,
                    .scroll = (mouse_packet[0] >> 3) & 0x0F
                };
                DEBUG_VERBOSE("MOUSE", "Calling mouse handler");
                handler(&packet);
            }
            
            mouse_cycle = 0;
            break;
    }
}

// Initialize the mouse
void init_mouse(void) {
    DEBUG_INFO("MOUSE", "Initializing PS/2 mouse");
    uint8_t status;
    
    // Enable the auxiliary mouse device
    DEBUG_INFO("MOUSE", "Enabling auxiliary mouse device");
    mouse_wait(1);
    outb(MOUSE_CMD, MOUSE_ENABLE);
    
    // Enable the interrupts
    DEBUG_INFO("MOUSE", "Enabling mouse interrupts");
    mouse_wait(1);
    outb(MOUSE_CMD, MOUSE_GET_STATUS);
    mouse_wait(0);
    status = inb(MOUSE_PORT) | 2;
    mouse_wait(1);
    outb(MOUSE_CMD, MOUSE_SET_STATUS);
    mouse_wait(1);
    outb(MOUSE_PORT, status);
    
    // Set default settings
    DEBUG_INFO("MOUSE", "Setting mouse defaults");
    mouse_write(MOUSE_DEFAULT);
    mouse_read(); // Acknowledge
    
    // Enable packet streaming
    DEBUG_INFO("MOUSE", "Enabling packet streaming");
    mouse_write(MOUSE_ENABLE_PACKET);
    mouse_read(); // Acknowledge
    
    // Install mouse handler
    DEBUG_INFO("MOUSE", "Installing mouse IRQ handler");
    irq_install_handler(MOUSE_IRQ, mouse_handler);
    
    // Register mouse driver
    DEBUG_INFO("MOUSE", "Registering mouse driver");
    static struct driver mouse_driver = {
        .name = "ps2_mouse",
        .type = DRIVER_TYPE_CHAR,
        .init = NULL, // Already initialized
        .read = NULL, // We use callback instead
        .write = NULL,
        .ioctl = NULL,
        .cleanup = NULL
    };
    register_driver(&mouse_driver);
    
    DEBUG_INFO("MOUSE", "Mouse initialization complete");
}

void get_mouse_position(int* x, int* y) {
    *x = mouse_x;
    *y = mouse_y;
}

void register_mouse_handler(mouse_callback callback) {
    DEBUG_INFO("MOUSE", callback ? "Registering mouse handler" : "Unregistering mouse handler");
    handler = callback;
} 