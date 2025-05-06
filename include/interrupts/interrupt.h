#ifndef _INTERRUPT_H
#define _INTERRUPT_H

#include "kernel/types.h"

/**
 * Interrupt handler context
 */
struct interrupt_context
{
    // Pushed by isr_common_stub
    uint32_t gs, fs, es, ds;
    uint32_t edi, esi, ebp, esp, ebx, edx, ecx, eax;
    uint32_t int_no, err_code;
    // Pushed by processor
    uint32_t eip, cs, eflags, useresp, ss;
} __attribute__((packed));

/**
 * Interrupt handler function type
 */
typedef void (*interrupt_handler_t)(struct interrupt_context *);

/**
 * Initialize the interrupt system
 */
void interrupt_init(void);

/**
 * Register an interrupt handler
 * @param int_no Interrupt number
 * @param handler Handler function
 * @return STATUS_SUCCESS if successful, error code otherwise
 */
status_t interrupt_register_handler(int_t int_no, interrupt_handler_t handler);

/**
 * Unregister an interrupt handler
 * @param int_no Interrupt number
 * @return STATUS_SUCCESS if successful, error code otherwise
 */
status_t interrupt_unregister_handler(int_t int_no);

/**
 * Enable interrupts
 */
static inline void interrupt_enable(void)
{
    asm volatile("sti");
}

/**
 * Disable interrupts
 */
static inline void interrupt_disable(void)
{
    asm volatile("cli");
}

/**
 * Check if interrupts are enabled
 */
static inline bool interrupt_are_enabled(void)
{
    uint32_t flags;
    asm volatile("pushf; pop %0" : "=r"(flags));
    return flags & (1 << 9); // Check IF flag
}

/**
 * Get current interrupt state and disable interrupts
 * @return Previous interrupt state
 */
static inline bool interrupt_save_disable(void)
{
    bool enabled = interrupt_are_enabled();
    interrupt_disable();
    return enabled;
}

/**
 * Restore previous interrupt state
 * @param enabled Previous interrupt state
 */
static inline void interrupt_restore(bool enabled)
{
    if (enabled)
    {
        interrupt_enable();
    }
}

/**
 * Execute code with interrupts disabled
 * @param code Code block to execute
 */
#define INTERRUPT_DISABLE_BLOCK(code)           \
    do                                          \
    {                                           \
        bool __prev = interrupt_save_disable(); \
        code;                                   \
        interrupt_restore(__prev);              \
    } while (0)

#endif /* _INTERRUPT_H */