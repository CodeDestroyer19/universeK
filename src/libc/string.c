#include "string.h"
#include "memory/malloc.h"
#include <stdarg.h>

void* memset(void* s, int c, size_t n) {
    unsigned char* p = s;
    while (n--) {
        *p++ = (unsigned char)c;
    }
    return s;
}

void* memcpy(void* dest, const void* src, size_t n) {
    unsigned char* d = dest;
    const unsigned char* s = src;
    while (n--) {
        *d++ = *s++;
    }
    return dest;
}

int memcmp(const void* s1, const void* s2, size_t n) {
    const unsigned char* p1 = s1;
    const unsigned char* p2 = s2;
    while (n--) {
        if (*p1 != *p2) {
            return *p1 - *p2;
        }
        p1++;
        p2++;
    }
    return 0;
}

size_t strlen(const char* s) {
    const char* p = s;
    while (*p) p++;
    return p - s;
}

int strcmp(const char* s1, const char* s2) {
    while (*s1 && *s1 == *s2) {
        s1++;
        s2++;
    }
    return *(unsigned char*)s1 - *(unsigned char*)s2;
}

char* strcpy(char* dest, const char* src) {
    char* d = dest;
    while ((*d++ = *src++));
    return dest;
}

char* strdup(const char* s) {
    size_t len = strlen(s) + 1;  // +1 for null terminator
    char* new_str = malloc(len);
    if (new_str) {
        memcpy(new_str, s, len);
    }
    return new_str;
}

int strncmp(const char* s1, const char* s2, size_t n) {
    while (n && *s1 && (*s1 == *s2)) {
        ++s1;
        ++s2;
        --n;
    }
    if (n == 0) {
        return 0;
    }
    return (*(unsigned char*)s1 - *(unsigned char*)s2);
}

char* strncpy(char* dest, const char* src, size_t n) {
    size_t i;
    for (i = 0; i < n && src[i] != '\0'; i++) {
        dest[i] = src[i];
    }
    for (; i < n; i++) {
        dest[i] = '\0';
    }
    return dest;
}

char* strchr(const char* s, int c) {
    while (*s != (char)c) {
        if (!*s++) {
            return NULL;
        }
    }
    return (char*)s;
}

int vsprintf(char* str, const char* format, va_list args) {
    // Basic implementation for now
    char* s;
    int d;
    char* start = str;

    while (*format) {
        if (*format != '%') {
            *str++ = *format++;
            continue;
        }

        format++;
        switch (*format) {
            case 's':
                s = va_arg(args, char*);
                while (*s) {
                    *str++ = *s++;
                }
                break;
            case 'd':
                d = va_arg(args, int);
                if (d < 0) {
                    *str++ = '-';
                    d = -d;
                }
                // Convert number to string
                char num[32];
                int i = 0;
                do {
                    num[i++] = d % 10 + '0';
                    d /= 10;
                } while (d);
                while (--i >= 0) {
                    *str++ = num[i];
                }
                break;
            default:
                *str++ = *format;
        }
        format++;
    }
    *str = '\0';
    return str - start;
} 