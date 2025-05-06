#include "interrupts/interrupt.h"
#include "debug/debug.h"
#include "io/io.h"
#include <string.h>

// Maximum number of interrupts (255 since int_t is uint8_t)
#define MAX_INTERRUPTS 255

// IDT entry structure
struct idt_entry {
    uint16_t base_low;
    uint16_t selector;
    uint8_t zero;
    uint8_t flags;
    uint16_t base_high;
} __attribute__((packed));

// IDT pointer structure
struct idt_ptr {
    uint16_t limit;
    uint32_t base;
} __attribute__((packed));

// Interrupt handlers array
static interrupt_handler_t interrupt_handlers[MAX_INTERRUPTS];

// IDT array
static struct idt_entry idt[MAX_INTERRUPTS];

// IDT pointer
static struct idt_ptr idtp;

// External assembly functions
extern void idt_load(struct idt_ptr* ptr);
extern void isr_common_stub(void);

// ISR handlers (defined in interrupt_asm.asm)
extern void isr0(void);
extern void isr1(void);
// ... Add more ISR handlers as needed

// IRQ handlers (defined in interrupt_asm.asm)
extern void irq0(void);
extern void irq1(void);
extern void irq2(void);
extern void irq3(void);
extern void irq4(void);
extern void irq5(void);
extern void irq6(void);
extern void irq7(void);
extern void irq8(void);
extern void irq9(void);
extern void irq10(void);
extern void irq11(void);
extern void irq12(void);
extern void irq13(void);
extern void irq14(void);
extern void irq15(void);

void idt_install(void) {
    interrupt_init();
}

/**
 * Set an IDT gate
 */
static void idt_set_gate(uint8_t num, uint32_t base, uint16_t selector, uint8_t flags) {
    idt[num].base_low = base & 0xFFFF;
    idt[num].base_high = (base >> 16) & 0xFFFF;
    idt[num].selector = selector;
    idt[num].zero = 0;
    idt[num].flags = flags;
}

/**
 * Common interrupt handler
 */
void interrupt_handler(struct interrupt_context* context) {
    DEBUG_TRACE("INT", "Interrupt %d received", context->int_no);
    
    if (interrupt_handlers[context->int_no]) {
        interrupt_handlers[context->int_no](context);
    } else {
        DEBUG_WARN("INT", "Unhandled interrupt %d", context->int_no);
    }
}

void interrupt_init(void) {
    DEBUG_INFO("INT", "Initializing interrupt system");
    
    // Clear interrupt handlers array
    for (int i = 0; i < MAX_INTERRUPTS; i++) {
        interrupt_handlers[i] = NULL;
    }
    
    // Set up IDT pointer
    idtp.limit = (sizeof(struct idt_entry) * MAX_INTERRUPTS) - 1;
    idtp.base = (uint32_t)&idt;
    
    // Clear IDT
    memset(&idt, 0, sizeof(struct idt_entry) * MAX_INTERRUPTS);
    
    // Set up exception handlers
    idt_set_gate(0, (uint32_t)isr0, 0x08, 0x8E);
    idt_set_gate(1, (uint32_t)isr1, 0x08, 0x8E);
    // ... Add more exception handlers
    
    // Set up IRQ handlers
    idt_set_gate(32, (uint32_t)irq0, 0x08, 0x8E);
    idt_set_gate(33, (uint32_t)irq1, 0x08, 0x8E);
    idt_set_gate(34, (uint32_t)irq2, 0x08, 0x8E);
    idt_set_gate(35, (uint32_t)irq3, 0x08, 0x8E);
    idt_set_gate(36, (uint32_t)irq4, 0x08, 0x8E);
    idt_set_gate(37, (uint32_t)irq5, 0x08, 0x8E);
    idt_set_gate(38, (uint32_t)irq6, 0x08, 0x8E);
    idt_set_gate(39, (uint32_t)irq7, 0x08, 0x8E);
    idt_set_gate(40, (uint32_t)irq8, 0x08, 0x8E);
    idt_set_gate(41, (uint32_t)irq9, 0x08, 0x8E);
    idt_set_gate(42, (uint32_t)irq10, 0x08, 0x8E);
    idt_set_gate(43, (uint32_t)irq11, 0x08, 0x8E);
    idt_set_gate(44, (uint32_t)irq12, 0x08, 0x8E);
    idt_set_gate(45, (uint32_t)irq13, 0x08, 0x8E);
    idt_set_gate(46, (uint32_t)irq14, 0x08, 0x8E);
    idt_set_gate(47, (uint32_t)irq15, 0x08, 0x8E);
    
    // Load IDT
    idt_load(&idtp);
    
    DEBUG_INFO("INT", "Interrupt system initialized");
}

status_t interrupt_register_handler(int_t int_no, interrupt_handler_t handler) {
    if (int_no >= MAX_INTERRUPTS) {
        return STATUS_INVALID_PARAM;
    }
    
    DEBUG_INFO("INT", "Registering handler for interrupt %d", int_no);
    
    INTERRUPT_DISABLE_BLOCK({
        interrupt_handlers[int_no] = handler;
    });
    
    return STATUS_SUCCESS;
}

status_t interrupt_unregister_handler(int_t int_no) {
    if (int_no >= MAX_INTERRUPTS) {
        return STATUS_INVALID_PARAM;
    }
    
    DEBUG_INFO("INT", "Unregistering handler for interrupt %d", int_no);
    
    INTERRUPT_DISABLE_BLOCK({
        interrupt_handlers[int_no] = NULL;
    });
    
    return STATUS_SUCCESS;
} 