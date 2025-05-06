#include "fs.h"
#include "debug/debug.h"
#include "string.h"
#include "memory/malloc.h"

#define MAX_FILES 256
#define MAX_FILE_SIZE 4096

typedef struct {
    char name[32];
    uint8_t* data;
    size_t size;
    bool used;
} file_t;

static file_t files[MAX_FILES];

void fs_init(void) {
    DEBUG_INFO("FS", "Initializing filesystem");
    memset(files, 0, sizeof(files));
    DEBUG_INFO("FS", "Filesystem initialized");
}

int fs_create(const char* name) {
    // Find free slot
    for (int i = 0; i < MAX_FILES; i++) {
        if (!files[i].used) {
            strncpy(files[i].name, name, sizeof(files[i].name) - 1);
            files[i].data = malloc(MAX_FILE_SIZE);
            if (!files[i].data) {
                DEBUG_ERROR("FS", "Failed to allocate file data");
                return -1;
            }
            files[i].size = 0;
            files[i].used = true;
            DEBUG_INFO("FS", "Created file: %s", name);
            return i;
        }
    }
    DEBUG_ERROR("FS", "No free file slots");
    return -1;
}

int fs_delete(int fd) {
    if (fd < 0 || fd >= MAX_FILES || !files[fd].used) {
        return -1;
    }
    
    free(files[fd].data);
    memset(&files[fd], 0, sizeof(file_t));
    DEBUG_INFO("FS", "Deleted file: %s", files[fd].name);
    return 0;
}

int fs_write(int fd, const uint8_t* data, size_t size) {
    if (fd < 0 || fd >= MAX_FILES || !files[fd].used) {
        return -1;
    }
    
    if (size > MAX_FILE_SIZE) {
        size = MAX_FILE_SIZE;
    }
    
    memcpy(files[fd].data, data, size);
    files[fd].size = size;
    DEBUG_INFO("FS", "Wrote %d bytes to file: %s", size, files[fd].name);
    return size;
}

int fs_read(int fd, uint8_t* data, size_t size) {
    if (fd < 0 || fd >= MAX_FILES || !files[fd].used) {
        return -1;
    }
    
    if (size > files[fd].size) {
        size = files[fd].size;
    }
    
    memcpy(data, files[fd].data, size);
    DEBUG_INFO("FS", "Read %d bytes from file: %s", size, files[fd].name);
    return size;
}

void fs_list(void) {
    DEBUG_INFO("FS", "File listing:");
    for (int i = 0; i < MAX_FILES; i++) {
        if (files[i].used) {
            DEBUG_INFO("FS", "  %s (%d bytes)", files[i].name, files[i].size);
        }
    }
} 