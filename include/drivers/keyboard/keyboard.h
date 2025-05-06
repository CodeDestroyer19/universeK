#ifndef _KEYBOARD_H
#define _KEYBOARD_H

#include "kernel/types.h"

/**
 * Keyboard event structure
 */
typedef struct
{
    uint8_t scancode; // Raw scancode
    char key;         // ASCII character (if applicable)
    bool pressed;     // True if key pressed, false if released
    bool shift;       // Shift modifier state
    bool ctrl;        // Ctrl modifier state
    bool alt;         // Alt modifier state
    bool caps_lock;   // Caps lock state
    bool num_lock;    // Num lock state
    bool scroll_lock; // Scroll lock state
} keyboard_event_t;

/**
 * Keyboard event handler function type
 */
typedef void (*keyboard_handler_t)(keyboard_event_t *event);

/**
 * Initialize the keyboard driver
 * @return STATUS_SUCCESS if successful, error code otherwise
 */
status_t keyboard_init(void);

/**
 * Register a keyboard event handler
 * @param handler Handler function to register
 * @return STATUS_SUCCESS if successful, error code otherwise
 */
status_t keyboard_register_handler(keyboard_handler_t handler);

/**
 * Unregister a keyboard event handler
 * @param handler Handler function to unregister
 */
void keyboard_unregister_handler(keyboard_handler_t handler);

#endif /* _KEYBOARD_H */