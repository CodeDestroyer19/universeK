#ifndef _IRQ_H
#define _IRQ_H

// Base IRQ numbers in the IDT
#define IRQ_BASE 0x20
#define IRQ0 (IRQ_BASE + 0)
#define IRQ1 (IRQ_BASE + 1)
#define IRQ2 (IRQ_BASE + 2)
#define IRQ3 (IRQ_BASE + 3)
#define IRQ4 (IRQ_BASE + 4)
#define IRQ5 (IRQ_BASE + 5)
#define IRQ6 (IRQ_BASE + 6)
#define IRQ7 (IRQ_BASE + 7)
#define IRQ8 (IRQ_BASE + 8)
#define IRQ9 (IRQ_BASE + 9)
#define IRQ10 (IRQ_BASE + 10)
#define IRQ11 (IRQ_BASE + 11)
#define IRQ12 (IRQ_BASE + 12)
#define IRQ13 (IRQ_BASE + 13)
#define IRQ14 (IRQ_BASE + 14)
#define IRQ15 (IRQ_BASE + 15)

// Function declarations
void irq_init(void);

#endif /* _IRQ_H */