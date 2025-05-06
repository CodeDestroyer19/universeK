#include "interrupts/pic.h"
#include "io/io.h"
#include "debug/debug.h"

// PIC ports
#define PIC1_COMMAND 0x20
#define PIC1_DATA    0x21
#define PIC2_COMMAND 0xA0
#define PIC2_DATA    0xA1

// PIC commands
#define PIC_EOI      0x20
#define ICW1_ICW4    0x01
#define ICW1_SINGLE  0x02
#define ICW1_INTERVAL4 0x04
#define ICW1_LEVEL   0x08
#define ICW1_INIT    0x10
#define ICW4_8086    0x01

// IRQ mappings
#define IRQ_BASE     0x20
#define IRQ_COUNT    16

/**
 * Send End-of-Interrupt to PIC
 */
void pic_send_eoi(uint8_t irq) {
    if (irq >= 8) {
        port_write_byte(PIC2_COMMAND, PIC_EOI);
    }
    port_write_byte(PIC1_COMMAND, PIC_EOI);
}

/**
 * Mask (disable) an IRQ
 */
void pic_mask_irq(uint8_t irq) {
    uint16_t port;
    uint8_t value;
    
    if (irq < 8) {
        port = PIC1_DATA;
    } else {
        port = PIC2_DATA;
        irq -= 8;
    }
    
    value = port_read_byte(port) | (1 << irq);
    port_write_byte(port, value);
}

/**
 * Unmask (enable) an IRQ
 */
void pic_unmask_irq(uint8_t irq) {
    uint16_t port;
    uint8_t value;
    
    if (irq < 8) {
        port = PIC1_DATA;
    } else {
        port = PIC2_DATA;
        irq -= 8;
    }
    
    value = port_read_byte(port) & ~(1 << irq);
    port_write_byte(port, value);
}

/**
 * Initialize the PIC
 */
void pic_init(void) {
    DEBUG_INFO("PIC", "Initializing PIC");
    
    uint8_t mask1, mask2;
    
    // Save masks
    mask1 = port_read_byte(PIC1_DATA);
    mask2 = port_read_byte(PIC2_DATA);
    
    DEBUG_INFO("PIC", "Remapping PIC");
    
    // Start initialization sequence
    port_write_byte(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
    io_wait();
    port_write_byte(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
    io_wait();
    
    // Set vector offsets
    port_write_byte(PIC1_DATA, IRQ_BASE);     // IRQ 0-7: int 0x20-0x27
    io_wait();
    port_write_byte(PIC2_DATA, IRQ_BASE + 8); // IRQ 8-15: int 0x28-0x2F
    io_wait();
    
    // Tell PICs about each other
    port_write_byte(PIC1_DATA, 4);            // Tell Master about Slave at IRQ2
    io_wait();
    port_write_byte(PIC2_DATA, 2);            // Tell Slave its cascade identity
    io_wait();
    
    // Set 8086 mode
    port_write_byte(PIC1_DATA, ICW4_8086);
    io_wait();
    port_write_byte(PIC2_DATA, ICW4_8086);
    io_wait();
    
    DEBUG_INFO("PIC", "Setting interrupt masks");
    
    // Mask all interrupts except:
    // - IRQ0 (timer)
    // - IRQ1 (keyboard)
    // - IRQ2 (cascade)
    // - IRQ12 (PS/2 mouse)
    mask1 = ~((1 << 0) | (1 << 1) | (1 << 2));  // Enable IRQ0, IRQ1, IRQ2
    mask2 = ~(1 << 4);  // Enable IRQ12 (12-8 = 4 in the second PIC)
    
    // Set masks
    port_write_byte(PIC1_DATA, mask1);
    port_write_byte(PIC2_DATA, mask2);
    
    DEBUG_INFO("PIC", "PIC initialized");
}

/**
 * Disable the PIC (useful for APIC mode)
 */
void pic_disable(void) {
    DEBUG_INFO("PIC", "Disabling PIC");
    port_write_byte(PIC1_DATA, 0xFF);
    port_write_byte(PIC2_DATA, 0xFF);
} 