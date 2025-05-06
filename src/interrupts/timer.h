#ifndef TIMER_H
#define TIMER_H

#include <stdint.h>

void timer_init(void);
uint32_t get_system_ticks(void);

#endif /* TIMER_H */