#ifndef _STRING_H
#define _STRING_H

#include "kernel/types.h"

// String functions
size_t strlen(const char *str);
int strcmp(const char *s1, const char *s2);
int strncmp(const char *s1, const char *s2, size_t n);
char *strchr(const char *s, int c);
char *strcpy(char *dest, const char *src);
char *strncpy(char *dest, const char *src, size_t n);
char *strdup(const char *s);

// Memory functions
void *memcpy(void *dest, const void *src, size_t n);
void *memset(void *s, int c, size_t n);
int memcmp(const void *ptr1, const void *ptr2, size_t count);

#endif