.section .text._start,"ax"
.global _start
.type _start,@function
_start:
    // Kernel supplies RDI/RSI and aligned RSP. CALL establishes SysV entry alignment.
    xor ebp, ebp
    cld
    call cathedral_user_main
    ud2
.size _start, . - _start
