#ifndef MOUSE_H
#define MOUSE_H

#include "kernel/types.h"

// Mouse packet structure
struct mouse_packet {
    uint8_t buttons;
    int16_t x;
    int16_t y;
    int8_t scroll;
};

// Mouse callback function type
typedef void (*mouse_callback)(struct mouse_packet* packet);

// Initialize the mouse driver
void init_mouse(void);

// Get the current mouse position
void get_mouse_position(int* x, int* y);

// Register a callback for mouse events
void register_mouse_handler(mouse_callback callback);

#endif /* MOUSE_H */