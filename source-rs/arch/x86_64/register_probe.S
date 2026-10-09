// Win64 probe: counter in RCX, tick deadline in RDX, distinct task pattern in R8.
// No yield, syscall or halt occurs in this loop. Only a timer can switch tasks.
.text
.global cathedral_register_probe
cathedral_register_probe:
    push rbx
    push rbp
    push rdi
    push rsi
    push r12
    push r13
    push r14
    push r15
    sub rsp, 520
    fxsave64 [rsp]
    fninit
    mov [rsp + 512], r8
    fild qword ptr [rsp + 512]
    // Distinct MXCSR rounding mode for odd/even patterns; exceptions masked.
    mov eax, r8d
    and eax, 1
    shl eax, 13
    or eax, 0x1f80
    mov [rsp + 512], eax
    ldmxcsr [rsp + 512]
    movq xmm0, r8
    .irp register, xmm1,xmm2,xmm3,xmm4,xmm5,xmm6,xmm7,xmm8,xmm9,xmm10,xmm11,xmm12,xmm13,xmm14,xmm15
    movdqa \register, xmm0
    .endr
    .irp register, rax,rbx,rbp,rdi,rsi,r9,r10,r11,r12,r13,r14,r15
    mov \register, r8
    .endr
1:
    .irp register, rax,rbx,rbp,rdi,rsi,r9,r10,r11,r12,r13,r14,r15
    cmp \register, r8
    jne 3f
    .endr
    .irp register, xmm0,xmm1,xmm2,xmm3,xmm4,xmm5,xmm6,xmm7,xmm8,xmm9,xmm10,xmm11,xmm12,xmm13,xmm14,xmm15
    movq rax, \register
    cmp rax, r8
    jne 3f
    .endr
    fistp qword ptr [rsp + 512]
    mov rax, [rsp + 512]
    cmp rax, r8
    jne 3f
    fild qword ptr [rsp + 512]
    stmxcsr [rsp + 512]
    mov eax, [rsp + 512]
    mov r9d, r8d
    and r9d, 1
    shl r9d, 13
    or r9d, 0x1f80
    cmp eax, r9d
    jne 3f
    mov r9, r8
    lock inc qword ptr [rcx]
    mov rax, [rip + {ticks}]
    sub rax, rdx
    jns 2f
    mov rax, r8
    jmp 1b
2:
    mov eax, 1
    jmp 4f
3:
    xor eax, eax
4:
    fxrstor64 [rsp]
    add rsp, 520
    pop r15
    pop r14
    pop r13
    pop r12
    pop rsi
    pop rdi
    pop rbp
    pop rbx
    ret
