; Multiboot header
MBALIGN     equ 1 << 0
MEMINFO     equ 1 << 1
FLAGS       equ MBALIGN | MEMINFO
MAGIC       equ 0x1BADB002
CHECKSUM    equ -(MAGIC + FLAGS)

section .multiboot
align 4
    dd MAGIC
    dd FLAGS
    dd CHECKSUM

section .bss
align 16
stack_bottom:
    resb 16384 ; 16 KiB
stack_top:

section .text
global _start
extern kernel_main

_start:
    mov esp, stack_top    ; Set up stack
    
    push ebx             ; Push multiboot info pointer
    push eax             ; Push multiboot magic number
    
    cli                  ; Disable interrupts
    call kernel_main     ; Call kernel
    
.hang:
    cli                  ; Disable interrupts again (paranoid)
    hlt                  ; Halt the CPU
    jmp .hang           ; If that didn't work, loop 