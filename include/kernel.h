#ifndef _KERNEL_H
#define _KERNEL_H

#include <stddef.h>
#include <stdint.h>

extern const size_t VGA_WIDTH;
extern const size_t VGA_HEIGHT;

// VGA functions
void terminal_initialize(void);
void terminal_putchar(char c);
void terminal_write(const char *data, size_t size);
void terminal_writestring(const char *data);
size_t get_terminal_row(void);
size_t get_terminal_column(void);

// Command handling
void handle_command(const char *cmd);

// Serial port functions
void init_serial(void);
int is_transmit_empty(void);
void write_serial(char c);
void write_serial_string(const char* str);

#endif