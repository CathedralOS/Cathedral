// Four exception classes for each architectural vector group; AArch32 is never entered.
// Context layout: q[32] at 0, x[31] at 512, SP/PC/PSTATE/FPCR/FPSR at 760..792.
.text
.p2align 11
.globl cathedral_vectors
cathedral_vectors:
    b kernel_sync
    .balign 128
    b vector_1
    .balign 128
    b vector_2
    .balign 128
    b vector_3
    .balign 128
    b kernel_sync
    .balign 128
    b vector_1
    .balign 128
    b vector_2
    .balign 128
    b vector_3
    .balign 128
    b vector_0
    .balign 128
    b vector_1
    .balign 128
    b vector_2
    .balign 128
    b vector_3
    .balign 128
    b vector_0
    .balign 128
    b vector_1
    .balign 128
    b vector_2
    .balign 128
    b vector_3
    .balign 128
.macro save_vector kind
    // TPIDR_EL1 is reserved as single-CPU entry scratch (not a TLS register).
    // Preserve x0 there, then x1 and the interrupted SP on the permanent IRQ
    // stack before changing SP. Never store onto an interrupted kernel stack.
    msr tpidr_el1, x0
    adrp x0, {irq_stack}
    ldr x0, [x0, :lo12:{irq_stack}]
    sub x0, x0, #800
    str x1, [x0, #520]
    mov x1, sp
    str x1, [x0, #760]
    mov sp, x0
    mrs x0, tpidr_el1
    msr tpidr_el1, xzr
    str x0, [sp, #512]
    str x2, [sp, #528]
    str x3, [sp, #536]
    str x4, [sp, #544]
    str x5, [sp, #552]
    str x6, [sp, #560]
    str x7, [sp, #568]
    str x8, [sp, #576]
    str x9, [sp, #584]
    str x10, [sp, #592]
    str x11, [sp, #600]
    str x12, [sp, #608]
    str x13, [sp, #616]
    str x14, [sp, #624]
    str x15, [sp, #632]
    str x16, [sp, #640]
    str x17, [sp, #648]
    str x18, [sp, #656]
    str x19, [sp, #664]
    str x20, [sp, #672]
    str x21, [sp, #680]
    str x22, [sp, #688]
    str x23, [sp, #696]
    str x24, [sp, #704]
    str x25, [sp, #712]
    str x26, [sp, #720]
    str x27, [sp, #728]
    str x28, [sp, #736]
    str x29, [sp, #744]
    str x30, [sp, #752]
    stp q0, q1, [sp, #0]
    stp q2, q3, [sp, #32]
    stp q4, q5, [sp, #64]
    stp q6, q7, [sp, #96]
    stp q8, q9, [sp, #128]
    stp q10, q11, [sp, #160]
    stp q12, q13, [sp, #192]
    stp q14, q15, [sp, #224]
    stp q16, q17, [sp, #256]
    stp q18, q19, [sp, #288]
    stp q20, q21, [sp, #320]
    stp q22, q23, [sp, #352]
    stp q24, q25, [sp, #384]
    stp q26, q27, [sp, #416]
    stp q28, q29, [sp, #448]
    stp q30, q31, [sp, #480]
    mov x1, #\kind
    b saved
    .endm

// EL1 synchronous faults must not overwrite an active IRQ frame/stack. Only
// supported returning BRK/SVC enter the normal save/dispatch path.
kernel_sync:
    msr tpidr_el1, x0
    mrs x0, esr_el1
    lsr x0, x0, #26
    cmp x0, #0x15
    b.eq returning_sync
    cmp x0, #0x3c
    b.ne fatal_vector
returning_sync:
    mrs x0, tpidr_el1
vector_0:
    save_vector 0
vector_1:
    save_vector 1
vector_2:
    mov x0, #2
    b fatal_vector
vector_3:
    mov x0, #3
    b fatal_vector
saved:
    mrs x9, spsr_el1
    str x9, [sp, #776]
    and x10, x9, #15
    cbnz x10, save_pc
    mrs x10, sp_el0
    str x10, [sp, #760]
save_pc:
    mrs x10, elr_el1
    str x10, [sp, #768]
    mrs x10, fpcr
    str x10, [sp, #784]
    mrs x10, fpsr
    str x10, [sp, #792]
    mov x0, sp
    mrs x2, esr_el1
    mrs x3, far_el1
    adrp x9, {irq_stack}
    ldr x9, [x9, :lo12:{irq_stack}]
    // Reserve top 1024 bytes for the saved frame; Rust dispatch grows below it.
    sub x9, x9, #1024
    mov sp, x9
    bl {dispatch}
    mov x30, x0
    ldr x9, [x30, #768]
    msr elr_el1, x9
    ldr x9, [x30, #776]
    msr spsr_el1, x9
    and x9, x9, #15
    ldr x10, [x30, #760]
    cbnz x9, kernel_stack
    msr sp_el0, x10
    adrp x9, {irq_stack}
    ldr x10, [x9, :lo12:{irq_stack}]
kernel_stack:
    mov sp, x10
    ldr x9, [x30, #784]
    msr fpcr, x9
    ldr x9, [x30, #792]
    msr fpsr, x9
    ldp q0, q1, [x30, #0]
    ldp q2, q3, [x30, #32]
    ldp q4, q5, [x30, #64]
    ldp q6, q7, [x30, #96]
    ldp q8, q9, [x30, #128]
    ldp q10, q11, [x30, #160]
    ldp q12, q13, [x30, #192]
    ldp q14, q15, [x30, #224]
    ldp q16, q17, [x30, #256]
    ldp q18, q19, [x30, #288]
    ldp q20, q21, [x30, #320]
    ldp q22, q23, [x30, #352]
    ldp q24, q25, [x30, #384]
    ldp q26, q27, [x30, #416]
    ldp q28, q29, [x30, #448]
    ldp q30, q31, [x30, #480]
    ldr x0, [x30, #512]
    ldr x1, [x30, #520]
    ldr x2, [x30, #528]
    ldr x3, [x30, #536]
    ldr x4, [x30, #544]
    ldr x5, [x30, #552]
    ldr x6, [x30, #560]
    ldr x7, [x30, #568]
    ldr x8, [x30, #576]
    ldr x9, [x30, #584]
    ldr x10, [x30, #592]
    ldr x11, [x30, #600]
    ldr x12, [x30, #608]
    ldr x13, [x30, #616]
    ldr x14, [x30, #624]
    ldr x15, [x30, #632]
    ldr x16, [x30, #640]
    ldr x17, [x30, #648]
    ldr x18, [x30, #656]
    ldr x19, [x30, #664]
    ldr x20, [x30, #672]
    ldr x21, [x30, #680]
    ldr x22, [x30, #688]
    ldr x23, [x30, #696]
    ldr x24, [x30, #704]
    ldr x25, [x30, #712]
    ldr x26, [x30, #720]
    ldr x27, [x30, #728]
    ldr x28, [x30, #736]
    ldr x29, [x30, #744]
    ldr x30, [x30, #752]
    eret

// No stack memory accesses until SP points to the dedicated fatal stack.
// A second fatal fault cannot safely restart reporting: park without recursion.
fatal_vector:
    msr daifset, #15
    adrp x9, {fatal_active}
    ldr x10, [x9, :lo12:{fatal_active}]
    cbnz x10, fatal_halt
    mov x10, #1
    str x10, [x9, :lo12:{fatal_active}]
    mrs x1, esr_el1
    mrs x2, far_el1
    mrs x3, elr_el1
    mov x4, sp
    mrs x9, spsr_el1
    and x9, x9, #15
    cbnz x9, fatal_stack
    mrs x4, sp_el0
fatal_stack:
    adrp x9, {fault_stack}
    ldr x9, [x9, :lo12:{fault_stack}]
    mov sp, x9
    bl {fatal_entry}
fatal_halt:
    wfi
    b fatal_halt
