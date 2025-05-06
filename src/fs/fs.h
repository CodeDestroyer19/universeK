#ifndef FS_H
#define FS_H

#include <stdint.h>
#include <stddef.h>

void fs_init(void);
int fs_create(const char *name);
int fs_delete(int fd);
int fs_write(int fd, const uint8_t *data, size_t size);
int fs_read(int fd, uint8_t *data, size_t size);
void fs_list(void);

#endif /* FS_H */