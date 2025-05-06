#ifndef INTERRUPT_H
#define INTERRUPT_H

#include <stdint.h>
#include <stdbool.h>
#include "kernel/types.h"

// Interrupt context structure
struct interrupt_context
{
    uint32_t gs, fs, es, ds;
    uint32_t edi, esi, ebp, esp, ebx, edx, ecx, eax;
    uint32_t int_no, err_code;
    uint32_t eip, cs, eflags, useresp, ss;
};

typedef void (*interrupt_handler_t)(struct interrupt_context *);

// Function declarations
void idt_install(void);
uint32_t get_system_ticks(void);
void interrupt_init(void);
status_t interrupt_register_handler(uint8_t int_no, interrupt_handler_t handler);
status_t interrupt_unregister_handler(uint8_t int_no);
bool interrupt_save_disable(void);
void interrupt_restore(bool previous_state);

#endif /* INTERRUPT_H */