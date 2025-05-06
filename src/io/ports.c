#include "io/io.h"
#include "kernel/types.h"
#include "debug/debug.h"

// IO delay constant - number of iterations for delay
#define IO_DELAY_ITERATIONS 1000

/**
 * Read a byte from an IO port
 * @param port The port to read from
 * @return The byte read
 */
uint8_t port_read_byte(uint16_t port) {
    uint8_t value;
    asm volatile ("inb %1, %0" : "=a"(value) : "Nd"(port));
    return value;
}

/**
 * Write a byte to an IO port
 * @param port The port to write to
 * @param value The value to write
 */
void port_write_byte(uint16_t port, uint8_t value) {
    asm volatile ("outb %0, %1" :: "a"(value), "Nd"(port));
}

/**
 * Read a word from an IO port
 * @param port The port to read from
 * @return The word read
 */
uint16_t port_read_word(uint16_t port) {
    uint16_t value;
    asm volatile ("inw %1, %0" : "=a"(value) : "Nd"(port));
    return value;
}

/**
 * Write a word to an IO port
 * @param port The port to write to
 * @param value The value to write
 */
void port_write_word(uint16_t port, uint16_t value) {
    asm volatile ("outw %0, %1" :: "a"(value), "Nd"(port));
}

/**
 * Read a long from an IO port
 * @param port The port to read from
 * @return The long read
 */
uint32_t port_read_long(uint16_t port) {
    uint32_t value;
    asm volatile ("inl %1, %0" : "=a"(value) : "Nd"(port));
    return value;
}

/**
 * Write a long to an IO port
 * @param port The port to write to
 * @param value The value to write
 */
void port_write_long(uint16_t port, uint32_t value) {
    asm volatile ("outl %0, %1" :: "a"(value), "Nd"(port));
}

/**
 * Perform a small delay by reading from an unused port
 * This is more reliable than just reading port 0x80
 */
void io_wait(void) {
    // Read from the keyboard controller status port
    // This is better than port 0x80 as it's an actual device
    for (int i = 0; i < IO_DELAY_ITERATIONS; i++) {
        port_read_byte(0x64);
    }
}

/**
 * Wait for a specific bit to be set in a port
 * @param port The port to read from
 * @param mask The bit mask to check
 * @param set Whether to wait for the bit to be set or cleared
 * @param timeout Maximum number of iterations to wait
 * @return true if successful, false if timeout
 */
bool port_wait_bit(uint16_t port, uint8_t mask, bool set, uint32_t timeout) {
    while (timeout--) {
        uint8_t value = port_read_byte(port);
        if (set) {
            if (value & mask) return true;
        } else {
            if (!(value & mask)) return true;
        }
        io_wait();
    }
    return false;
} 