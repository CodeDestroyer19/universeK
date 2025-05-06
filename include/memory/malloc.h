#ifndef _MALLOC_H
#define _MALLOC_H

#include "kernel/types.h"

void *malloc(size_t size);
void free(void *ptr);
void *calloc(size_t num, size_t size);
void *realloc(void *ptr, size_t size);

#endif /* _MALLOC_H */