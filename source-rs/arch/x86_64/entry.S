// Stable-Rust interrupt entry. Scheduler paths preserve complete qemu64 state.
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
    mov eax, 1
    jmp cathedral_switch_entry

.global cathedral_yield_entry
cathedral_yield_entry:
    push rax
    xor eax, eax

cathedral_switch_entry:
    push rbx
    push rcx
    push rdx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    mov edx, eax
    sub rsp, 512
    // FXSAVE leaves reserved bytes untouched: initialize them before Rust copies
    // the frame. GPRs are already saved; these instructions preserve FP state.
    cld
    mov rdi, rsp
    xor eax, eax
    mov ecx, 64
    rep stosq
    fxsave64 [rsp]
    test edx, edx
    jz 1f
    mov al, 0x20
    out 0x20, al
1:
    mov rcx, rsp
    sub rsp, 32
    call {switch}
    // The callback copied out of the shared IST and selected a permanent context.
    mov rsp, rax
    fxrstor64 [rsp]
    add rsp, 512
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rbp
    pop rdx
    pop rcx
    pop rbx
    pop rax
    iretq
