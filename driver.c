#include "driver.h"
#include "string.h"

#define MAX_DRIVERS 32

static struct driver* drivers[MAX_DRIVERS];
static int num_drivers = 0;

// Register a new driver
int register_driver(struct driver* drv) {
    if (num_drivers >= MAX_DRIVERS) {
        return DRIVER_ERROR;
    }
    
    // Check if driver already exists
    for (int i = 0; i < num_drivers; i++) {
        if (strcmp(drivers[i]->name, drv->name) == 0) {
            return DRIVER_ERROR;
        }
    }
    
    drivers[num_drivers++] = drv;
    return DRIVER_OK;
}

// Get a driver by name
struct driver* get_driver(const char* name) {
    for (int i = 0; i < num_drivers; i++) {
        if (strcmp(drivers[i]->name, name) == 0) {
            return drivers[i];
        }
    }
    return NULL;
}

// List all registered drivers
void list_drivers(void) {
    extern void terminal_writestring(const char* data);
    
    terminal_writestring("Registered drivers:\n");
    for (int i = 0; i < num_drivers; i++) {
        terminal_writestring("  - ");
        terminal_writestring(drivers[i]->name);
        terminal_writestring("\n");
    }
}

// I/O helper functions
void outb(uint16_t port, uint8_t val) {
    asm volatile ("outb %0, %1" :: "a"(val), "Nd"(port));
}

uint8_t inb(uint16_t port) {
    uint8_t ret;
    asm volatile ("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

void io_wait(void) {
    // Port 0x80 is used for 'checkpoints' during POST.
    // Writing to it should delay for a small amount of time
    outb(0x80, 0);
} 