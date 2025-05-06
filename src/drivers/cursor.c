#include "cursor.h"
#include "io/io.h"

void update_cursor(int x, int y, int width) {
    uint16_t pos = y * width + x;
 
    port_write_byte(0x3D4, 0x0F);
    port_write_byte(0x3D5, (uint8_t)(pos & 0xFF));
    port_write_byte(0x3D4, 0x0E);
    port_write_byte(0x3D5, (uint8_t)((pos >> 8) & 0xFF));
} 