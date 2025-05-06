#ifndef _WINDOW_H
#define _WINDOW_H

#include "kernel/types.h"
#include "vga.h"

// Window types
typedef enum
{
    WINDOW_TYPE_NORMAL,
    WINDOW_TYPE_TERMINAL,
    WINDOW_TYPE_SYSTEM
} WindowType;

// Forward declaration
struct Window;

// Window event handlers
typedef void (*WindowDrawHandler)(struct Window *win);
typedef void (*WindowKeyHandler)(struct Window *win, char key);
typedef void (*WindowUpdateHandler)(struct Window *win);

// Window structure
typedef struct Window
{
    int x;
    int y;
    int width;
    int height;
    char *title;
    WindowType type;
    void *data;
    WindowDrawHandler on_draw;
    WindowKeyHandler on_key;
    WindowUpdateHandler on_update;
} Window;

// Window system initialization
void window_init(void);

// Window functions
Window *window_create(int x, int y, int width, int height, const char *title);
void window_destroy(Window *win);
void window_draw(Window *win);
void window_clear(Window *win, uint8_t color);
void window_draw_char(Window *win, int x, int y, char c, uint8_t color);
void window_putchar(Window *win, int x, int y, char c, uint8_t color); // Alias for draw_char
void window_draw_text(Window *win, int x, int y, const char *text, uint8_t color);
void window_draw_rect(Window *win, int x, int y, int width, int height, uint8_t color);
void window_fill_rect(Window *win, int x, int y, int width, int height, uint8_t color);
Window *get_focused_window(void);
void window_focus(Window *win);
void window_draw_all(void);

#endif /* _WINDOW_H */