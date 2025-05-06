#ifndef _DEBUG_H
#define _DEBUG_H

#include <stdint.h>

// Debug levels
#define DEBUG_NONE_LEVEL 0
#define DEBUG_ERROR_LEVEL 1
#define DEBUG_WARN_LEVEL 2
#define DEBUG_INFO_LEVEL 3
#define DEBUG_VERBOSE_LEVEL 4

// Current debug level (can be changed at runtime)
extern uint8_t debug_level;

// Debug functions
void debug_init(void);
void debug_set_level(uint8_t level);
void debug_print(uint8_t level, const char *component, const char *msg);
void debug_print_hex(uint8_t level, const char *component, const char *msg, uint32_t value);

// Helper macros
#define DEBUG_ERROR(comp, msg) debug_print(DEBUG_ERROR_LEVEL, comp, msg)
#define DEBUG_WARN(comp, msg) debug_print(DEBUG_WARN_LEVEL, comp, msg)
#define DEBUG_INFO(comp, msg) debug_print(DEBUG_INFO_LEVEL, comp, msg)
#define DEBUG_VERBOSE(comp, msg) debug_print(DEBUG_VERBOSE_LEVEL, comp, msg)

#define DEBUG_ERROR_HEX(comp, msg, val) debug_print_hex(DEBUG_ERROR_LEVEL, comp, msg, val)
#define DEBUG_WARN_HEX(comp, msg, val) debug_print_hex(DEBUG_WARN_LEVEL, comp, msg, val)
#define DEBUG_INFO_HEX(comp, msg, val) debug_print_hex(DEBUG_INFO_LEVEL, comp, msg, val)
#define DEBUG_VERBOSE_HEX(comp, msg, val) debug_print_hex(DEBUG_VERBOSE_LEVEL, comp, msg, val)

#endif