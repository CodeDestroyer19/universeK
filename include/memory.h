#ifndef _MEMORY_H
#define _MEMORY_H

#include <stddef.h>

// Memory management functions
void memory_init(void);
void *kmalloc(size_t size);
void kfree(void *ptr);

#endif