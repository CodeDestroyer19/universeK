#include "stdio.h"
#include "stdlib.h"
#include "string.h"

static void reverse(char* str, int length) {
    int start = 0;
    int end = length - 1;
    while (start < end) {
        char temp = str[start];
        str[start] = str[end];
        str[end] = temp;
        start++;
        end--;
    }
}

static int itoa_base(int num, char* str, int base) {
    int i = 0;
    int is_negative = 0;

    if (num == 0) {
        str[i++] = '0';
        str[i] = '\0';
        return i;
    }

    if (num < 0 && base == 10) {
        is_negative = 1;
        num = -num;
    }

    while (num != 0) {
        int rem = num % base;
        str[i++] = (rem > 9) ? (rem - 10) + 'a' : rem + '0';
        num = num / base;
    }

    if (is_negative)
        str[i++] = '-';

    str[i] = '\0';
    reverse(str, i);
    return i;
}

int vsprintf(char *str, const char *format, va_list ap) {
    char *str_start = str;
    char num_str[32];
    
    while (*format) {
        if (*format != '%') {
            *str++ = *format++;
            continue;
        }
        
        format++;
        switch (*format) {
            case 's': {
                char *s = va_arg(ap, char*);
                while (*s) *str++ = *s++;
                break;
            }
            case 'd': {
                int num = va_arg(ap, int);
                int len = itoa_base(num, num_str, 10);
                for (int i = 0; i < len; i++) *str++ = num_str[i];
                break;
            }
            case 'x': {
                int num = va_arg(ap, int);
                int len = itoa_base(num, num_str, 16);
                for (int i = 0; i < len; i++) *str++ = num_str[i];
                break;
            }
            case '%':
                *str++ = '%';
                break;
            default:
                *str++ = '%';
                *str++ = *format;
        }
        format++;
    }
    
    *str = '\0';
    return str - str_start;
}

int sprintf(char *str, const char *format, ...) {
    va_list ap;
    va_start(ap, format);
    int ret = vsprintf(str, format, ap);
    va_end(ap);
    return ret;
} 