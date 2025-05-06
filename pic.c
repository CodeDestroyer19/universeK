#include <stdint.h>
#include "driver.h"  // for io_wait

#define PIC1_COMMAND 0x20
#define PIC1_DATA 0x21
#define PIC2_COMMAND 0xA0
#define PIC2_DATA 0xA1

#define ICW1_ICW4 0x01
#define ICW1_SINGLE 0x02
#define ICW1_INTERVAL4 0x04
#define ICW1_LEVEL 0x08
#define ICW1_INIT 0x10

#define ICW4_8086 0x01

extern void write_serial_string(const char* str);

void pic_remap(void) {
    write_serial_string("Remapping PIC...\n");
    
    outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
    io_wait();
    outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
    io_wait();

    outb(PIC1_DATA, 0x20);  // Start PIC1 at 32
    io_wait();
    outb(PIC2_DATA, 0x28);  // Start PIC2 at 40
    io_wait();

    outb(PIC1_DATA, 4);     // Tell PIC1 about PIC2
    io_wait();
    outb(PIC2_DATA, 2);     // Tell PIC2 its cascade identity
    io_wait();

    outb(PIC1_DATA, ICW4_8086);
    io_wait();
    outb(PIC2_DATA, ICW4_8086);
    io_wait();

    write_serial_string("PIC remapped successfully\n");
}

void pic_init(void) {
    pic_remap();
    write_serial_string("Setting up PIC masks...\n");

    // Unmask all IRQs for testing
    outb(PIC1_DATA, 0x00);
    io_wait();
    outb(PIC2_DATA, 0x00);
    io_wait();
    write_serial_string("PIC masks cleared (all IRQs unmasked)\n");
} 