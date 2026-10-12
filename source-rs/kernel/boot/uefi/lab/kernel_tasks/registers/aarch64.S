// AAPCS64: x0 progress, x1 counter deadline, x2 distinct task pattern.
// Busy loop: no yield or syscall; x18 stays reserved for the platform ABI.
.text
.globl cathedral_register_probe
cathedral_register_probe:
    sub sp, sp, #240
    stp x19, x20, [sp, #0]
    stp x21, x22, [sp, #16]
    stp x23, x24, [sp, #32]
    stp x25, x26, [sp, #48]
    stp x27, x28, [sp, #64]
    stp x29, x30, [sp, #80]
    stp q8, q9, [sp, #96]
    stp q10, q11, [sp, #128]
    stp q12, q13, [sp, #160]
    stp q14, q15, [sp, #192]
    mrs x3, fpcr
    mrs x9, fpsr
    stp x3, x9, [sp, #224]
    and x3, x2, #1
    lsl x3, x3, #22
    msr fpcr, x3
    and x3, x2, #1
    lsl x3, x3, #27
    msr fpsr, x3
    dup v0.2d, x2
    dup v1.2d, x2
    dup v2.2d, x2
    dup v3.2d, x2
    dup v4.2d, x2
    dup v5.2d, x2
    dup v6.2d, x2
    dup v7.2d, x2
    dup v8.2d, x2
    dup v9.2d, x2
    dup v10.2d, x2
    dup v11.2d, x2
    dup v12.2d, x2
    dup v13.2d, x2
    dup v14.2d, x2
    dup v15.2d, x2
    dup v16.2d, x2
    dup v17.2d, x2
    dup v18.2d, x2
    dup v19.2d, x2
    dup v20.2d, x2
    dup v21.2d, x2
    dup v22.2d, x2
    dup v23.2d, x2
    dup v24.2d, x2
    dup v25.2d, x2
    dup v26.2d, x2
    dup v27.2d, x2
    dup v28.2d, x2
    dup v29.2d, x2
    dup v30.2d, x2
    dup v31.2d, x2
    mov x4, x2
    mov x5, x2
    mov x6, x2
    mov x7, x2
    mov x8, x2
    mov x10, x2
    mov x11, x2
    mov x12, x2
    mov x13, x2
    mov x14, x2
    mov x15, x2
    mov x16, x2
    mov x17, x2
    mov x19, x2
    mov x20, x2
    mov x21, x2
    mov x22, x2
    mov x23, x2
    mov x24, x2
    mov x25, x2
    mov x26, x2
    mov x27, x2
    mov x28, x2
    mov x29, x2
    mov x30, x2
1:
    cmp x4, x2
    b.ne 3f
    cmp x5, x2
    b.ne 3f
    cmp x6, x2
    b.ne 3f
    cmp x7, x2
    b.ne 3f
    cmp x8, x2
    b.ne 3f
    cmp x10, x2
    b.ne 3f
    cmp x11, x2
    b.ne 3f
    cmp x12, x2
    b.ne 3f
    cmp x13, x2
    b.ne 3f
    cmp x14, x2
    b.ne 3f
    cmp x15, x2
    b.ne 3f
    cmp x16, x2
    b.ne 3f
    cmp x17, x2
    b.ne 3f
    cmp x19, x2
    b.ne 3f
    cmp x20, x2
    b.ne 3f
    cmp x21, x2
    b.ne 3f
    cmp x22, x2
    b.ne 3f
    cmp x23, x2
    b.ne 3f
    cmp x24, x2
    b.ne 3f
    cmp x25, x2
    b.ne 3f
    cmp x26, x2
    b.ne 3f
    cmp x27, x2
    b.ne 3f
    cmp x28, x2
    b.ne 3f
    cmp x29, x2
    b.ne 3f
    cmp x30, x2
    b.ne 3f
    umov x3, v0.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v0.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v1.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v1.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v2.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v2.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v3.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v3.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v4.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v4.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v5.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v5.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v6.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v6.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v7.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v7.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v8.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v8.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v9.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v9.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v10.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v10.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v11.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v11.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v12.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v12.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v13.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v13.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v14.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v14.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v15.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v15.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v16.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v16.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v17.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v17.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v18.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v18.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v19.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v19.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v20.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v20.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v21.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v21.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v22.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v22.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v23.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v23.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v24.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v24.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v25.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v25.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v26.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v26.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v27.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v27.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v28.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v28.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v29.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v29.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v30.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v30.d[1]
    cmp x3, x2
    b.ne 3f
    umov x3, v31.d[0]
    cmp x3, x2
    b.ne 3f
    umov x3, v31.d[1]
    cmp x3, x2
    b.ne 3f
    mrs x3, fpcr
    and x9, x2, #1
    lsl x9, x9, #22
    cmp x3, x9
    b.ne 3f
    mrs x3, fpsr
    and x9, x2, #1
    lsl x9, x9, #27
    cmp x3, x9
    b.ne 3f
2:
    ldxr x3, [x0]
    add x3, x3, #1
    stxr w9, x3, [x0]
    cbnz w9, 2b
    mrs x9, cntvct_el0
    sub x9, x9, x1
    tbnz x9, #63, 1b
    mov x0, #1
    b 4f
3:
    mov x0, #0
4:
    ldp x3, x9, [sp, #224]
    msr fpcr, x3
    msr fpsr, x9
    ldp q8, q9, [sp, #96]
    ldp q10, q11, [sp, #128]
    ldp q12, q13, [sp, #160]
    ldp q14, q15, [sp, #192]
    ldp x19, x20, [sp, #0]
    ldp x21, x22, [sp, #16]
    ldp x23, x24, [sp, #32]
    ldp x25, x26, [sp, #48]
    ldp x27, x28, [sp, #64]
    ldp x29, x30, [sp, #80]
    add sp, sp, #240
    ret
