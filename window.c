#include "window.h"
#include "vga.h"
#include "debug.h"
#include "memory.h"
#include "string.h"
#include "font.h"

// Window list
static struct window* windows[MAX_WINDOWS];
static int num_windows = 0;
static struct window* active_window = NULL;

// Initialize window system
void window_init(void) {
    DEBUG_INFO("WINDOW", "Initializing window system");
    for (int i = 0; i < MAX_WINDOWS; i++) {
        windows[i] = NULL;
    }
    num_windows = 0;
    active_window = NULL;
    DEBUG_INFO("WINDOW", "Window system initialized");
}

// Create a new window
struct window* window_create(int x, int y, int width, int height, const char* title) {
    if (num_windows >= MAX_WINDOWS) {
        DEBUG_ERROR("WINDOW", "Maximum number of windows reached");
        return NULL;
    }

    struct window* win = (struct window*)kmalloc(sizeof(struct window));
    if (!win) {
        DEBUG_ERROR("WINDOW", "Failed to allocate window structure");
        return NULL;
    }

    win->x = x;
    win->y = y;
    win->width = width;
    win->height = height;
    win->flags = WINDOW_VISIBLE | WINDOW_MOVABLE | WINDOW_HAS_TITLE | WINDOW_HAS_BORDER;
    strncpy(win->title, title, 31);
    win->title[31] = '\0';

    // Allocate window buffer
    win->buffer = (uint8_t*)kmalloc(width * height);
    if (!win->buffer) {
        DEBUG_ERROR("WINDOW", "Failed to allocate window buffer");
        kfree(win);
        return NULL;
    }

    // Clear window buffer
    memset(win->buffer, WINDOW_COLOR_BACKGROUND, width * height);

    // Initialize handlers
    win->on_click = NULL;
    win->on_key = NULL;
    win->on_draw = NULL;

    // Add to window list
    windows[num_windows++] = win;
    active_window = win;

    DEBUG_INFO("WINDOW", "Window created");
    return win;
}

// Destroy a window
void window_destroy(struct window* win) {
    if (!win) return;

    // Remove from window list
    for (int i = 0; i < num_windows; i++) {
        if (windows[i] == win) {
            for (int j = i; j < num_windows - 1; j++) {
                windows[j] = windows[j + 1];
            }
            num_windows--;
            break;
        }
    }

    // Update active window
    if (active_window == win) {
        active_window = num_windows > 0 ? windows[num_windows - 1] : NULL;
    }

    // Free memory
    if (win->buffer) kfree(win->buffer);
    kfree(win);

    DEBUG_INFO("WINDOW", "Window destroyed");
}

// Show a window
void window_show(struct window* win) {
    if (!win) return;
    win->flags |= WINDOW_VISIBLE;
}

// Hide a window
void window_hide(struct window* win) {
    if (!win) return;
    win->flags &= ~WINDOW_VISIBLE;
}

// Move a window
void window_move(struct window* win, int x, int y) {
    if (!win || !(win->flags & WINDOW_MOVABLE)) return;
    win->x = x;
    win->y = y;
}

// Resize a window
void window_resize(struct window* win, int width, int height) {
    if (!win || !(win->flags & WINDOW_RESIZABLE)) return;

    uint8_t* new_buffer = (uint8_t*)kmalloc(width * height);
    if (!new_buffer) {
        DEBUG_ERROR("WINDOW", "Failed to allocate new window buffer");
        return;
    }

    // Copy old content
    int min_width = width < win->width ? width : win->width;
    int min_height = height < win->height ? height : win->height;
    for (int y = 0; y < min_height; y++) {
        memcpy(&new_buffer[y * width], &win->buffer[y * win->width], min_width);
    }

    // Fill new areas with background color
    for (int y = 0; y < height; y++) {
        for (int x = min_width; x < width; x++) {
            new_buffer[y * width + x] = WINDOW_COLOR_BACKGROUND;
        }
    }
    for (int y = min_height; y < height; y++) {
        memset(&new_buffer[y * width], WINDOW_COLOR_BACKGROUND, width);
    }

    kfree(win->buffer);
    win->buffer = new_buffer;
    win->width = width;
    win->height = height;
}

