#include "debug.h"

uint8_t debug_level = DEBUG_INFO_LEVEL;  // Default debug level

// External functions we need
extern void write_serial_string(const char* str);
extern void terminal_writestring(const char* str);

// Level strings
static const char* level_strings[] = {
    "NONE",
    "ERROR",
    "WARN",
    "INFO",
    "VERBOSE"
};

// Buffer for number conversion
static char num_buffer[32];

// Convert a number to hex string
static void uint_to_hex(uint32_t num, char* str) {
    const char hex_digits[] = "0123456789ABCDEF";
    int i = 0;
    
    // Handle 0 case
    if (num == 0) {
        str[0] = '0';
        str[1] = '\0';
        return;
    }
    
    // Convert number to hex digits
    while (num > 0) {
        str[i++] = hex_digits[num & 0xF];
        num >>= 4;
    }
    
    // Add 0x prefix
    str[i++] = 'x';
    str[i++] = '0';
    str[i] = '\0';
    
    // Reverse the string
    for (int j = 0; j < i/2; j++) {
        char temp = str[j];
        str[j] = str[i-1-j];
        str[i-1-j] = temp;
    }
}

void debug_init(void) {
    debug_level = DEBUG_INFO_LEVEL;
    debug_print(DEBUG_INFO_LEVEL, "DEBUG", "Debug system initialized");
}

void debug_set_level(uint8_t level) {
    if (level <= DEBUG_VERBOSE_LEVEL) {
        debug_level = level;
    }
}

void debug_print(uint8_t level, const char* component, const char* msg) {
    if (level <= debug_level) {
        write_serial_string("[");
        write_serial_string(level_strings[level]);
        write_serial_string("][");
        write_serial_string(component);
        write_serial_string("] ");
        write_serial_string(msg);
        write_serial_string("\n");
        
        // Also print errors to screen
        if (level == DEBUG_ERROR_LEVEL) {
            terminal_writestring("[ERROR] ");
            terminal_writestring(msg);
            terminal_writestring("\n");
        }
    }
}

void debug_print_hex(uint8_t level, const char* component, const char* msg, uint32_t value) {
    if (level <= debug_level) {
        write_serial_string("[");
        write_serial_string(level_strings[level]);
        write_serial_string("][");
        write_serial_string(component);
        write_serial_string("] ");
        write_serial_string(msg);
        write_serial_string(" ");
        uint_to_hex(value, num_buffer);
        write_serial_string(num_buffer);
        write_serial_string("\n");
        
        // Also print errors to screen
        if (level == DEBUG_ERROR_LEVEL) {
            terminal_writestring("[ERROR] ");
            terminal_writestring(msg);
            terminal_writestring(" ");
            terminal_writestring(num_buffer);
            terminal_writestring("\n");
        }
    }
} 