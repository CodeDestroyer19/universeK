/* Entry point */
ENTRY(_start)

/* Define the program headers we want in our final ELF executable */
PHDRS
{
    text    PT_LOAD FLAGS(5);          /* Read + Execute */
    rodata  PT_LOAD FLAGS(4);          /* Read only */
    data    PT_LOAD FLAGS(6);          /* Read + Write */
}

SECTIONS
{
    /* Start at 1MB - standard for kernels */
    . = 1M;

    /* Multiboot header first */
    .text BLOCK(4K) : ALIGN(4K)
    {
        *(.multiboot)
        *(.text)
    } :text

    /* Read-only data */
    .rodata BLOCK(4K) : ALIGN(4K)
    {
        *(.rodata)
    } :rodata

    /* Read-write data (initialized) */
    .data BLOCK(4K) : ALIGN(4K)
    {
        *(.data)
    } :data

    /* Read-write data (uninitialized) and stack */
    .bss BLOCK(4K) : ALIGN(4K)
    {
        *(COMMON)
        *(.bss)
        *(.bootstrap_stack)
    } :data

    /* Heap section */
    .heap BLOCK(4K) : ALIGN(4K)
    {
        _heap_start = .;
        . = . + 4M;    /* 4MB heap */
        _heap_end = .;
    } :data

    /* Remove unnecessary sections */
    /DISCARD/ :
    {
        *(.comment)
        *(.eh_frame)
        *(.note.gnu.build-id)
    }
} 