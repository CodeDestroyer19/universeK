#include <stdint.h>
#include <stddef.h>
#include "memory.h"

// Memory map entry from GRUB
struct multiboot_mmap_entry {
    uint32_t size;
    uint64_t addr;
    uint64_t len;
    uint32_t type;
} __attribute__((packed));

#define BLOCK_SIZE 4096
#define BLOCKS_PER_BUCKET 8
#define HEAP_START 0x400000  // Start at 4MB
#define HEAP_SIZE  0x400000  // 4MB heap size

static uint8_t* heap_start = (uint8_t*)HEAP_START;
static uint8_t* heap_end = (uint8_t*)(HEAP_START + HEAP_SIZE);
static uint32_t* bitmap = (uint32_t*)HEAP_START;
static size_t bitmap_size;

extern void write_serial_string(const char* str);

void memory_init(void) {
    write_serial_string("Initializing memory management...\n");
    
    // Calculate bitmap size (1 bit per block)
    bitmap_size = (HEAP_SIZE / BLOCK_SIZE) / 8;
    heap_start += bitmap_size;  // Reserve space for bitmap
    
    // Clear bitmap
    for (size_t i = 0; i < bitmap_size; i++) {
        bitmap[i] = 0;
    }
    
    write_serial_string("Memory management initialized\n");
}

static size_t find_free_blocks(size_t num_blocks) {
    size_t current_block = 0;
    size_t consecutive_blocks = 0;
    
    while (current_block < (HEAP_SIZE / BLOCK_SIZE)) {
        size_t byte_index = current_block / 32;
        size_t bit_index = current_block % 32;
        
        if (!(bitmap[byte_index] & (1 << bit_index))) {
            consecutive_blocks++;
            if (consecutive_blocks == num_blocks) {
                return current_block - num_blocks + 1;
            }
        } else {
            consecutive_blocks = 0;
        }
        current_block++;
    }
    
    return (size_t)-1;  // No free blocks found
}

static void mark_blocks(size_t start_block, size_t num_blocks, int used) {
    for (size_t i = 0; i < num_blocks; i++) {
        size_t current_block = start_block + i;
        size_t byte_index = current_block / 32;
        size_t bit_index = current_block % 32;
        
        if (used) {
            bitmap[byte_index] |= (1 << bit_index);
        } else {
            bitmap[byte_index] &= ~(1 << bit_index);
        }
    }
}

void* kmalloc(size_t size) {
    if (size == 0) return NULL;
    
    // Calculate number of blocks needed
    size_t num_blocks = (size + BLOCK_SIZE - 1) / BLOCK_SIZE;
    
    // Find free blocks
    size_t start_block = find_free_blocks(num_blocks);
    if (start_block == (size_t)-1) {
        return NULL;  // Out of memory
    }
    
    // Mark blocks as used
    mark_blocks(start_block, num_blocks, 1);
    
    // Return pointer to allocated memory
    return (void*)(heap_start + (start_block * BLOCK_SIZE));
}

void kfree(void* ptr) {
    if (!ptr || ptr < (void*)heap_start || ptr >= (void*)heap_end) {
        return;
    }
    
    // Calculate block number
    size_t block = ((uint8_t*)ptr - heap_start) / BLOCK_SIZE;
    size_t num_blocks = 1;
    
    // Find number of blocks allocated
    while (block + num_blocks < (HEAP_SIZE / BLOCK_SIZE)) {
        size_t byte_index = (block + num_blocks) / 32;
        size_t bit_index = (block + num_blocks) % 32;
        
        if (!(bitmap[byte_index] & (1 << bit_index))) {
            break;
        }
        num_blocks++;
    }
    
    // Mark blocks as free
    mark_blocks(block, num_blocks, 0);
} 