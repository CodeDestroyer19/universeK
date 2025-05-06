#ifndef _INTERRUPT_H
#define _INTERRUPT_H

#include <stdint.h>

// Register structure passed to interrupt handlers
struct regs
{
    uint32_t gs, fs, es, ds;
    uint32_t edi, esi, ebp, esp, ebx, edx, ecx, eax;
    uint32_t int_no, err_code;
    uint32_t eip, cs, eflags, useresp, ss;
};

// IRQ handler type
typedef void (*irq_handler_t)(struct regs *);

// Initialize the IRQ system
void irq_init(void);

// Install an IRQ handler
void irq_install_handler(int irq, irq_handler_t handler);

// Remove an IRQ handler
void irq_uninstall_handler(int irq);

#endif