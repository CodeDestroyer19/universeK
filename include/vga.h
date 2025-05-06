#ifndef _VGA_H
#define _VGA_H

#include "kernel/types.h"

// VGA color constants
#define VGA_BLACK 0x00
#define VGA_BLUE 0x01
#define VGA_GREEN 0x02
#define VGA_CYAN 0x03
#define VGA_RED 0x04
#define VGA_MAGENTA 0x05
#define VGA_BROWN 0x06
#define VGA_LIGHT_GRAY 0x07
#define VGA_DARK_GRAY 0x08
#define VGA_LIGHT_BLUE 0x09
#define VGA_LIGHT_GREEN 0x0A
#define VGA_LIGHT_CYAN 0x0B
#define VGA_LIGHT_RED 0x0C
#define VGA_LIGHT_MAGENTA 0x0D
#define VGA_YELLOW 0x0E
#define VGA_WHITE 0x0F

// VGA dimensions
#define VGA_WIDTH 320
#define VGA_HEIGHT 200

// VGA functions
void vga_init(void);
void vga_clear(uint8_t color);
void vga_putpixel(int x, int y, uint8_t color);
void vga_draw_char(int x, int y, char c, uint8_t color);
void vga_draw_string(int x, int y, const char *str, uint8_t color);
void vga_draw_rect(int x, int y, int width, int height, uint8_t color);
void vga_fill_rect(int x, int y, int width, int height, uint8_t color);

#endif /* _VGA_H */