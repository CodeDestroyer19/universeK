#include "interrupt.h"
#include "pic.h"
#include <stddef.h>
#include "driver.h"
#include "debug.h"
#include "kernel.h"

#define IRQ_COUNT 16

static irq_handler_t irq_handlers[IRQ_COUNT];

// Initialize IRQ system
void irq_init(void) {
    DEBUG_INFO("IRQ", "Initializing IRQ system");
    
    // Initialize all handlers to NULL
    for (int i = 0; i < IRQ_COUNT; i++) {
        irq_handlers[i] = NULL;
    }
    
    DEBUG_INFO("IRQ", "IRQ handlers initialized to NULL");
}

void irq_install_handler(int irq, irq_handler_t handler) {
    if (irq >= 0 && irq < IRQ_COUNT) {
        DEBUG_INFO_HEX("IRQ", "Installing handler for IRQ", irq);
        irq_handlers[irq] = handler;
    } else {
        DEBUG_ERROR_HEX("IRQ", "Invalid IRQ number", irq);
    }
}

void irq_uninstall_handler(int irq) {
    if (irq >= 0 && irq < IRQ_COUNT) {
        DEBUG_INFO_HEX("IRQ", "Uninstalling handler for IRQ", irq);
        irq_handlers[irq] = NULL;
    } else {
        DEBUG_ERROR_HEX("IRQ", "Invalid IRQ number", irq);
    }
}

// This function is called from assembly
void irq_handler(struct regs* r) {
    int irq = r->int_no - 32;
    
    // Enhanced debug logging
    write_serial_string("\n[IRQ] Interrupt received: ");
    char irq_num[4];
    irq_num[0] = '0' + ((r->int_no / 10) % 10);
    irq_num[1] = '0' + (r->int_no % 10);
    irq_num[2] = '\n';
    irq_num[3] = '\0';
    write_serial_string(irq_num);

    // Log specific IRQ types
    if (r->int_no == 32) {
        write_serial_string("[IRQ] Timer interrupt\n");
    } else if (r->int_no == 33) {
        write_serial_string("[IRQ] Keyboard interrupt\n");
    } else if (r->int_no == 44) {
        write_serial_string("[IRQ] Mouse interrupt\n");
    }
    
    if (irq >= 0 && irq < IRQ_COUNT) {
        if (irq_handlers[irq]) {
            DEBUG_INFO_HEX("IRQ", "Calling handler for IRQ", irq);
            irq_handlers[irq](r);
            DEBUG_INFO_HEX("IRQ", "Handler completed for IRQ", irq);
        } else {
            DEBUG_WARN_HEX("IRQ", "No handler for IRQ", irq);
        }
    } else {
        DEBUG_ERROR_HEX("IRQ", "Invalid IRQ number from interrupt", r->int_no);
    }
    
    // Send EOI (End of Interrupt)
    if (irq >= 8) {
        DEBUG_VERBOSE("IRQ", "Sending EOI to slave PIC");
        outb(0xA0, 0x20);  // Send EOI to slave PIC
        io_wait();  // Add small delay
    }
    DEBUG_VERBOSE("IRQ", "Sending EOI to master PIC");
    outb(0x20, 0x20);  // Send EOI to master PIC
    io_wait();  // Add small delay
} 