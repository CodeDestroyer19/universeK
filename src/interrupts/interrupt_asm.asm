; Interrupt handling assembly code
[BITS 32]

; External C functions
extern interrupt_handler

; Global assembly functions
global idt_load
global isr_common_stub

; Export IRQ handlers
%assign i 0
%rep 16
    global irq%+i
%assign i i+1
%endrep

; ISR handlers
%macro ISR_NOERR 1
global isr%1
isr%1:
    push dword 0     ; Push dummy error code
    push dword %1    ; Push interrupt number
    jmp isr_common_stub
%endmacro

%macro ISR_ERR 1
global isr%1
isr%1:
    push dword %1    ; Push interrupt number (error code already pushed)
    jmp isr_common_stub
%endmacro

; IRQ handlers
%macro IRQ 2
global irq%1
irq%1:
    push dword 0     ; Push dummy error code
    push dword %2    ; Push interrupt number (32 + IRQ number)
    jmp isr_common_stub
%endmacro

; Load IDT
idt_load:
    mov eax, [esp + 4]  ; Get pointer to IDT
    lidt [eax]          ; Load IDT
    ret

; Common ISR stub
isr_common_stub:
    ; Save registers
    pusha
    push ds
    push es
    push fs
    push gs
    
    ; Load kernel data segment
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    
    ; Call C handler
    push esp            ; Push pointer to registers
    call interrupt_handler
    add esp, 4          ; Remove argument
    
    ; Restore registers
    pop gs
    pop fs
    pop es
    pop ds
    popa
    
    ; Remove error code and interrupt number
    add esp, 8
    
    ; Return from interrupt
    iret

; CPU exceptions
ISR_NOERR 0    ; Divide by zero
ISR_NOERR 1    ; Debug
ISR_NOERR 2    ; Non-maskable interrupt
ISR_NOERR 3    ; Breakpoint
ISR_NOERR 4    ; Overflow
ISR_NOERR 5    ; Bound range exceeded
ISR_NOERR 6    ; Invalid opcode
ISR_NOERR 7    ; Device not available
ISR_ERR   8    ; Double fault
ISR_NOERR 9    ; Coprocessor segment overrun
ISR_ERR   10   ; Invalid TSS
ISR_ERR   11   ; Segment not present
ISR_ERR   12   ; Stack-segment fault
ISR_ERR   13   ; General protection fault
ISR_ERR   14   ; Page fault
ISR_NOERR 15   ; Reserved
ISR_NOERR 16   ; x87 FPU error
ISR_ERR   17   ; Alignment check
ISR_NOERR 18   ; Machine check
ISR_NOERR 19   ; SIMD floating-point
ISR_NOERR 20   ; Virtualization
ISR_ERR   21   ; Control protection
ISR_NOERR 22   ; Reserved
ISR_NOERR 23   ; Reserved
ISR_NOERR 24   ; Reserved
ISR_NOERR 25   ; Reserved
ISR_NOERR 26   ; Reserved
ISR_NOERR 27   ; Reserved
ISR_NOERR 28   ; Reserved
ISR_NOERR 29   ; Reserved
ISR_ERR   30   ; Security
ISR_NOERR 31   ; Reserved

; IRQs
IRQ 0, 32      ; Timer
IRQ 1, 33      ; Keyboard
IRQ 2, 34      ; Cascade for IRQ 8-15
IRQ 3, 35      ; COM2
IRQ 4, 36      ; COM1
IRQ 5, 37      ; LPT2
IRQ 6, 38      ; Floppy
IRQ 7, 39      ; LPT1
IRQ 8, 40      ; RTC
IRQ 9, 41      ; Free
IRQ 10, 42     ; Free
IRQ 11, 43     ; Free
IRQ 12, 44     ; PS/2 Mouse
IRQ 13, 45     ; FPU
IRQ 14, 46     ; Primary ATA
IRQ 15, 47     ; Secondary ATA 