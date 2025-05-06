#ifndef _FILESYSTEM_H
#define _FILESYSTEM_H

#include <stdint.h>
#include <stddef.h>

// Filesystem constants
#define MAX_FILES 256
#define MAX_FILENAME 64
#define MAX_FILE_SIZE 4096

// Filesystem functions
void fs_init(void);
int fs_create(const char *name);
int fs_write(int fd, const uint8_t *data, size_t size);
int fs_read(int fd, uint8_t *buffer, size_t size);
int fs_delete(int fd);
void fs_list(void);

// File information functions
int fs_find_by_name(const char *name);
const char *fs_get_name(int fd);
size_t fs_get_size(int fd);
int fs_exists(int fd);

#endif