// Draw a window
void window_draw(struct window* win) {
    if (!win || !(win->flags & WINDOW_VISIBLE)) return;

    // Draw border
    if (win->flags & WINDOW_HAS_BORDER) {
        vga_draw_rect(win->x - 1, win->y - 1, win->width + 2, win->height + 2, WINDOW_COLOR_BORDER);
    }

    // Draw title bar
    if (win->flags & WINDOW_HAS_TITLE) {
        vga_fill_rect(win->x, win->y - 10, win->width, 10, WINDOW_COLOR_TITLE);
        vga_puts(win->x + 2, win->y - 9, win->title, WINDOW_COLOR_TITLE_TEXT);
    }

    // Draw window contents
    for (int y = 0; y < win->height; y++) {
        for (int x = 0; x < win->width; x++) {
            vga_putpixel(win->x + x, win->y + y, win->buffer[y * win->width + x]);
        }
    }

    // Call custom draw handler
    if (win->on_draw) {
        win->on_draw(win);
    }
}

// Draw all windows
void window_draw_all(void) {
    vga_clear(VGA_BLACK);
    for (int i = 0; i < num_windows; i++) {
        window_draw(windows[i]);
    }
    vga_swap_buffers();
}

// Handle mouse input
void window_handle_mouse(int x, int y, uint8_t buttons) {
    // Find window under cursor
    struct window* target = NULL;
    for (int i = num_windows - 1; i >= 0; i--) {
        struct window* win = windows[i];
        if (!(win->flags & WINDOW_VISIBLE)) continue;

        // Check title bar
        if ((win->flags & WINDOW_HAS_TITLE) &&
            x >= win->x && x < win->x + win->width &&
            y >= win->y - 10 && y < win->y) {
            target = win;
            break;
        }

        // Check window area
        if (x >= win->x && x < win->x + win->width &&
            y >= win->y && y < win->y + win->height) {
            target = win;
            break;
        }
    }

    if (target && buttons) {
        // Make window active
        if (target != active_window) {
            // Move window to top
            for (int i = 0; i < num_windows; i++) {
                if (windows[i] == target) {
                    for (int j = i; j < num_windows - 1; j++) {
                        windows[j] = windows[j + 1];
                    }
                    windows[num_windows - 1] = target;
                    break;
                }
            }
            active_window = target;
        }

        // Call click handler
        if (target->on_click) {
            target->on_click(x - target->x, y - target->y);
        }
    }
}

// Handle keyboard input
void window_handle_key(char key) {
    if (active_window && active_window->on_key) {
        active_window->on_key(key);
    }
}

// Window drawing helpers
void window_clear(struct window* win, uint8_t color) {
    if (!win || !win->buffer) return;
    memset(win->buffer, color, win->width * win->height);
}

void window_putpixel(struct window* win, int x, int y, uint8_t color) {
    if (!win || !win->buffer) return;
    if (x < 0 || x >= win->width || y < 0 || y >= win->height) return;
    win->buffer[y * win->width + x] = color;
}

void window_draw_rect(struct window* win, int x, int y, int width, int height, uint8_t color) {
    if (!win || !win->buffer) return;
    for (int i = 0; i < width; i++) {
        window_putpixel(win, x + i, y, color);
        window_putpixel(win, x + i, y + height - 1, color);
    }
    for (int i = 0; i < height; i++) {
        window_putpixel(win, x, y + i, color);
        window_putpixel(win, x + width - 1, y + i, color);
    }
}

void window_fill_rect(struct window* win, int x, int y, int width, int height, uint8_t color) {
    if (!win || !win->buffer) return;
    for (int j = 0; j < height; j++) {
        for (int i = 0; i < width; i++) {
            window_putpixel(win, x + i, y + j, color);
        }
    }
}

void window_draw_text(struct window* win, int x, int y, const char* text, uint8_t color) {
    if (!win || !win->buffer || !text) return;
    
    int orig_x = x;
    while (*text) {
        if (*text == '\n') {
            y += 9;
            x = orig_x;
        } else {
            for (int i = 0; i < 8; i++) {
                for (int j = 0; j < 8; j++) {
                    if (font8x8_basic[(unsigned char)*text][i] & (1 << j)) {
                        window_putpixel(win, x + j, y + i, color);
                    }
                }
            }
            x += 8;
        }
        text++;
    }
}

void window_putchar(struct window* win, int x, int y, char c, uint8_t color) {
    if (!win || !win->buffer) return;
    
    for (int i = 0; i < 8; i++) {
        for (int j = 0; j < 8; j++) {
            if (font8x8_basic[(unsigned char)c][i] & (1 << j)) {
                window_putpixel(win, x + j, y + i, color);
            }
        }
    }
} 