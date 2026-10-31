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
    .quad cathedral_endpoint_receive
    .quad cathedral_object_create
    .quad cathedral_object_send
    .quad cathedral_object_receive
    .quad cathedral_object_read
    .quad cathedral_private_write
    .quad cathedral_object_freeze
    .quad cathedral_wait_many
    .quad cathedral_launch_at
    .quad cathedral_launch_port
    .quad cathedral_task_connect
    .quad cathedral_spawn_tree
    .quad cathedral_task_cancel
    .quad cathedral_task_wait
    .quad cathedral_endpoint_send
    .quad cathedral_resource_budget
    .quad cathedral_notification_budget

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
.macro shared_words write, sealed=0
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
    .if \sealed
        mov eax, 60                  // MEMORY_MAP_OBJECT
    .else
        mov eax, 83                  // MEMORY_MAP_SHARED
    .endif
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
cathedral_object_read:
    shared_words 0, 1

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

cathedral_endpoint_receive:
    stage_packet
    mov eax, 37                      // IPC_RECEIVE_CAP
    mov rdi, r8
    mov rsi, rsp
    xor edx, edx
    int 0x80
    finish_packet 1

// Copy from one borrowed private array into fresh zeroed pages, then seal and
// unmap before returning its handle. No raw mapping escapes. Every failed phase
// releases the newly allocated object; caller data is never modified/retained.
cathedral_object_create:
    test rsi, rsi
    jz .Lobject_invalid
    cmp rsi, 16384
    ja .Lobject_invalid
    push r12
    push r13
    push r14
    mov r12, rdi                    // borrowed source
    mov r13, rsi                    // exact byte count
    lea rdi, [rsi + 4095]
    shr rdi, 12
    xor esi, esi
    xor edx, edx
    mov eax, 27                     // MEMORY_ALLOCATE
    int 0x80
    test rax, rax
    js .Lobject_done
    mov r14, rax                    // exclusively owned private handle
    mov rdi, r14
    mov eax, 28                     // MEMORY_ADDRESS
    int 0x80
    test rax, rax
    js .Lobject_release
    mov rdi, rax
    mov rsi, r12
    mov rcx, r13
    rep movsb
    mov rdi, r14
    xor esi, esi
    xor edx, edx
    mov eax, 59                     // MEMORY_FREEZE
    int 0x80
    test rax, rax
    jnz .Lobject_release
    mov eax, 61                     // MEMORY_UNMAP_OBJECT
    int 0x80
    test rax, rax
    jnz .Lobject_release
    mov rax, r14
    jmp .Lobject_done
.Lobject_release:
    mov r12, rax
    mov rdi, r14
    xor esi, esi
    xor edx, edx
    mov eax, 29                     // MEMORY_RELEASE
    int 0x80
    mov rax, r12
.Lobject_done:
    pop r14
    pop r13
    pop r12
    ret
.Lobject_invalid:
    mov rax, -22
    ret
cathedral_object_send:
    stage_packet
    mov eax, 56                     // IPC_SEND_MEMORY
    mov rdi, r8
    mov rsi, rsp
    xor edx, edx
    int 0x80
    finish_packet 0

cathedral_object_receive:
    stage_packet
    mov eax, 57                      // IPC_RECEIVE_MEMORY
    mov rdi, r8
    mov rsi, rsp
    xor edx, edx
    int 0x80
    finish_packet 1

// Copy one bounded chunk into a live private allocation. Reauthenticate mode
// and the entire physical span on every call; no mapped address escapes. The
// single execution lane prevents local release/seal between validation and copy.
cathedral_private_write:
    test rcx, rcx
    jz .Lprivate_invalid
    cmp rcx, 8192
    ja .Lprivate_invalid
    mov r8, rdi                      // private handle
    mov r9, rsi                      // byte offset
    mov r10, rdx                     // borrowed source
    mov r11, rcx                     // exact byte count
    xor esi, esi
    xor edx, edx
    mov eax, 32                      // MEMORY_PAGES
    int 0x80
    test rax, rax
    js .Lprivate_done
    cmp rax, 256
    ja .Lprivate_invalid
    shl rax, 12
    mov rcx, r9
    add rcx, r11
    jc .Lprivate_invalid
    cmp rcx, rax
    ja .Lprivate_invalid
    mov rdi, r8
    mov eax, 88                      // MEMORY_PRIVATE_ADDRESS
    int 0x80
    test rax, rax
    js .Lprivate_done
    lea rdi, [rax + r9]
    mov rsi, r10
    mov rcx, r11
    rep movsb
    xor eax, eax
    ret
