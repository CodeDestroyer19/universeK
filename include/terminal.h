#ifndef _TERMINAL_H
#define _TERMINAL_H

#include "window.h"

/**
 * Initialize a terminal window
 * @param win Window to initialize as terminal
 */
void terminal_init(Window *win);

/**
 * Handle keyboard input to terminal
 * @param win Terminal window
 * @param c Character to process
 */
void terminal_input_char(Window *win, char c);

/**
 * Update terminal state (cursor blinking, etc)
 * @param win Terminal window
 */
void terminal_update(Window *win);

#endif /* _TERMINAL_H */