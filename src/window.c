#include "window.h"
#include "debug/debug.h"
#include "string.h"
#include "memory/malloc.h"

#define MAX_WINDOWS 16

static Window* windows[MAX_WINDOWS];
static int num_windows = 0;
static Window* focused_window = NULL;

void window_init(void) {
    DEBUG_INFO("WINDOW", "Initializing window system");
    num_windows = 0;
    focused_window = NULL;
    memset(windows, 0, sizeof(windows));
}

Window* window_create(int x, int y, int width, int height, const char* title) {
    if (num_windows >= MAX_WINDOWS) {
        DEBUG_ERROR("WINDOW", "Maximum number of windows reached");
        return NULL;
    }

    Window* win = (Window*)malloc(sizeof(Window));
    if (!win) {
        DEBUG_ERROR("WINDOW", "Failed to allocate window");
        return NULL;
    }

    win->x = x;
    win->y = y;
    win->width = width;
    win->height = height;
    win->title = strdup(title);
    win->type = WINDOW_TYPE_NORMAL;
    win->data = NULL;
    win->on_draw = NULL;
    win->on_key = NULL;
    win->on_update = NULL;

    windows[num_windows++] = win;
    return win;
}

void window_destroy(Window* win) {
    if (!win) return;

    // Find and remove from windows array
    for (int i = 0; i < num_windows; i++) {
        if (windows[i] == win) {
            // Shift remaining windows down
            for (int j = i; j < num_windows - 1; j++) {
                windows[j] = windows[j + 1];
            }
            num_windows--;
            break;
        }
    }

    if (focused_window == win) {
        focused_window = NULL;
    }

    free(win->title);
    free(win);
}

void window_draw(Window* win) {
    if (!win) return;

    // Draw window border
    vga_draw_rect(win->x, win->y, win->width, win->height, VGA_LIGHT_GRAY);

    // Draw title bar
    vga_fill_rect(win->x, win->y, win->width, 10, VGA_BLUE);
    vga_draw_string(win->x + 2, win->y + 1, win->title, VGA_WHITE);

    // Call custom draw handler
    if (win->on_draw) {
        win->on_draw(win);
    }
}

void window_clear(Window* win, uint8_t color) {
    if (!win) return;
    vga_fill_rect(win->x + 1, win->y + 11, win->width - 2, win->height - 12, color);
}

void window_draw_char(Window* win, int x, int y, char c, uint8_t color) {
    if (!win) return;
    vga_draw_char(win->x + x + 1, win->y + y + 11, c, color);
}

void window_draw_text(Window* win, int x, int y, const char* text, uint8_t color) {
    if (!win) return;
    vga_draw_string(win->x + x + 1, win->y + y + 11, text, color);
}

void window_draw_rect(Window* win, int x, int y, int width, int height, uint8_t color) {
    if (!win) return;
    vga_draw_rect(win->x + x + 1, win->y + y + 11, width, height, color);
}

void window_fill_rect(Window* win, int x, int y, int width, int height, uint8_t color) {
    if (!win) return;
    vga_fill_rect(win->x + x + 1, win->y + y + 11, width, height, color);
}

Window* get_focused_window(void) {
    return focused_window;
}

void window_focus(Window* win) {
    focused_window = win;
}

void window_draw_all(void) {
    // Clear screen
    vga_clear(VGA_BLACK);

    // Draw all windows
    for (int i = 0; i < num_windows; i++) {
        window_draw(windows[i]);
    }
}

void window_putchar(Window* win, int x, int y, char c, uint8_t color) {
    window_draw_char(win, x, y, c, color);
} 