.Lprivate_invalid:
    mov rax, -22
.Lprivate_done:
    ret

// Freeze first, then remove the caller's alias. Failure leaves custody with the
// caller, which must release even if the first phase already sealed the object.
cathedral_object_freeze:
    xor esi, esi
    xor edx, edx
    mov eax, 59                      // MEMORY_FREEZE
    int 0x80
    test rax, rax
    jnz .Lfreeze_done
    mov eax, 61                      // MEMORY_UNMAP_OBJECT
    int 0x80
.Lfreeze_done:
    ret

// The kernel already supports 48 interests. Stage all 768 borrowed bytes in
// one page-contained 1024-byte slot, preserving the caller's stack and inputs.
.section .text.cathedral_kernel_gate,"ax",@progbits
cathedral_wait_many:
    push rbp
    mov rbp, rsp
    sub rsp, 2048
    and rsp, -1024
    mov r8, rsi
    mov r9, rdx
    mov rsi, rdi
    mov rdi, rsp
    mov ecx, 96
    rep movsq
    mov eax, 68                      // EVENT_WAIT
    mov rdi, rsp
    mov rsi, r8
    mov rdx, r9
    int 0x80
    mov rsp, rbp
    pop rbp
    ret


// Child tasks. Launch, port and child names are kernel tickets scoped to this
// task; the kernel checks ownership of each one. Nothing here infers a PID.
cathedral_launch_at:
    mov eax, 7                       // TASK_LAUNCH
    xor esi, esi
    xor edx, edx
    int 0x80
    ret
cathedral_launch_port:
    mov eax, 49                      // TASK_LAUNCH_PORT
    xor esi, esi
    xor edx, edx
    int 0x80
    ret
cathedral_task_connect:
    mov eax, 11                      // TASK_CONNECT
    xor esi, esi
    xor edx, edx
    int 0x80
    ret
cathedral_task_cancel:
    mov eax, 13                      // TASK_CANCEL
    xor esi, esi
    xor edx, edx
    int 0x80
    ret

// Stage the borrowed 176-byte spawn record in one 256-byte-aligned slot so the
// kernel's single-page copy-in never straddles a boundary. Nothing is written back.
cathedral_spawn_tree:
    push rbp
    mov rbp, rsp
    sub rsp, 512
    and rsp, -256
    mov r8, rdi
    mov rdi, rsp
    mov ecx, 22
    rep movsq
    mov eax, 40                      // TASK_SPAWN_TREE
    mov rdi, r8
    mov rsi, rsp
    xor edx, edx
    int 0x80
    mov rsp, rbp
    pop rbp
    ret

// The kernel writes the 40-byte exit record; copy it back into the borrowed array.
cathedral_task_wait:
    mov r8, rdi
    mov rdi, rsi
    stage_buffer
    mov eax, 9                       // TASK_WAIT
    mov rdi, r8
    mov rsi, rsp
    mov edx, 40
    int 0x80
    finish_buffer 1

cathedral_endpoint_send:
    stage_packet
    mov eax, 36                      // IPC_SEND_CAP
    mov rdi, r8
    mov rsi, rsp
    xor edx, edx
    int 0x80
    finish_packet 0

// Own-account budget snapshots: 48 and 32 writable bytes within the 64-byte stage.
cathedral_resource_budget:
    stage_buffer
    mov eax, 43                      // RESOURCE_BUDGET
    mov rdi, rsp
    xor esi, esi
    xor edx, edx
    int 0x80
    finish_buffer 1
cathedral_notification_budget:
    stage_buffer
    mov eax, 81                      // NOTIFY_BUDGET
    mov rdi, rsp
    xor esi, esi
    xor edx, edx
    int 0x80
    finish_buffer 1

.section .note.GNU-stack,"",@progbits
