// Stable-Rust interrupt entry. Returning handlers touch no SIMD/FPU state and
// preserve every GPR they use. Hardware's saved RFLAGS are restored by iretq.
.text
.macro exception number, error=0
.p2align 4
.global cathedral_exception_\number
cathedral_exception_\number:
    .if !\error
    push 0
    .endif
    push \number
    jmp cathedral_fatal_entry
.endm

exception 0
exception 1
exception 2
// Breakpoint is a returning self-test path, with no hardware error code.
.global cathedral_exception_3
cathedral_exception_3:
    lock inc qword ptr [rip + {breakpoints}]
    iretq
exception 4
exception 5
exception 6
exception 7
exception 8, 1
exception 9
exception 10, 1
exception 11, 1
exception 12, 1
exception 13, 1
exception 14, 1
exception 15
exception 16
exception 17, 1
exception 18
exception 19
exception 20
exception 21, 1
exception 22
exception 23
exception 24
exception 25
exception 26
exception 27
exception 28
exception 29, 1
exception 30, 1
exception 31
exception 255

cathedral_fatal_entry:
    cld
    mov rcx, [rsp]
    mov rdx, [rsp + 8]
    mov r8, [rsp + 16]
    mov r9, cr2
    and rsp, -16
    sub rsp, 32
    call {fault}
    ud2

.global cathedral_timer_entry
cathedral_timer_entry:
    mov qword ptr [rip + {irq_stack}], rsp
    lock inc qword ptr [rip + {ticks}]
    push rax
    mov al, 0x20
    out 0x20, al
    pop rax
    iretq
