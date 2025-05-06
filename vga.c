#include "vga.h"
#include "debug.h"
#include "font.h"
#include <stddef.h>
#include "driver.h"  // for outb/inb
#include "memory.h"  // for kmalloc
#include <string.h>

// Helper function for absolute value
static inline int abs(int x) {
    return x < 0 ? -x : x;
}

// VGA memory and ports
#define VGA_AC_INDEX 0x3C0
#define VGA_AC_WRITE 0x3C0
#define VGA_AC_READ 0x3C1
#define VGA_MISC_WRITE 0x3C2
#define VGA_SEQ_INDEX 0x3C4
#define VGA_SEQ_DATA 0x3C5
#define VGA_DAC_READ_INDEX 0x3C7
#define VGA_DAC_WRITE_INDEX 0x3C8
#define VGA_DAC_DATA 0x3C9
#define VGA_MISC_READ 0x3CC
#define VGA_GC_INDEX 0x3CE
#define VGA_GC_DATA 0x3CF
#define VGA_CRTC_INDEX 0x3D4
#define VGA_CRTC_DATA 0x3D5
#define VGA_INSTAT_READ 0x3DA

#define VGA_NUM_SEQ_REGS 5
#define VGA_NUM_CRTC_REGS 25
#define VGA_NUM_GC_REGS 9
#define VGA_NUM_AC_REGS 21

// VGA memory
static uint8_t* vga_memory = (uint8_t*)0xA0000;
static uint8_t* back_buffer = NULL;

// Register values for 320x200x256 mode
static unsigned char g_320x200x256[] = {
    /* MISC */
    0x63,
    /* SEQ */
    0x03, 0x01, 0x0F, 0x00, 0x0E,
    /* CRTC */
    0x5F, 0x4F, 0x50, 0x82, 0x54, 0x80, 0xBF, 0x1F,
    0x00, 0x41, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x9C, 0x0E, 0x8F, 0x28, 0x40, 0x96, 0xB9, 0xA3,
    0xFF,
    /* GC */
    0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x05, 0x0F,
    0xFF,
    /* AC */
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
    0x41, 0x00, 0x0F, 0x00, 0x00
};

// Write to VGA registers
static void write_registers(unsigned char* regs) {
    unsigned i;

    // Write MISCELLANEOUS reg
    outb(VGA_MISC_WRITE, *regs);
    regs++;

    // Write SEQUENCER regs
    for(i = 0; i < VGA_NUM_SEQ_REGS; i++) {
        outb(VGA_SEQ_INDEX, i);
        outb(VGA_SEQ_DATA, *regs);
        regs++;
    }

    // Unlock CRTC registers
    outb(VGA_CRTC_INDEX, 0x03);
    outb(VGA_CRTC_DATA, inb(VGA_CRTC_DATA) | 0x80);
    outb(VGA_CRTC_INDEX, 0x11);
    outb(VGA_CRTC_DATA, inb(VGA_CRTC_DATA) & ~0x80);

    // Write CRTC regs
    for(i = 0; i < VGA_NUM_CRTC_REGS; i++) {
        outb(VGA_CRTC_INDEX, i);
        outb(VGA_CRTC_DATA, *regs);
        regs++;
    }

    // Write GRAPHICS CONTROLLER regs
    for(i = 0; i < VGA_NUM_GC_REGS; i++) {
        outb(VGA_GC_INDEX, i);
        outb(VGA_GC_DATA, *regs);
        regs++;
    }

    // Write ATTRIBUTE CONTROLLER regs
    for(i = 0; i < VGA_NUM_AC_REGS; i++) {
        inb(VGA_INSTAT_READ);
        outb(VGA_AC_INDEX, i);
        outb(VGA_AC_WRITE, *regs);
        regs++;
    }

    // Lock 16-color palette and unblank display
    inb(VGA_INSTAT_READ);
    outb(VGA_AC_INDEX, 0x20);
}

void vga_init(void) {
    DEBUG_INFO("VGA", "Initializing VGA graphics mode");
    
    // Switch to graphics mode
    write_registers(g_320x200x256);
    
    // Clear screen
    vga_clear(VGA_BLACK);
    
    // Allocate back buffer - use kmalloc instead of hardcoded address
    back_buffer = (uint8_t*)kmalloc(VGA_GRAPHICS_WIDTH * VGA_GRAPHICS_HEIGHT);
    if (!back_buffer) {
        DEBUG_ERROR("VGA", "Failed to allocate back buffer");
        return;
    }
    
    // Clear back buffer
    memset(back_buffer, 0, VGA_GRAPHICS_WIDTH * VGA_GRAPHICS_HEIGHT);
    
    DEBUG_INFO("VGA", "VGA graphics mode initialized");
}

