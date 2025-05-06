#ifndef _PIC_H
#define _PIC_H

#include "kernel/types.h"

/**
 * Initialize the PIC
 */
void pic_init(void);

/**
 * Send End-of-Interrupt to PIC
 * @param irq IRQ number (0-15)
 */
void pic_send_eoi(uint8_t irq);

/**
 * Mask (disable) an IRQ
 * @param irq IRQ number (0-15)
 */
void pic_mask_irq(uint8_t irq);

/**
 * Unmask (enable) an IRQ
 * @param irq IRQ number (0-15)
 */
void pic_unmask_irq(uint8_t irq);

/**
 * Disable the PIC (useful for APIC mode)
 */
void pic_disable(void);

#endif /* _PIC_H */