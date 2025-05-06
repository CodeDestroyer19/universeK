#include "debug/debug.h"
#include "io/io.h"
#include <stdarg.h>
#include <stdlib.h>
#include <stdio.h>

// Debug configuration
static debug_level_t current_debug_level = DEBUG_LEVEL_INFO;
static bool use_colors = true;

// Serial port definitions
#define SERIAL_PORT 0x3F8
#define SERIAL_DATA SERIAL_PORT
#define SERIAL_INT  (SERIAL_PORT + 1)
#define SERIAL_FIFO (SERIAL_PORT + 2)
#define SERIAL_LCR  (SERIAL_PORT + 3)
#define SERIAL_MCR  (SERIAL_PORT + 4)
#define SERIAL_LSR  (SERIAL_PORT + 5)

// Color codes
static const char* color_codes[] = {
    "\033[0m",    // None
    "\033[31m",   // Red
    "\033[32m",   // Green
    "\033[33m",   // Yellow
    "\033[34m",   // Blue
    "\033[35m",   // Magenta
    "\033[36m",   // Cyan
    "\033[37m",   // White
};

// Level prefixes
static const char* level_strings[] = {
    "NONE",
    "ERROR",
    "WARN",
    "INFO",
    "DEBUG",
    "TRACE"
};

/**
 * Initialize serial port for debugging
 */
static void init_serial(void) {
    // Disable interrupts
    port_write_byte(SERIAL_INT, 0x00);
    
    // Enable DLAB (set baud rate divisor)
    port_write_byte(SERIAL_LCR, 0x80);
    
    // Set divisor to 3 (38400 baud)
    port_write_byte(SERIAL_DATA, 0x03);
    port_write_byte(SERIAL_INT, 0x00);
    
    // 8 bits, no parity, one stop bit
    port_write_byte(SERIAL_LCR, 0x03);
    
    // Enable FIFO, clear them, with 14-byte threshold
    port_write_byte(SERIAL_FIFO, 0xC7);
    
    // IRQs enabled, RTS/DSR set
    port_write_byte(SERIAL_MCR, 0x0B);
}

/**
 * Check if serial port is ready to transmit
 */
static bool serial_is_transmit_empty(void) {
    return port_read_byte(SERIAL_LSR) & 0x20;
}

/**
 * Write a single character to serial port
 */
static void serial_write_char(char c) {
    while (!serial_is_transmit_empty());
    port_write_byte(SERIAL_DATA, c);
}

/**
 * Write a string to serial port
 */
static void serial_write_string(const char* str) {
    while (*str) {
        serial_write_char(*str++);
    }
}

/**
 * Convert number to string
 */
static char* itoa(int value, char* str, int base) {
    char* rc;
    char* ptr;
    char* low;
    
    if (base < 2 || base > 36) {
        *str = '\0';
        return str;
    }
    
    rc = ptr = str;
    
    if (value < 0 && base == 10) {
        *ptr++ = '-';
    }
    
    low = ptr;
    
    do {
        *ptr++ = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"[abs(value % base)];
        value /= base;
    } while (value);
    
    *ptr-- = '\0';
    
    while (low < ptr) {
        char tmp = *low;
        *low++ = *ptr;
        *ptr-- = tmp;
    }
    
    return rc;
}

void debug_init(void) {
    init_serial();
    DEBUG_INFO("DEBUG", "Debug system initialized");
}

void debug_set_level(debug_level_t level) {
    current_debug_level = level;
}

debug_level_t debug_get_level(void) {
    return current_debug_level;
}

void debug_set_color(bool enable) {
    use_colors = enable;
}

void debug_print(debug_level_t level, debug_color_t color, const char* component, const char* fmt, ...) {
    if (level > current_debug_level) {
        return;
    }
    
    // Print timestamp
    char time_str[16];
    extern uint32_t get_system_ticks(void);  // Defined in timer.c
    uint32_t ticks = get_system_ticks();
    itoa(ticks, time_str, 10);
    
    // Print color if enabled
    if (use_colors) {
        serial_write_string(color_codes[color]);
    }
    
    // Print prefix
    serial_write_char('[');
    serial_write_string(time_str);
    serial_write_string("] ");
    serial_write_string(level_strings[level]);
    serial_write_string(" [");
    serial_write_string(component);
    serial_write_string("] ");
    
    // Print formatted message
    char buf[256];
    va_list args;
    va_start(args, fmt);
    vsprintf(buf, fmt, args);
    va_end(args);
    serial_write_string(buf);
    
    // Reset color and add newline
    if (use_colors) {
        serial_write_string(color_codes[DEBUG_COLOR_NONE]);
    }
    serial_write_string("\r\n");
}

void debug_dump_hex(debug_level_t level, const void* data, size_t size) {
    if (level > current_debug_level) {
        return;
    }
    
    const uint8_t* bytes = (const uint8_t*)data;
    char hex[3];
    
    for (size_t i = 0; i < size; i++) {
        if (i % 16 == 0) {
            if (i > 0) {
                serial_write_string("\r\n");
            }
            char addr[9];
            itoa(i, addr, 16);
            serial_write_string(addr);
            serial_write_string(": ");
        }
        
        itoa(bytes[i], hex, 16);
        if (bytes[i] < 0x10) {
            serial_write_char('0');
        }
        serial_write_string(hex);
        serial_write_char(' ');
    }
    serial_write_string("\r\n");
}

void debug_backtrace(debug_level_t level) {
    if (level > current_debug_level) {
        return;
    }
    
    // Get EBP
    uint32_t ebp;
    asm volatile("mov %%ebp, %0" : "=r"(ebp));
    
    DEBUG_ERROR("BACKTRACE", "Stack trace:");
    
    // Walk the stack
    for (int frame = 0; frame < 10 && ebp; frame++) {
        uint32_t eip = *((uint32_t*)ebp + 1);
        DEBUG_ERROR("BACKTRACE", "  [%d] EIP = 0x%x", frame, eip);
        ebp = *(uint32_t*)ebp;
    }
} 