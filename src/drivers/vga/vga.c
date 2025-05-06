#include "vga.h"
#include "io/io.h"
#include "debug/debug.h"

#define VGA_WIDTH 320
#define VGA_HEIGHT 200
#define VGA_FRAMEBUFFER 0xA0000

static uint8_t* vga_buffer = (uint8_t*)VGA_FRAMEBUFFER;
static uint8_t font8x8[128][8];  // Basic 8x8 font

void vga_init(void) {
    DEBUG_INFO("VGA", "Initializing VGA driver");
    
    // Set VGA mode 13h (320x200x256 colors)
    port_write_byte(0x3C2, 0x63);
    port_write_byte(0x3D4, 0x00);
    port_write_byte(0x3D5, 0x5F);
    port_write_byte(0x3D4, 0x01);
    port_write_byte(0x3D5, 0x4F);
    
    // Initialize basic font (for now just use simple blocks)
    for (int i = 0; i < 128; i++) {
        for (int j = 0; j < 8; j++) {
            font8x8[i][j] = 0xFF;  // Simple block for each character
        }
    }
    
    DEBUG_INFO("VGA", "VGA initialized in mode 13h");
}

void vga_clear(uint8_t color) {
    for (int i = 0; i < VGA_WIDTH * VGA_HEIGHT; i++) {
        vga_buffer[i] = color;
    }
}

void vga_draw_pixel(int x, int y, uint8_t color) {
    if (x >= 0 && x < VGA_WIDTH && y >= 0 && y < VGA_HEIGHT) {
        vga_buffer[y * VGA_WIDTH + x] = color;
    }
}

void vga_draw_char(int x, int y, char c, uint8_t color) {
    if ((unsigned char)c >= 128) return;
    
    for (int row = 0; row < 8; row++) {
        uint8_t line = font8x8[(unsigned char)c][row];
        for (int col = 0; col < 8; col++) {
            if (line & (1 << (7 - col))) {
                vga_draw_pixel(x + col, y + row, color);
            }
        }
    }
}

void vga_draw_string(int x, int y, const char* str, uint8_t color) {
    int pos_x = x;
    while (*str) {
        vga_draw_char(pos_x, y, *str++, color);
        pos_x += 8;  // Move 8 pixels right for next character
    }
}

void vga_draw_rect(int x, int y, int width, int height, uint8_t color) {
    // Draw horizontal lines
    for (int i = x; i < x + width; i++) {
        vga_draw_pixel(i, y, color);
        vga_draw_pixel(i, y + height - 1, color);
    }
    // Draw vertical lines
    for (int i = y; i < y + height; i++) {
        vga_draw_pixel(x, i, color);
        vga_draw_pixel(x + width - 1, i, color);
    }
}

void vga_fill_rect(int x, int y, int width, int height, uint8_t color) {
    for (int i = y; i < y + height; i++) {
        for (int j = x; j < x + width; j++) {
            vga_draw_pixel(j, i, color);
        }
    }
} 