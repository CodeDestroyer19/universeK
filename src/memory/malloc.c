#include "memory/malloc.h"
#include "debug/debug.h"
#include <string.h>

// Memory block header
typedef struct block_header {
    size_t size;
    bool used;
    struct block_header* next;
} block_header_t;

// Start of heap
#define HEAP_START 0x400000
#define HEAP_SIZE  0x400000  // 4MB heap

static block_header_t* heap_start = (block_header_t*)HEAP_START;
static bool initialized = false;

// Initialize the heap
static void init_heap(void) {
    if (initialized) return;
    
    heap_start->size = HEAP_SIZE - sizeof(block_header_t);
    heap_start->used = false;
    heap_start->next = NULL;
    
    initialized = true;
    DEBUG_INFO("MALLOC", "Heap initialized at 0x%x, size %d bytes", HEAP_START, HEAP_SIZE);
}

void* malloc(size_t size) {
    if (!initialized) init_heap();
    
    // Align size to 8 bytes
    size = (size + 7) & ~7;
    
    block_header_t* current = heap_start;
    
    while (current) {
        if (!current->used && current->size >= size) {
            // Split block if it's too large
            if (current->size > size + sizeof(block_header_t) + 8) {
                block_header_t* new_block = (block_header_t*)((char*)current + sizeof(block_header_t) + size);
                new_block->size = current->size - size - sizeof(block_header_t);
                new_block->used = false;
                new_block->next = current->next;
                
                current->size = size;
                current->next = new_block;
            }
            
            current->used = true;
            DEBUG_TRACE("MALLOC", "Allocated %d bytes at 0x%x", size, (char*)current + sizeof(block_header_t));
            return (char*)current + sizeof(block_header_t);
        }
        current = current->next;
    }
    
    DEBUG_ERROR("MALLOC", "Out of memory: failed to allocate %d bytes", size);
    return NULL;
}

void free(void* ptr) {
    if (!ptr) return;
    
    block_header_t* header = (block_header_t*)((char*)ptr - sizeof(block_header_t));
    header->used = false;
    
    // Coalesce with next block if it's free
    if (header->next && !header->next->used) {
        header->size += sizeof(block_header_t) + header->next->size;
        header->next = header->next->next;
    }
    
    // Find previous block to coalesce
    block_header_t* current = heap_start;
    while (current && current->next != header) {
        current = current->next;
    }
    
    // Coalesce with previous block if it's free
    if (current && !current->used) {
        current->size += sizeof(block_header_t) + header->size;
        current->next = header->next;
    }
    
    DEBUG_TRACE("MALLOC", "Freed memory at 0x%x", ptr);
}

void* calloc(size_t num, size_t size) {
    size_t total = num * size;
    void* ptr = malloc(total);
    if (ptr) {
        memset(ptr, 0, total);
    }
    return ptr;
}

void* realloc(void* ptr, size_t size) {
    if (!ptr) return malloc(size);
    if (size == 0) {
        free(ptr);
        return NULL;
    }
    
    block_header_t* header = (block_header_t*)((char*)ptr - sizeof(block_header_t));
    if (header->size >= size) return ptr;
    
    void* new_ptr = malloc(size);
    if (!new_ptr) return NULL;
    
    memcpy(new_ptr, ptr, header->size);
    free(ptr);
    
    return new_ptr;
} 