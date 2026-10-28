// SysV entry into the compiled Omega Program export. The kernel supplies a
// 16-byte-aligned stack with no return address. No Rust startup or allocator.
.intel_syntax noprefix
.section .text._start,"ax",@progbits
.global _start
_start:
    xor ebp, ebp
    cld
    mov rdx, rsi
    mov rsi, rdi
    lea rdi, [rip + cathedral_kernel_gate]
    call omega_app_step
    mov rdi, rax
    mov eax, 2                       // EXIT
    int 0x80
    ud2

.section .rodata.cathedral_kernel_gate,"a",@progbits
.balign 8
cathedral_kernel_gate:
    .quad cathedral_yield
    .quad cathedral_environment
    .quad cathedral_channel
    .quad cathedral_send
    .quad cathedral_receive
    .quad cathedral_close
    .quad cathedral_write
    .quad cathedral_clock
    .quad cathedral_clock_read
    .quad cathedral_wait

.section .text.cathedral_kernel_gate,"ax",@progbits
cathedral_yield:
    mov eax, 1                       // YIELD
    xor edi, edi
    xor esi, esi
    xor edx, edx
    int 0x80
    ret
cathedral_close:
    mov eax, 35                      // IPC_CLOSE
    xor esi, esi
    xor edx, edx
    int 0x80
    ret
cathedral_clock:
    mov eax, 15                      // CLOCK_HANDLE
    xor edi, edi
    xor esi, esi
    xor edx, edx
    int 0x80
    ret

// Stage exactly one borrowed 64-byte array into page-contained scratch. The
// pointer never escapes this call. Read-only inputs are never written back.
// r8 = handle, r9 = length; preserve SysV callee-saved registers and the original
// stack even if 64-byte alignment consumes another 63 bytes.
.macro stage_buffer
    push rbp
    mov rbp, rsp
    push r12
    sub rsp, 128
    and rsp, -64
    mov r12, rdi
    mov rsi, rdi
    mov rdi, rsp
    mov ecx, 8
    rep movsq
.endm
.macro finish_buffer output
    .if \output
        mov rsi, rsp
        mov rdi, r12
        mov ecx, 8
        rep movsq
    .endif
    mov r12, qword ptr [rbp - 8]
    mov rsp, rbp
    pop rbp
    ret
.endm

cathedral_environment:
    stage_buffer
    mov eax, 39                      // TASK_ENVIRONMENT
    mov rdi, rsp
    xor esi, esi
    xor edx, edx
    int 0x80
    finish_buffer 1

cathedral_channel:
    stage_buffer
    mov eax, 34                      // IPC_CREATE
    mov rdi, rsp
    xor esi, esi
    xor edx, edx
    int 0x80
    finish_buffer 1

cathedral_send:
    mov r8, rdi
    mov r9, rdx
    mov rdi, rsi
    stage_buffer
    mov eax, 4                       // IPC_SEND
    mov rdi, r8
    mov rsi, rsp
    mov rdx, r9
    int 0x80
    finish_buffer 0

cathedral_receive:
    mov r8, rdi
    mov rdi, rsi
    stage_buffer
    mov eax, 5                       // IPC_RECEIVE
    mov rdi, r8
    mov rsi, rsp
    mov edx, 64
    int 0x80
    finish_buffer 1

cathedral_write:
    mov r9, rsi
    stage_buffer
    xor eax, eax                     // WRITE
    mov rdi, rsp
    mov rsi, r9
    xor edx, edx
    int 0x80
    finish_buffer 0

cathedral_clock_read:
    mov r8, rdi
    mov rdi, rsi
    stage_buffer
    mov eax, 12                      // CLOCK_READ
    mov rdi, r8
    mov rsi, rsp
    mov edx, 8
    int 0x80
    finish_buffer 1

cathedral_wait:
    mov r8, rsi
    mov r9, rdx
    stage_buffer
    mov eax, 68                      // EVENT_WAIT
    mov rdi, rsp
    mov rsi, r8
    mov rdx, r9
    int 0x80
    finish_buffer 0

.section .note.GNU-stack,"",@progbits
