#include "terminal.h"
#include "vga.h"
#include "memory/malloc.h"
#include "debug/debug.h"
#include <string.h>
#include <stdlib.h>


#define TERMINAL_BUFFER_SIZE 1024
#define TERMINAL_INPUT_SIZE 256

typedef struct {
    char buffer[TERMINAL_BUFFER_SIZE];
    int buffer_pos;
    char input_buffer[TERMINAL_INPUT_SIZE];
    int input_pos;
    int cursor_x;
    int cursor_y;
    int width;
    int height;
    bool cursor_visible;
} Terminal;

void terminal_init(Window* win) {
    Terminal* term = (Terminal*)malloc(sizeof(Terminal));
    memset(term, 0, sizeof(Terminal));
    term->width = win->width / 8;  // Assuming 8x8 font
    term->height = win->height / 8;
    term->cursor_visible = true;
    win->data = term;
}

void terminal_input_char(Window* win, char c) {
    Terminal* term = (Terminal*)win->data;
    
    // Handle backspace
    if (c == '\b') {
        if (term->input_pos > 0) {
            term->input_pos--;
            term->input_buffer[term->input_pos] = 0;
            // Update display
            term->cursor_x--;
            if (term->cursor_x < 0) {
                term->cursor_x = term->width - 1;
                term->cursor_y--;
            }
            window_draw_char(win, term->cursor_x * 8, term->cursor_y * 8, ' ', VGA_WHITE);
        }
        return;
    }
    
    // Handle enter
    if (c == '\n' || c == '\r') {
        // Process command
        term->input_buffer[term->input_pos] = 0;
        DEBUG_INFO("TERM", "Command: %s", term->input_buffer);
        
        // Add to output buffer
        term->cursor_x = 0;
        term->cursor_y++;
        if (term->cursor_y >= term->height) {
            term->cursor_y = term->height - 1;
            // Scroll terminal
            // TODO: Implement scrolling
        }
        
        // Clear input buffer
        term->input_pos = 0;
        memset(term->input_buffer, 0, TERMINAL_INPUT_SIZE);
        return;
    }
    
    // Regular character
    if (term->input_pos < TERMINAL_INPUT_SIZE - 1) {
        term->input_buffer[term->input_pos++] = c;
        window_draw_char(win, term->cursor_x * 8, term->cursor_y * 8, c, VGA_WHITE);
        term->cursor_x++;
        if (term->cursor_x >= term->width) {
            term->cursor_x = 0;
            term->cursor_y++;
            if (term->cursor_y >= term->height) {
                term->cursor_y = term->height - 1;
                // TODO: Implement scrolling
            }
        }
    }
}

void terminal_update(Window* win) {
    Terminal* term = (Terminal*)win->data;
    
    // Update cursor
    static int blink_counter = 0;
    if (++blink_counter >= 10) {
        blink_counter = 0;
        term->cursor_visible = !term->cursor_visible;
        
        if (term->cursor_visible) {
            window_draw_char(win, term->cursor_x * 8, term->cursor_y * 8, '_', VGA_WHITE);
        } else {
            window_draw_char(win, term->cursor_x * 8, term->cursor_y * 8, ' ', VGA_WHITE);
        }
    }
} 