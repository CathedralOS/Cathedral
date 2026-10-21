//! ARM64 stack-entry probes. A guard access on a healthy stack is insufficient:
//! these boots abandon SP itself, then require the original fault on a safe stack.
use cathedral_arch as arch;
use core::{
    arch::{asm, global_asm},
    sync::atomic::{AtomicU64, Ordering},
};

static EXPECTED_SP: AtomicU64 = AtomicU64::new(0);
static EMERGENCY_TOP: AtomicU64 = AtomicU64::new(0);

pub(super) fn probe(layout: &arch::BootLayout) {
    if cfg!(feature = "fault-stack-entry") {
        let first: u64;
        let second: u64;
        let before = arch::ticks();
        // SAFETY: Exclusive boot, IRQs masked. Leave only 16 bytes of valid
        // stack, exercise returning BRK/SVC and a real timer IRQ, then restore
        // the caller stack. This terminal probe never resumes ordinary boot.
        unsafe {
            arch::start_timer();
            asm!(
                "mov x10, sp", "mov sp, x9", "brk #0", "svc #0",
                "msr daifclr, #2", "isb", "wfi", "msr daifset, #2", "isb",
                "mov sp, x10",
                in("x9") layout.stack_bottom + 16,
                out("x10") _,
                inout("x0") 0x1234u64 => first,
                inout("x1") 0x5678u64 => second,
            );
        }
        assert_eq!((first, second), (0x1234, 0x5678));
        assert!(arch::ticks() > before);
        // SAFETY: Terminal smoke proof, boot's UART owner will not resume.
        let mut console = unsafe { crate::console::acquire() };
        use core::fmt::Write;
        writeln!(console, "CATHEDRAL_RS_STACK_ENTRY_OK").ok();
        super::complete();
    }
    let bottom = if cfg!(feature = "fault-stack-exhaustion") {
        layout.stack_bottom
    } else if cfg!(feature = "fault-irq-stack-exhaustion") {
        layout.interrupt_stack().bottom
    } else {
        return;
    };
    EXPECTED_SP.store(bottom - 16, Ordering::Release);
    EMERGENCY_TOP.store(layout.emergency_tops[0], Ordering::Release);
    // SAFETY: Intentional terminal stack exhaustion; no Rust frame uses the
    // abandoned stack. Symbols identify the exact store that must be diagnosed.
    unsafe { overflow(bottom) }
}

pub(super) fn matches(fault: &arch::Fault) -> bool {
    let sp = EXPECTED_SP.load(Ordering::Acquire);
    let top = EMERGENCY_TOP.load(Ordering::Acquire);
    sp != 0
        && fault.vector == 0x25
        && fault.error & 0x7f == 0x47 // EL1 write, level-3 translation fault.
        && fault.address == sp
        && fault.stack == sp
        && fault.instruction == faulting_write as *const () as u64
        && (top - 4 * 4096..top).contains(&arch::stack_pointer())
}

global_asm!(
    ".text",
    ".globl cathedral_stack_overflow",
    "cathedral_stack_overflow:",
    "mov sp, x0",
    "sub sp, sp, #16",
    ".globl cathedral_stack_faulting_write",
    "cathedral_stack_faulting_write:",
    "str xzr, [sp]",
    "brk #1",
);
unsafe extern "C" {
    #[link_name = "cathedral_stack_overflow"]
    fn overflow(bottom: u64) -> !;
    #[link_name = "cathedral_stack_faulting_write"]
    fn faulting_write();
}
