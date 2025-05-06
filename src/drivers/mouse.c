#include "mouse.h"
#include "io/io.h"
#include "debug/debug.h"
#include "interrupts/interrupt.h"
#include "interrupts/pic.h"
#include "interrupts/irq.h"
#include <driver.h>
#include "kernel/types.h"

// Mouse ports and commands
#define MOUSE_PORT   0x60
#define MOUSE_STATUS 0x64
#define MOUSE_CMD    0x64
#define MOUSE_IRQ    12

// Mouse command bytes
#define MOUSE_ENABLE         0xA8    // Enable auxiliary mouse device
#define MOUSE_GET_STATUS     0x20    // Get controller command byte
#define MOUSE_SET_STATUS     0x60    // Set controller command byte
#define MOUSE_WRITE_MOUSE    0xD4    // Write to mouse
#define MOUSE_DEFAULT        0xF6    // Set default parameters
#define MOUSE_ENABLE_PACKET  0xF4    // Enable packet streaming
#define MOUSE_DISABLE       0xF5    // Disable mouse
#define MOUSE_RESET         0xFF    // Reset mouse

// Mouse responses
#define MOUSE_ACK           0xFA    // Command acknowledged
#define MOUSE_ERROR         0xFC    // Command error
#define MOUSE_SELF_TEST     0xAA    // Self test passed

// Mouse state
static int mouse_x = 0;
static int mouse_y = 0;
static uint8_t mouse_cycle = 0;
static uint8_t mouse_packet[3];
static mouse_callback handler = NULL;

// Driver structure
static struct driver mouse_driver = {
    .name = "ps2_mouse",
    .type = DRIVER_TYPE_CHAR,
    .init = NULL,
    .read = NULL,
    .write = NULL,
    .ioctl = NULL,
    .cleanup = NULL
};

// Wait for mouse input buffer to be clear
static void mouse_wait(uint8_t type) {
    uint32_t timeout = 100000;
    if (type == 0) {
        while (timeout--) {
            if ((port_read_byte(MOUSE_STATUS) & 1) == 1) {
                return;
            }
        }
        DEBUG_ERROR("MOUSE", "Timeout waiting for input buffer (read)");
    } else {
        while (timeout--) {
            if ((port_read_byte(MOUSE_STATUS) & 2) == 0) {
                return;
            }
        }
        DEBUG_ERROR("MOUSE", "Timeout waiting for input buffer (write)");
    }
}

// Write to mouse controller
static void mouse_write_cmd(uint8_t cmd) {
    mouse_wait(1);
    port_write_byte(MOUSE_CMD, cmd);
}

// Write to mouse device
static void mouse_write(uint8_t data) {
    DEBUG_INFO("MOUSE", "Writing command 0x%02X", data);
    mouse_wait(1);
    port_write_byte(MOUSE_CMD, MOUSE_WRITE_MOUSE);
    mouse_wait(1);
    port_write_byte(MOUSE_PORT, data);
}

// Read from mouse with timeout
static int mouse_read_timeout(void) {
    uint32_t timeout = 100000;
    while (timeout--) {
        if (port_read_byte(MOUSE_STATUS) & 1) {
            return port_read_byte(MOUSE_PORT);
        }
    }
    return -1;
}

// Read from mouse
static uint8_t mouse_read(void) {
    mouse_wait(0);
    uint8_t data = port_read_byte(MOUSE_PORT);
    DEBUG_INFO("MOUSE", "Read data 0x%02X", data);
    return data;
}

// Wait for and verify ACK
static bool mouse_expect_ack(void) {
    int response = mouse_read_timeout();
    if (response < 0) {
        DEBUG_ERROR("MOUSE", "Timeout waiting for ACK");
        return false;
    }
    if (response != MOUSE_ACK) {
        DEBUG_ERROR("MOUSE", "Expected ACK (0xFA) but got 0x%02X", response);
        return false;
    }
    return true;
}

