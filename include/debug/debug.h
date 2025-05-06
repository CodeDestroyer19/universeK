#ifndef _DEBUG_H
#define _DEBUG_H

#include "kernel/types.h"

/**
 * Debug levels
 */
typedef enum
{
    DEBUG_LEVEL_NONE = 0,
    DEBUG_LEVEL_ERROR,
    DEBUG_LEVEL_WARN,
    DEBUG_LEVEL_INFO,
    DEBUG_LEVEL_DEBUG,
    DEBUG_LEVEL_TRACE,
} debug_level_t;

/**
 * Debug colors
 */
typedef enum
{
    DEBUG_COLOR_NONE = 0,
    DEBUG_COLOR_RED,
    DEBUG_COLOR_GREEN,
    DEBUG_COLOR_YELLOW,
    DEBUG_COLOR_BLUE,
    DEBUG_COLOR_MAGENTA,
    DEBUG_COLOR_CYAN,
    DEBUG_COLOR_WHITE,
} debug_color_t;

/**
 * Debug initialization
 */
void debug_init(void);

/**
 * Set the current debug level
 */
void debug_set_level(debug_level_t level);

/**
 * Get the current debug level
 */
debug_level_t debug_get_level(void);

/**
 * Enable/disable debug colors
 */
void debug_set_color(bool enable);

/**
 * Core debug functions
 */
void debug_print(debug_level_t level, debug_color_t color, const char *component, const char *fmt, ...);
void debug_dump_hex(debug_level_t level, const void *data, size_t size);
void debug_backtrace(debug_level_t level);

/**
 * Helper macros for different debug levels
 */
#define DEBUG_ERROR(component, fmt, ...) \
    debug_print(DEBUG_LEVEL_ERROR, DEBUG_COLOR_RED, component, fmt, ##__VA_ARGS__)

#define DEBUG_WARN(component, fmt, ...) \
    debug_print(DEBUG_LEVEL_WARN, DEBUG_COLOR_YELLOW, component, fmt, ##__VA_ARGS__)

#define DEBUG_INFO(component, fmt, ...) \
    debug_print(DEBUG_LEVEL_INFO, DEBUG_COLOR_WHITE, component, fmt, ##__VA_ARGS__)

#define DEBUG_DEBUG(component, fmt, ...) \
    debug_print(DEBUG_LEVEL_DEBUG, DEBUG_COLOR_CYAN, component, fmt, ##__VA_ARGS__)

#define DEBUG_TRACE(component, fmt, ...) \
    debug_print(DEBUG_LEVEL_TRACE, DEBUG_COLOR_MAGENTA, component, fmt, ##__VA_ARGS__)

/**
 * Helper macros for hex dumps
 */
#define DEBUG_DUMP(level, data, size) \
    debug_dump_hex(level, data, size)

/**
 * Helper macro for backtraces
 */
#define DEBUG_STACK() \
    debug_backtrace(DEBUG_LEVEL_ERROR)

/**
 * Assertion macro
 */
#define ASSERT(condition)                                                                                      \
    do                                                                                                         \
    {                                                                                                          \
        if (!(condition))                                                                                      \
        {                                                                                                      \
            DEBUG_ERROR("ASSERT", "Assertion failed: %s\nFile: %s\nLine: %d", #condition, __FILE__, __LINE__); \
            DEBUG_STACK();                                                                                     \
            for (;;)                                                                                           \
                ;                                                                                              \
        }                                                                                                      \
    } while (0)

#endif /* _DEBUG_H */