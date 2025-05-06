#ifndef VGA_H
#define VGA_H

#include <stdint.h>
#include <stddef.h>

void vga_init(void);
void vga_clear(uint8_t color);
void vga_draw_pixel(int x, int y, uint8_t color);
void vga_draw_char(int x, int y, char c, uint8_t color);
void vga_draw_string(int x, int y, const char *str, uint8_t color);
void vga_draw_rect(int x, int y, int width, int height, uint8_t color);
void vga_fill_rect(int x, int y, int width, int height, uint8_t color);

#endif /* VGA_H */