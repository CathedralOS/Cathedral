//! Single-CPU GDT/TSS/IDT installation and bounded interrupt handlers.
//! Installation orders context-save setup, segment tables, then vector gates.
//! Follow entries for assembly transitions and tables for privilege/stack selection.
mod entries;
mod tables;
pub(super) use tables::user_selectors;

use super::{BootLayout, halt_forever, stack_pointer};
use core::{
    arch::asm,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, Ordering},
};

static INSTALLED: AtomicBool = AtomicBool::new(false);
static FAULT_HANDLER: AtomicPtr<()> = AtomicPtr::new(core::ptr::null_mut());
pub(super) static TICKS: AtomicU64 = AtomicU64::new(0);
static BREAKPOINTS: AtomicU64 = AtomicU64::new(0);
static IRQ_STACK: AtomicU64 = AtomicU64::new(0);
#[derive(Debug)]
pub struct Fault {
    pub vector: u64,
    pub error: u64,
    pub instruction: u64,
    pub address: u64,
    pub stack: u64,
}

pub(super) extern "win64" fn dispatch_fault(
    vector: u64,
    error: u64,
    instruction: u64,
    address: u64,
) -> ! {
    let handler = FAULT_HANDLER.load(Ordering::Acquire);
    if !handler.is_null() {
        // SAFETY: Only install_interrupts writes this pointer, using this exact
        // function type. The target is permanent code and this path never returns.
        let handler: fn(Fault) -> ! = unsafe { core::mem::transmute(handler) };
        handler(Fault {
            vector,
            error,
            instruction,
            address,
            stack: stack_pointer(),
        });
    }
    // SAFETY: A fatal trap runs privileged and cannot resume without a handler.
    unsafe { halt_forever() }
}

/// # Safety
/// Called exactly once on the sole boot CPU with IRQs off, after activating
/// layout's mappings and stack. Image, tables and stacks must remain resident.
/// The callback is a non-returning, allocation-free fatal diagnostic path.
pub unsafe fn install_interrupts(layout: &BootLayout, handler: fn(Fault) -> !) {
    assert!(
        !INSTALLED.load(Ordering::Acquire),
        "interrupt tables already installed"
    );
    FAULT_HANDLER.store(handler as *mut (), Ordering::Release);
    // SAFETY: Initialization is exclusive, single-CPU and before IRQ delivery.
    // Static table addresses and all four mapped emergency stacks are permanent.
    unsafe {
        super::context::enable_context_save();
        tables::install_segments(layout);
        tables::install_gates();
    }
    INSTALLED.store(true, Ordering::Release);
}

/// # Safety
/// Requires installed Cathedral exception tables on the boot CPU.
pub unsafe fn test_breakpoint() {
    assert!(INSTALLED.load(Ordering::Acquire));
    let before = BREAKPOINTS.load(Ordering::Acquire);
    // SAFETY: Vector 3 has a returning assembly handler preserving all registers.
    unsafe {
        asm!("int3");
    }
    assert_eq!(BREAKPOINTS.load(Ordering::Acquire), before + 1);
}

/// # Safety
/// Requires installed IDT, exclusive ownership of PC PIC/PIT ports and one CPU.
pub unsafe fn start_timer() {
    assert!(INSTALLED.load(Ordering::Acquire));
    // SAFETY: The selected PC lab grants this CPU exclusive legacy-controller access.
    unsafe {
        crate::x86::legacy_timer::start_100_hz();
    }
}

/// Wait with the interrupt-enable/halt pair adjacent so a wake cannot be lost.
/// Returns with IRQs masked. The host smoke timeout bounds a broken timer.
/// # Safety
/// Requires the timer/IDT installed and no locks held across interrupt delivery.
pub unsafe fn wait_for_ticks(count: u64) -> u64 {
    let start = TICKS.load(Ordering::Acquire);
    loop {
        let elapsed = TICKS.load(Ordering::Acquire).wrapping_sub(start);
        if elapsed >= count {
            return elapsed;
        }
        // SAFETY: All handlers and stacks are installed; STI's interrupt shadow
        // ensures HLT executes before any pending IRQ can be serviced.
        unsafe {
            asm!("sti", "hlt", "cli", options(nostack));
        }
    }
}

pub fn last_irq_stack() -> u64 {
    IRQ_STACK.load(Ordering::Acquire)
}

pub fn ticks() -> u64 {
    TICKS.load(Ordering::Acquire)
}

/// # Safety
/// Requires the installed IDT. Intentionally raises a fatal invalid-opcode trap.
pub unsafe fn probe_invalid_opcode() -> ! {
    // SAFETY: Caller explicitly requests this diagnostic, with a valid trap path.
    unsafe {
        asm!("ud2", options(noreturn));
    }
}

/// # Safety
/// Requires installed exception handlers and double-fault IST. Guard must be
/// unmapped. Abandons the current stack to force a page-fault delivery failure.
pub unsafe fn probe_double_fault(guard: u64) -> ! {
    // SAFETY: The diagnostic deliberately invalidates RSP and never resumes.
    unsafe {
        asm!("mov rsp, rcx", "push rax", "ud2", in("rcx") guard + 8, options(noreturn));
    }
}