void vga_putpixel(int x, int y, uint8_t color) {
    if (x >= 0 && x < VGA_GRAPHICS_WIDTH && y >= 0 && y < VGA_GRAPHICS_HEIGHT) {
        back_buffer[y * VGA_GRAPHICS_WIDTH + x] = color;
    }
}

void vga_clear(uint8_t color) {
    for (int i = 0; i < VGA_PIXELS; i++) {
        back_buffer[i] = color;
    }
}

void vga_draw_line(int x1, int y1, int x2, int y2, uint8_t color) {
    int dx = abs(x2 - x1);
    int dy = abs(y2 - y1);
    int sx = x1 < x2 ? 1 : -1;
    int sy = y1 < y2 ? 1 : -1;
    int err = (dx > dy ? dx : -dy) / 2;
    int e2;

    while (1) {
        vga_putpixel(x1, y1, color);
        if (x1 == x2 && y1 == y2) break;
        e2 = err;
        if (e2 > -dx) { err -= dy; x1 += sx; }
        if (e2 < dy) { err += dx; y1 += sy; }
    }
}

void vga_draw_rect(int x, int y, int width, int height, uint8_t color) {
    vga_draw_line(x, y, x + width - 1, y, color);
    vga_draw_line(x + width - 1, y, x + width - 1, y + height - 1, color);
    vga_draw_line(x, y + height - 1, x + width - 1, y + height - 1, color);
    vga_draw_line(x, y, x, y + height - 1, color);
}

void vga_fill_rect(int x, int y, int width, int height, uint8_t color) {
    for (int i = 0; i < height; i++) {
        for (int j = 0; j < width; j++) {
            vga_putpixel(x + j, y + i, color);
        }
    }
}

void vga_draw_circle(int x0, int y0, int radius, uint8_t color) {
    int x = radius;
    int y = 0;
    int err = 0;

    while (x >= y) {
        vga_putpixel(x0 + x, y0 + y, color);
        vga_putpixel(x0 + y, y0 + x, color);
        vga_putpixel(x0 - y, y0 + x, color);
        vga_putpixel(x0 - x, y0 + y, color);
        vga_putpixel(x0 - x, y0 - y, color);
        vga_putpixel(x0 - y, y0 - x, color);
        vga_putpixel(x0 + y, y0 - x, color);
        vga_putpixel(x0 + x, y0 - y, color);

        if (err <= 0) {
            y += 1;
            err += 2*y + 1;
        }
        if (err > 0) {
            x -= 1;
            err -= 2*x + 1;
        }
    }
}

void vga_fill_circle(int x0, int y0, int radius, uint8_t color) {
    int x = radius;
    int y = 0;
    int err = 0;

    while (x >= y) {
        for (int i = -x; i <= x; i++) {
            vga_putpixel(x0 + i, y0 + y, color);
            vga_putpixel(x0 + i, y0 - y, color);
        }
        for (int i = -y; i <= y; i++) {
            vga_putpixel(x0 + i, y0 + x, color);
            vga_putpixel(x0 + i, y0 - x, color);
        }

        if (err <= 0) {
            y += 1;
            err += 2*y + 1;
        }
        if (err > 0) {
            x -= 1;
            err -= 2*x + 1;
        }
    }
}

void vga_putchar(int x, int y, char c, uint8_t color) {
    // Font implementation will be added
    extern const uint8_t font8x8_basic[128][8];
    
    for (int i = 0; i < 8; i++) {
        for (int j = 0; j < 8; j++) {
            if (font8x8_basic[(unsigned char)c][i] & (1 << j)) {
                vga_putpixel(x + j, y + i, color);
            }
        }
    }
}

void vga_puts(int x, int y, const char* str, uint8_t color) {
    int orig_x = x;
    while (*str) {
        if (*str == '\n') {
            y += 9;
            x = orig_x;
        } else {
            vga_putchar(x, y, *str, color);
            x += 8;
        }
        str++;
    }
}

void vga_swap_buffers(void) {
    if (!back_buffer) {
        DEBUG_ERROR("VGA", "Back buffer is NULL");
        return;
    }
    
    // Copy back buffer to VGA memory
    memcpy(vga_memory, back_buffer, VGA_GRAPHICS_WIDTH * VGA_GRAPHICS_HEIGHT);
}

// Add cleanup function
void vga_cleanup(void) {
    if (back_buffer) {
        kfree(back_buffer);
        back_buffer = NULL;
    }
    DEBUG_INFO("VGA", "VGA resources cleaned up");
} 