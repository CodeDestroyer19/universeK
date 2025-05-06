#include "timer.h"
#include "interrupts/interrupt.h"
#include "interrupts/pic.h"
#include "io/io.h"
#include "debug/debug.h"

#define PIT_CHANNEL0 0x40
#define PIT_COMMAND  0x43
#define PIT_FREQUENCY 1193180
#define TIMER_FREQUENCY 100  // 100 Hz = 10ms per tick

static volatile uint32_t timer_ticks = 0;

static void timer_callback(struct interrupt_context* context) {
    (void)context;  // Unused parameter
    timer_ticks++;
    pic_send_eoi(0);  // Send EOI to PIC
}

void timer_init(void) {
    DEBUG_INFO("TIMER", "Initializing system timer");
    
    // Calculate divisor for desired frequency
    uint32_t divisor = PIT_FREQUENCY / TIMER_FREQUENCY;
    
    // Set PIT command byte:
    // 0x36 = 0b00110110
    // - Channel 0
    // - Access mode: lobyte/hibyte
    // - Mode 3 (square wave generator)
    // - Binary mode
    port_write_byte(PIT_COMMAND, 0x36);
    
    // Set frequency divisor
    port_write_byte(PIT_CHANNEL0, divisor & 0xFF);         // Low byte
    port_write_byte(PIT_CHANNEL0, (divisor >> 8) & 0xFF);  // High byte
    
    // Register timer interrupt handler
    interrupt_register_handler(32, timer_callback);  // IRQ0 is mapped to interrupt 32
    
    DEBUG_INFO("TIMER", "System timer initialized at %dHz", TIMER_FREQUENCY);
}

uint32_t get_system_ticks(void) {
    return timer_ticks;
} 