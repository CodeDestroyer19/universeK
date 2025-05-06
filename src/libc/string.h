#ifndef STRING_H
#define STRING_H

#include <stddef.h>
#include <stdarg.h>

void *memset(void *s, int c, size_t n);
void *memcpy(void *dest, const void *src, size_t n);
int memcmp(const void *s1, const void *s2, size_t n);
size_t strlen(const char *s);
int strcmp(const char *s1, const char *s2);
char *strcpy(char *dest, const char *src);
char *strdup(const char *s);

// Add missing string functions
int strncmp(const char *s1, const char *s2, size_t n);
char *strncpy(char *dest, const char *src, size_t n);
char *strchr(const char *s, int c);
int vsprintf(char *str, const char *format, va_list args);

#endif /* STRING_H */