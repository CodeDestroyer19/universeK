# Compiler and linker settings
CC = gcc
AS = nasm
LD = ld
CFLAGS = -m32 -nostdlib -nostdinc -fno-builtin -fno-stack-protector -nostartfiles -nodefaultlibs -Wall -Wextra -Werror -I./include
ASFLAGS = -f elf32
LDFLAGS = -m elf_i386 -T link.ld

# Source files
ASM_SOURCES = boot.asm src/interrupts/interrupt_asm.asm
C_SOURCES = kernel.c \
           src/io/ports.c \
           src/debug/debug.c \
           src/interrupts/interrupt.c \
           src/interrupts/pic.c \
           src/interrupts/irq.c \
           src/interrupts/timer.c \
           src/drivers/keyboard/keyboard.c \
           src/drivers/mouse.c \
           src/drivers/cursor.c \
           src/drivers/vga/vga.c \
           src/drivers/driver.c \
           src/memory/memory.c \
           src/memory/malloc.c \
           src/fs/fs.c \
           src/terminal.c \
           src/window.c \
           src/libc/string.c

# Object files
OBJECTS = $(ASM_SOURCES:.asm=.o) $(C_SOURCES:.c=.o)

# Output files
KERNEL = kernel.bin
ISO = kernel.iso

# Targets
all: $(ISO)

$(KERNEL): $(OBJECTS)
	$(LD) $(LDFLAGS) -o $@ $(OBJECTS)

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

%.o: %.asm
	$(AS) $(ASFLAGS) $< -o $@

$(ISO): $(KERNEL)
	mkdir -p isodir/boot/grub
	cp $(KERNEL) isodir/boot/
	cp grub.cfg isodir/boot/grub/
	grub-mkrescue -o $(ISO) isodir

clean:
	rm -rf $(OBJECTS) $(KERNEL) isodir $(ISO)

.PHONY: all clean 