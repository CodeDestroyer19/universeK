#ifndef _MOUSE_H
#define _MOUSE_H

#include <stdint.h>

// Mouse packet structure
struct mouse_packet
{
    uint8_t buttons;
    int16_t x;
    int16_t y;
    int8_t scroll;
};

// Mouse button states
#define MOUSE_LEFT_BUTTON 0x01
#define MOUSE_RIGHT_BUTTON 0x02
#define MOUSE_MIDDLE_BUTTON 0x04

// Initialize the mouse driver
void init_mouse(void);

// Get the current mouse position
void get_mouse_position(int *x, int *y);

// Register a callback for mouse events
typedef void (*mouse_callback)(struct mouse_packet *packet);
void register_mouse_handler(mouse_callback callback);

#endif