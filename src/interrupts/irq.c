#include "interrupts/irq.h"
#include "interrupts/interrupt.h"
#include "debug/debug.h"

void irq_init(void) {
    DEBUG_INFO("IRQ", "Initializing IRQ system");
    // IRQ setup is handled by PIC initialization
    DEBUG_INFO("IRQ", "IRQ system initialized");
} 