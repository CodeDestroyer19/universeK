[bits 16]                       ; Specify 16-bit code
[org 0x7C00]                    ; BIOS loads bootloader at this address

; Constants
STAGE2_SEGMENT equ 0x1000       ; Segment where we'll load stage 2
STAGE2_OFFSET  equ 0x0000       ; Offset within segment
STAGE2_SECTORS equ 6            ; Number of sectors to load

start:
    cli                         ; Disable interrupts
    xor ax, ax                 ; Zero AX register
    mov ds, ax                 ; Set Data Segment to 0
    mov es, ax                 ; Set Extra Segment to 0
    mov ss, ax                 ; Set Stack Segment to 0
    mov sp, 0x7C00            ; Set Stack Pointer to bootloader start
    sti                         ; Enable interrupts

    ; Print initialization message
    mov si, init_msg
    call print_string

    ; Save boot drive number
    mov [boot_drive], dl
    mov si, drive_msg
    call print_string
    mov al, dl
    call print_hex_byte

    ; Print welcome message
    mov si, welcome_msg
    call print_string

    ; Print loading stage2 message
    mov si, loading_stage2_msg
    call print_string

    ; Load stage 2
    mov ax, STAGE2_SEGMENT     ; Set up ES:BX to point to where we want to load
    mov es, ax
    mov bx, STAGE2_OFFSET

    ; Read from disk
    mov ah, 0x02              ; BIOS read sector function
    mov al, STAGE2_SECTORS    ; Number of sectors to read
    mov ch, 0                 ; Cylinder number
    mov cl, 2                 ; Sector number (1-based, sector 1 is bootloader)
    mov dh, 0                 ; Head number
    mov dl, [boot_drive]      ; Drive number
    int 0x13                  ; BIOS interrupt

    jc disk_error             ; Jump if error (carry flag set)

    ; Compare number of sectors actually read
    cmp al, STAGE2_SECTORS
    jne sectors_error

    ; Print success message
    mov si, stage2_loaded_msg
    call print_string
    
    ; Print sectors loaded
    mov si, sectors_loaded_msg
    call print_string
    mov al, STAGE2_SECTORS
    call print_hex_byte
    
    ; Print jumping message
    mov si, jumping_msg
    call print_string

    ; Jump to stage 2
    mov dl, [boot_drive]      ; Pass boot drive to stage 2
    jmp STAGE2_SEGMENT:STAGE2_OFFSET

disk_error:
    mov si, disk_error_msg
    call print_string
    ; Print error code
    mov si, error_code_msg
    call print_string
    mov al, ah        ; Error code is in AH after int 0x13
    call print_hex_byte
    jmp $             ; Hang

sectors_error:
    mov si, sectors_error_msg
    call print_string
    mov si, expected_msg
    call print_string
    mov al, STAGE2_SECTORS
    call print_hex_byte
    mov si, actual_msg
    call print_string
    ; AL already contains the actual sectors read
    call print_hex_byte
    jmp $             ; Hang

; Print null-terminated string pointed to by SI
print_string:
    pusha
.loop:
    lodsb                     ; Load byte at SI into AL
    test al, al              ; Check if character is 0 (end of string)
    jz .done                 ; If zero, exit routine
    mov ah, 0x0E             ; BIOS teletype function
    int 0x10                 ; BIOS interrupt
    jmp .loop
.done:
    popa
    ret

; Print byte in AL as hex
print_hex_byte:
    pusha
    mov cx, 2          ; Two hex digits for a byte
    mov ah, 0          ; Zero out AH
.digit_loop:
    rol ax, 4          ; Rotate left to get high nibble first
    mov bx, ax         ; Save rotated value
    and al, 0x0F       ; Mask off low 4 bits (current digit)
    cmp al, 10         ; Check if 0-9 or A-F
    jl .print_digit
    add al, 'A' - 10 - '0'  ; Convert to A-F
.print_digit:
    add al, '0'        ; Convert to ASCII
    mov ah, 0x0E       ; BIOS teletype
    int 0x10           ; Print character
    mov ax, bx         ; Restore rotated value
    loop .digit_loop   ; Next digit
    popa
    ret

; Data
init_msg:         db 'Initializing bootloader...', 13, 10, 0
drive_msg:        db 'Boot drive: 0x', 0
welcome_msg:      db 'BearOS Bootloader Stage 1', 13, 10, 0
loading_stage2_msg: db 'Loading Stage 2...', 13, 10, 0
stage2_loaded_msg: db 'Stage 2 loaded successfully!', 13, 10, 0
sectors_loaded_msg: db 'Sectors loaded: 0x', 0
jumping_msg:      db 'Jumping to Stage 2...', 13, 10, 0
disk_error_msg:   db 'Disk read error!', 13, 10, 0
error_code_msg:   db 'Error code: 0x', 0
sectors_error_msg: db 'Sector count mismatch!', 13, 10, 0
expected_msg:     db 'Expected: 0x', 0
actual_msg:       db ' Actual: 0x', 0
boot_drive:       db 0

; Padding and magic number
times 510-($-$$) db 0        ; Fill remaining bytes with 0
dw 0xAA55                    ; Boot signature 