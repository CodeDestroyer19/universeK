#include <stdint.h>
#include "driver.h"

#define CRTC_COMMAND_PORT 0x3D4
#define CRTC_DATA_PORT 0x3D5
#define CURSOR_HIGH_PORT 0x0E
#define CURSOR_LOW_PORT 0x0F

void update_cursor(int x, int y, int width) {
    uint16_t pos = y * width + x;

    outb(CRTC_COMMAND_PORT, CURSOR_HIGH_PORT);
    outb(CRTC_DATA_PORT, (uint8_t)((pos >> 8) & 0xFF));
    outb(CRTC_COMMAND_PORT, CURSOR_LOW_PORT);
    outb(CRTC_DATA_PORT, (uint8_t)(pos & 0xFF));
}

void enable_cursor(void) {
    outb(CRTC_COMMAND_PORT, 0x0A);
    outb(CRTC_DATA_PORT, (inb(CRTC_DATA_PORT) & 0xC0) | 0x0E);
    outb(CRTC_COMMAND_PORT, 0x0B);
    outb(CRTC_DATA_PORT, (inb(CRTC_DATA_PORT) & 0xE0) | 0x0F);
} 