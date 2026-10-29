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
    .quad cathedral_memory_allocate
    .quad cathedral_memory_share
    .quad cathedral_memory_release
    .quad cathedral_shared_read
    .quad cathedral_shared_write
    .quad cathedral_shared_send
    .quad cathedral_shared_receive
    .quad cathedral_notify_create
    .quad cathedral_notify_drop
    .quad cathedral_notify_signal
    .quad cathedral_notify_wait
    .quad cathedral_notify_control
    .quad cathedral_notify_send
    .quad cathedral_notify_receive
    .quad cathedral_bootstrap

.section .text.cathedral_kernel_gate,"ax",@progbits
cathedral_yield:
    mov eax, 1                       // YIELD
    xor edi, edi
    xor esi, esi
    xor edx, edx
    int 0x80
    ret
cathedral_bootstrap:
    mov eax, 3                       // IPC_HANDLE
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

cathedral_memory_allocate:
    mov eax, 27                      // MEMORY_ALLOCATE
    xor esi, esi
    xor edx, edx
    int 0x80
    ret
cathedral_memory_share:
    mov eax, 82                      // MEMORY_SHARE_MUTABLE
    xor esi, esi
    xor edx, edx
    int 0x80
    ret
cathedral_memory_release:
    mov eax, 29                      // MEMORY_RELEASE
    xor esi, esi
    xor edx, edx
    int 0x80
    ret

// Borrow only the private destination/source array, never the shared mapping.
// Authenticate the current handle and full word span on EVERY call. No retained
// address can outlive release or alias a reused mapping slot. INT 80 preserves
// every register except RAX. The current kernel has one execution lane per task.
// These checked calls are deliberately a first adapter, not a fast mapping API.
.macro shared_words write
    mov r8, rdi                     // local memory authority
    mov r9, rsi                     // first word
    mov r10, rdx                    // borrowed private array
    mov r11, rcx                    // word count
    test r11, r11
    jz .Linvalid\@
    cmp r11, 8
    ja .Linvalid\@
    .if \write
        // Only an owner can return BUSY (already shared OR frozen). MAP_SHARED
        // below additionally rejects frozen objects. A reader returns DENIED.
        mov eax, 82                  // MEMORY_SHARE_MUTABLE
        xor esi, esi
        xor edx, edx
        int 0x80
        test rax, rax
        jz .Lowner\@
        cmp rax, -16
        jne .Ldone\@
    .Lowner\@:
    .endif
    mov eax, 32                      // MEMORY_PAGES
    mov rdi, r8
    xor esi, esi
    xor edx, edx
    int 0x80
    test rax, rax
    js .Ldone\@
    cmp rax, 256
    ja .Linvalid\@
    shl rax, 9                      // 512 words/page
    mov rcx, r9
    add rcx, r11
    jc .Linvalid\@
    cmp rcx, rax
    ja .Linvalid\@
    mov eax, 83                      // MEMORY_MAP_SHARED
    mov rdi, r8
    xor esi, esi
    xor edx, edx
    int 0x80
    test rax, rax
    js .Ldone\@
    lea rax, [rax + r9*8]
    xor ecx, ecx
.Lword\@:
    .if \write
        mov rdx, qword ptr [r10 + rcx*8]
        mov qword ptr [rax + rcx*8], rdx
    .else
        mov rdx, qword ptr [rax + rcx*8]
        mov qword ptr [r10 + rcx*8], rdx
    .endif
    inc rcx
    cmp rcx, r11
    jne .Lword\@
    // Aligned x86 MOV gives word atomicity and acquire/release ordering.
    // No atomic multiword snapshot is promised; the protocol checks publication.
    xor eax, eax
    ret
.Linvalid\@:
    mov rax, -22
.Ldone\@:
    ret
.endm
cathedral_shared_read:
    shared_words 0
cathedral_shared_write:
    shared_words 1

// Typed handoffs use length + handle + 64 payload bytes. An aligned 128-byte
// scratch keeps the entire 80-byte packet within one page, including its tail.
.macro stage_packet
    push rbp
    mov rbp, rsp
    push r12
    sub rsp, 256
    and rsp, -128
    mov r12, rsi
    mov r8, rdi
    mov rdi, rsp
    mov ecx, 10
    rep movsq
.endm
.macro finish_packet output
    .if \output
        mov rsi, rsp
        mov rdi, r12
        mov ecx, 10
        rep movsq
    .endif
    mov r12, qword ptr [rbp - 8]
    mov rsp, rbp
    pop rbp
    ret
.endm
cathedral_shared_send:
    stage_packet
    mov eax, 84                      // IPC_SEND_SHARED
    mov rdi, r8
    mov rsi, rsp
    xor edx, edx
    int 0x80
    finish_packet 0
cathedral_shared_receive:
    stage_packet
    mov eax, 85                      // IPC_RECEIVE_SHARED
    mov rdi, r8
    mov rsi, rsp
    xor edx, edx
    int 0x80
    finish_packet 1

cathedral_notify_create:
    stage_buffer
    mov eax, 73                      // NOTIFY_CREATE
    mov rdi, rsp
    xor esi, esi
    xor edx, edx
    int 0x80
    finish_buffer 1
cathedral_notify_drop:
    mov eax, 74                      // NOTIFY_DROP
    xor esi, esi
    xor edx, edx
    int 0x80
    ret
cathedral_notify_signal:
    mov eax, 75                      // NOTIFY_SIGNAL
    xor esi, esi
    xor edx, edx
    int 0x80
    ret
cathedral_notify_wait:
    mov eax, 76                      // NOTIFY_WAIT
    int 0x80
    ret
cathedral_notify_control:
    mov eax, 78                      // NOTIFY_CONTROL
    xor edx, edx
    int 0x80
    ret
cathedral_notify_send:
    stage_packet
    mov eax, 79                      // NOTIFY_SEND
    mov rdi, r8
    mov rsi, rsp
    xor edx, edx
    int 0x80
    finish_packet 0
cathedral_notify_receive:
    stage_packet
    mov eax, 80                      // NOTIFY_RECEIVE
    mov rdi, r8
    mov rsi, rsp
    xor edx, edx
    int 0x80
    finish_packet 1

.section .note.GNU-stack,"",@progbits
