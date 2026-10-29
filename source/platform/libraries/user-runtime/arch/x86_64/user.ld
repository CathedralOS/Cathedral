/* Reference kernel's admitted ELF64 layout, shared with the Rust runtime ABI. */
ENTRY(_start)
PHDRS {
    text PT_LOAD FLAGS(5);
    rodata PT_LOAD FLAGS(4);
    stack PT_GNU_STACK FLAGS(6);
}
SECTIONS {
    . = 0x8000000000;
    .text : { KEEP(*(.text._start)) *(.text .text.*) } :text
    . = ALIGN(4096);
    .rodata : { KEEP(*(.cathedral.imports)) *(.rodata .rodata.*) } :rodata
    /DISCARD/ : { *(.eh_frame*) *(.comment) *(.note*) }
}
