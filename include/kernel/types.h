#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifndef _KERNEL_TYPES_H
#define _KERNEL_TYPES_H

// Basic integer types for bare metal environment
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;
typedef unsigned int uintptr_t; // For 32-bit x86 architecture

/**
 * Status codes for kernel operations
 */
typedef enum
{
    STATUS_SUCCESS = 0,
    STATUS_ERROR = -1,
    STATUS_TIMEOUT = -2,
    STATUS_INVALID_PARAM = -3,
    STATUS_NOT_IMPLEMENTED = -4,
    STATUS_DEVICE_ERROR = -5,
    STATUS_NO_MEMORY = -6,
    STATUS_BUSY = -7,
    STATUS_NOT_FOUND = -8,
} status_t;

/**
 * Physical memory address type
 */
typedef uintptr_t phys_addr_t;

/**
 * Virtual memory address type
 */
typedef uintptr_t virt_addr_t;

/**
 * IO port type
 */
typedef uint16_t port_t;

/**
 * IRQ number type
 */
typedef uint8_t irq_t;

/**
 * Interrupt number type
 */
typedef uint8_t int_t;

/**
 * Process ID type
 */
typedef uint32_t pid_t;

/**
 * Thread ID type
 */
typedef uint32_t tid_t;

/**
 * Error checking macros
 */
#define IS_ERROR(status) ((status) < 0)
#define IS_SUCCESS(status) ((status) >= 0)

/**
 * Bit manipulation macros
 */
#define BIT(n) (1U << (n))
#define SET_BIT(x, n) ((x) |= BIT(n))
#define CLEAR_BIT(x, n) ((x) &= ~BIT(n))
#define TEST_BIT(x, n) ((x) & BIT(n))

/**
 * Array size macro
 */
#define ARRAY_SIZE(x) (sizeof(x) / sizeof((x)[0]))

/**
 * Alignment macros
 */
#define ALIGN_UP(x, align) (((x) + ((align) - 1)) & ~((align) - 1))
#define ALIGN_DOWN(x, align) ((x) & ~((align) - 1))
#define IS_ALIGNED(x, align) (((x) & ((align) - 1)) == 0)

/**
 * Page size constants
 */
#define PAGE_SIZE 4096
#define PAGE_SHIFT 12
#define PAGE_MASK (~(PAGE_SIZE - 1))

#endif /* _KERNEL_TYPES_H */