// Mouse interrupt handler
static void mouse_handler(struct interrupt_context* context) {
    (void)context; // Unused parameter
    
    uint8_t status = port_read_byte(MOUSE_STATUS);
    if (!(status & 0x20)) {
        DEBUG_ERROR("MOUSE", "Spurious mouse interrupt");
        return; // Not mouse data
    }
    
    uint8_t data = mouse_read();
    
    switch(mouse_cycle) {
        case 0:
            mouse_packet[0] = data;
            if (data & 0x08) { // Check if this is the start of a packet
                DEBUG_INFO("MOUSE", "Packet start received");
                mouse_cycle++;
            } else {
                DEBUG_WARN("MOUSE", "Invalid first byte");
            }
            break;
        case 1:
            mouse_packet[1] = data;
            DEBUG_INFO("MOUSE", "X movement");
            mouse_cycle++;
            break;
        case 2:
            mouse_packet[2] = data;
            DEBUG_INFO("MOUSE", "Y movement");
            
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
            
            DEBUG_INFO("MOUSE", "Position updated");
            
            // Call handler if registered
            if (handler) {
                struct mouse_packet packet = {
                    .buttons = mouse_packet[0] & 0x07,
                    .x = mouse_x,
                    .y = mouse_y,
                    .scroll = (mouse_packet[0] >> 3) & 0x0F
                };
                DEBUG_INFO("MOUSE", "Calling mouse handler");
                handler(&packet);
            }
            
            mouse_cycle = 0;
            break;
    }
    
    // Send EOI to PIC
    pic_send_eoi(MOUSE_IRQ);
}

void init_mouse(void) {
    DEBUG_INFO("MOUSE", "Initializing PS/2 mouse");
    
    // Disable interrupts during initialization
    pic_mask_irq(MOUSE_IRQ);
    
    // Enable the auxiliary mouse device
    DEBUG_INFO("MOUSE", "Enabling auxiliary mouse device");
    mouse_write_cmd(MOUSE_ENABLE);
    
    // Get current controller command byte
    DEBUG_INFO("MOUSE", "Getting controller status");
    mouse_write_cmd(MOUSE_GET_STATUS);
    uint8_t status = mouse_read();
    
    // Modify status: Enable IRQ12, disable IRQ1, enable mouse
    status |= 0x02;  // Enable IRQ12
    status &= ~0x10; // Enable mouse
    status |= 0x20;  // Enable mouse clock
    
    // Write back the modified command byte
    DEBUG_INFO("MOUSE", "Setting controller configuration");
    mouse_write_cmd(MOUSE_SET_STATUS);
    mouse_wait(1);
    port_write_byte(MOUSE_PORT, status);
    
    // Disable mouse device before configuration
    DEBUG_INFO("MOUSE", "Disabling mouse");
    mouse_write(MOUSE_DISABLE);
    if (!mouse_expect_ack()) return;
    
    // Reset the mouse device
    DEBUG_INFO("MOUSE", "Resetting mouse");
    mouse_write(MOUSE_RESET);
    if (!mouse_expect_ack()) return;
    
    // Wait for self-test completion
    int response = mouse_read_timeout();
    if (response != MOUSE_SELF_TEST) {
        DEBUG_ERROR("MOUSE", "Mouse self-test failed: got 0x%02X, expected 0xAA", response);
        return;
    }
    
    // Set default parameters
    DEBUG_INFO("MOUSE", "Setting mouse defaults");
    mouse_write(MOUSE_DEFAULT);
    if (!mouse_expect_ack()) return;
    
    // Enable packet streaming
    DEBUG_INFO("MOUSE", "Enabling packet streaming");
    mouse_write(MOUSE_ENABLE_PACKET);
    if (!mouse_expect_ack()) return;
    
    // Register interrupt handler
    interrupt_register_handler(IRQ_BASE + MOUSE_IRQ, mouse_handler);
    
    // Enable mouse interrupt
    pic_unmask_irq(MOUSE_IRQ);
    
    // Register driver
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