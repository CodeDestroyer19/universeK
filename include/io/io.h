#ifndef _IO_H
#define _IO_H

#include <stdbool.h>
#include "kernel/types.h"

/**
 * Port IO operations
 */
uint8_t port_read_byte(uint16_t port);
void port_write_byte(uint16_t port, uint8_t value);
uint16_t port_read_word(uint16_t port);
void port_write_word(uint16_t port, uint16_t value);
uint32_t port_read_long(uint16_t port);
void port_write_long(uint16_t port, uint32_t value);

/**
 * IO timing operations
 */
void io_wait(void);
bool port_wait_bit(uint16_t port, uint8_t mask, bool set, uint32_t timeout);

#endif /* _IO_H */