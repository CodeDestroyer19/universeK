#include <driver.h>
#include "debug/debug.h"
#include <string.h>

#define MAX_DRIVERS 32

static struct driver* drivers[MAX_DRIVERS];
static int num_drivers = 0;

// Register a new driver
int register_driver(struct driver* drv) {
    if (!drv) {
        DEBUG_ERROR("DRIVERS", "Invalid driver pointer");
        return DRIVER_ERROR;
    }

    if (num_drivers >= MAX_DRIVERS) {
        DEBUG_ERROR("DRIVERS", "Maximum number of drivers reached");
        return DRIVER_ERROR;
    }
    
    // Check if driver already exists
    for (int i = 0; i < num_drivers; i++) {
        if (drivers[i] && drivers[i]->name && drv->name && strcmp(drivers[i]->name, drv->name) == 0) {
            DEBUG_ERROR("DRIVERS", "Driver already registered");
            return DRIVER_ERROR;
        }
    }
    
    drivers[num_drivers++] = drv;
    DEBUG_INFO("DRIVERS", "Registered driver");
    return DRIVER_OK;
}

void list_drivers(void) {
    DEBUG_INFO("DRIVERS", "Installed drivers:");
    for (int i = 0; i < num_drivers; i++) {
        if (drivers[i] && drivers[i]->name) {
            DEBUG_INFO("DRIVERS", drivers[i]->name);
        }
    }
    if (num_drivers == 0) {
        DEBUG_INFO("DRIVERS", "No drivers registered");
    }
} 