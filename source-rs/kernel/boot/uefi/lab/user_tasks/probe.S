.text
.p2align 4
.global cathedral_user_probe_start
.global cathedral_user_probe_end
cathedral_user_probe_start:
    mov r12, rdi
    mov r14, rsi
    mov rbx, {data}
    cmp qword ptr [rbx], 0
    jne probe_fail
    mov r13, 0x1234567812345600
    add r13, r12
    mov [rbx], r13
    movq xmm15, r13
    // Syscalls preserve all state except their RAX result.
    mov eax, 0xffff
    int 0x80
    cmp rax, -38
    jne probe_fail
    xor eax, eax
    mov rdi, r14
    mov esi, 8
    int 0x80
    cmp rax, -14
    jne probe_fail
    xor eax, eax
    mov rdi, 0x0000800000000000
    int 0x80
    cmp rax, -14
    jne probe_fail
    xor eax, eax
    mov rdi, -4
    int 0x80
    cmp rax, -14
    jne probe_fail
    xor eax, eax
    lea rdi, [rbx + 4095]
    mov esi, 2
    int 0x80
    cmp rax, -14
    jne probe_fail
    xor eax, eax
    mov rdi, rbx
    mov esi, 257
    int 0x80
    cmp rax, -22
    jne probe_fail
    mov eax, 1
    int 0x80
    test rax, rax
    jne probe_fail
    // Busy work never yields. Both tasks must receive actual timer preemption.
    mov ecx, 20000000
    test r12, r12
    jnz probe_spin
    // Survivor runs longer so it must make progress after its peer faults.
    imul ecx, ecx, 3
probe_spin:
    cmp [rbx], r13
    jne probe_fail
    movq rax, xmm15
    cmp rax, r13
    jne probe_fail
    dec ecx
    jnz probe_spin
    cmp r12, 1
    je probe_kernel
    cmp r12, 2
    je probe_cli
    cmp r12, 3
    je probe_port
    cmp r12, 4
    je probe_readonly
    cmp r12, 5
    je probe_nx
    cmp r12, 6
    je probe_guard
    cmp r12, 7
    je probe_gate
    cmp r12, 8
    je probe_ud
    cmp r12, 9
    je probe_kernel
    cmp r12, 10
    je probe_bad_stack
    // Exercise both private writable regions, including non-UTF-8 diagnostics.
    mov word ptr [rbx + 16], 0x0aff
    lea rdi, [rbx + 16]
    mov esi, 2
    xor eax, eax
    int 0x80
    cmp rax, 2
    jne probe_fail
    mov word ptr [rsp - 8], 0x0a2e
    lea rdi, [rsp - 8]
    xor eax, eax
    int 0x80
    cmp rax, 2
    jne probe_fail
    lea rdi, [rip + probe_message]
    mov esi, offset probe_message_length
    xor eax, eax
    int 0x80
    cmp rax, offset probe_message_length
    jne probe_fail
    xor edi, edi
    mov eax, 2
    int 0x80
    ud2
probe_kernel:
    mov rax, [r14]
    jmp probe_fail
probe_cli:
    cli
    jmp probe_fail
probe_port:
    mov dx, 0x3f8
    out dx, al
    jmp probe_fail
probe_readonly:
    mov rax, {code}
    mov byte ptr [rax], 0
    jmp probe_fail
probe_nx:
    mov byte ptr [rsp - 8], 0xc3
    lea rax, [rsp - 8]
    jmp rax
probe_guard:
    mov rax, {stack}
    mov byte ptr [rax - 8], 1
    jmp probe_fail
probe_gate:
    int 0x30
    jmp probe_fail
probe_ud:
    ud2
probe_bad_stack:
    xor esp, esp
    cli
    jmp probe_fail
probe_fail:
    mov edi, 99
    mov eax, 2
    int 0x80
    ud2
probe_message:
    .ascii "user task survived\n"
probe_message_end:
.set probe_message_length, probe_message_end - probe_message
cathedral_user_probe_end:
