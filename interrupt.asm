; Interrupt handling
global idt_load
global irq_common_stub
extern irq_handler

section .text

idt_load:
    mov edx, [esp + 4]
    lidt [edx]
    ret

; Common IRQ stub that calls the C handler
irq_common_stub:
    pusha           ; Pushes edi,esi,ebp,esp,ebx,edx,ecx,eax
    push ds
    push es
    push fs
    push gs
    
    mov ax, 0x10   ; Load the kernel data segment descriptor
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    
    push esp       ; Push pointer to regs struct as argument
    
    call irq_handler
    
    add esp, 4     ; Remove the argument
    
    pop gs
    pop fs
    pop es
    pop ds
    popa
    add esp, 8     ; Remove error code and interrupt number
    iret

; Generate IRQ handlers
%macro IRQ 2
global irq%1
irq%1:
    push byte 0    ; Push dummy error code
    push byte %2   ; Push interrupt number
    jmp irq_common_stub
%endmacro

; IRQs 0-15 map to interrupts 32-47
IRQ   0,    32  ; Timer
IRQ   1,    33  ; Keyboard
IRQ   2,    34  ; Cascade for IRQ 8-15
IRQ   3,    35  ; COM2
IRQ   4,    36  ; COM1
IRQ   5,    37  ; LPT2
IRQ   6,    38  ; Floppy
IRQ   7,    39  ; LPT1
IRQ   8,    40  ; RTC
IRQ   9,    41  ; Free
IRQ  10,    42  ; Free
IRQ  11,    43  ; Free
IRQ  12,    44  ; PS/2 Mouse
IRQ  13,    45  ; FPU
IRQ  14,    46  ; IDE Primary
IRQ  15,    47  ; IDE Secondary 