#include <stdint.h>
#include <stddef.h>
#include "memory.h"
#include "filesystem.h"
#include "string.h"

#define MAX_FILES 256
#define MAX_FILENAME 64
#define MAX_FILE_SIZE 4096

struct file {
    char name[MAX_FILENAME];
    uint8_t* data;
    size_t size;
    int used;
};

static struct file files[MAX_FILES];
extern void write_serial_string(const char* str);

void fs_init(void) {
    write_serial_string("Initializing filesystem...\n");
    
    // Initialize file table
    for (int i = 0; i < MAX_FILES; i++) {
        files[i].used = 0;
        files[i].data = NULL;
        files[i].size = 0;
    }
    
    write_serial_string("Filesystem initialized\n");
}

int fs_find_by_name(const char* name) {
    for (int i = 0; i < MAX_FILES; i++) {
        if (files[i].used && strcmp(files[i].name, name) == 0) {
            return i;
        }
    }
    return -1;
}

const char* fs_get_name(int fd) {
    if (fd < 0 || fd >= MAX_FILES || !files[fd].used) {
        return NULL;
    }
    return files[fd].name;
}

size_t fs_get_size(int fd) {
    if (fd < 0 || fd >= MAX_FILES || !files[fd].used) {
        return 0;
    }
    return files[fd].size;
}

int fs_exists(int fd) {
    return (fd >= 0 && fd < MAX_FILES && files[fd].used);
}

int fs_create(const char* name) {
    // Check if file already exists
    if (fs_find_by_name(name) >= 0) {
        return -1;  // File already exists
    }
    
    // Find free file entry
    int index = -1;
    for (int i = 0; i < MAX_FILES; i++) {
        if (!files[i].used) {
            index = i;
            break;
        }
    }
    
    if (index == -1) {
        return -1;  // No free file slots
    }
    
    // Copy filename
    size_t name_len = 0;
    while (name[name_len] && name_len < MAX_FILENAME - 1) {
        files[index].name[name_len] = name[name_len];
        name_len++;
    }
    files[index].name[name_len] = '\0';
    
    files[index].used = 1;
    files[index].size = 0;
    files[index].data = NULL;
    
    return index;
}

int fs_write(int fd, const uint8_t* data, size_t size) {
    if (fd < 0 || fd >= MAX_FILES || !files[fd].used) {
        return -1;
    }
    
    if (size > MAX_FILE_SIZE) {
        return -1;
    }
    
    // Allocate or reallocate memory
    if (files[fd].data) {
        kfree(files[fd].data);
    }
    
    files[fd].data = kmalloc(size);
    if (!files[fd].data) {
        return -1;
    }
    
    // Copy data
    memcpy(files[fd].data, data, size);
    files[fd].size = size;
    
    return size;
}

int fs_read(int fd, uint8_t* buffer, size_t size) {
    if (fd < 0 || fd >= MAX_FILES || !files[fd].used || !files[fd].data) {
        return -1;
    }
    
    size_t read_size = size;
    if (read_size > files[fd].size) {
        read_size = files[fd].size;
    }
    
    // Copy data
    memcpy(buffer, files[fd].data, read_size);
    
    return read_size;
}

int fs_delete(int fd) {
    if (fd < 0 || fd >= MAX_FILES || !files[fd].used) {
        return -1;
    }
    
    if (files[fd].data) {
        kfree(files[fd].data);
    }
    
    files[fd].used = 0;
    files[fd].size = 0;
    files[fd].data = NULL;
    files[fd].name[0] = '\0';
    
    return 0;
}

void fs_list(void) {
    int found = 0;
    write_serial_string("File listing:\n");
    
    for (int i = 0; i < MAX_FILES; i++) {
        if (files[i].used) {
            found = 1;
            write_serial_string(files[i].name);
            write_serial_string(" (");
            // Convert size to string - simple implementation
            char size_str[32];
            size_t size = files[i].size;
            int pos = 0;
            
            if (size == 0) {
                size_str[pos++] = '0';
            } else {
                while (size > 0) {
                    size_str[pos++] = '0' + (size % 10);
                    size /= 10;
                }
            }
            size_str[pos] = '\0';
            
            // Reverse the string
            for (int j = 0; j < pos / 2; j++) {
                char temp = size_str[j];
                size_str[j] = size_str[pos - 1 - j];
                size_str[pos - 1 - j] = temp;
            }
            
            write_serial_string(size_str);
            write_serial_string(" bytes)\n");
        }
    }
    
    if (!found) {
        write_serial_string("No files found\n");
    }
} 