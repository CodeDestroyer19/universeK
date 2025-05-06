#ifndef _STDIO_H
#define _STDIO_H

#include "stdarg.h"

int vsprintf(char *str, const char *format, va_list ap);
int sprintf(char *str, const char *format, ...);

#endif /* _STDIO_H */