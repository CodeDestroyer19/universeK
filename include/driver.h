#ifndef _DRIVER_H
#define _DRIVER_H

#include <stdint.h>
#include <stddef.h>

// Driver status codes
#define DRIVER_OK 0
#define DRIVER_ERROR -1
#define DRIVER_NOT_IMPLEMENTED -2

// Driver types
#define DRIVER_TYPE_CHAR 1  // Character devices (keyboard, mouse, etc.)
#define DRIVER_TYPE_BLOCK 2 // Block devices (disk, etc.)
#define DRIVER_TYPE_NET 3   // Network devices

// Driver structure
struct driver
{
    const char *name;                            // Driver name
    int type;                                    // Driver type
    int (*init)(void);                           // Initialize the device
    int (*read)(void *buf, size_t count);        // Read from device
    int (*write)(const void *buf, size_t count); // Write to device
    int (*ioctl)(uint32_t cmd, void *arg);       // Device-specific control
    void (*cleanup)(void);                       // Cleanup/shutdown
};

// Driver registration and management
int register_driver(struct driver *drv);
struct driver *get_driver(const char *name);
void list_drivers(void);

// Helper functions for drivers
void outb(uint16_t port, uint8_t val);
uint8_t inb(uint16_t port);
void io_wait(void);

#endif