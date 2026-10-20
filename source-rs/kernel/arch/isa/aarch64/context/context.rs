// Exception entrance: restore the kernel root, classify the trap, run shared scheduling,
// then select the return root. state.rs and entry.S define the saved-register ABI.
use crate::{Fault, SwitchCause, UserFault};
use core::{
    arch::{asm, global_asm},
    sync::atomic::{AtomicPtr, Ordering},
};
type Handler = unsafe fn(&Context, SwitchCause, u64) -> *const Context;
static HANDLER: AtomicPtr<()> = AtomicPtr::new(core::ptr::null_mut());
/// # Safety
/// Sole CPU, IRQs off; callback is allocation-free and returns a live mapped context.
pub unsafe fn set_switch_handler(handler: Option<Handler>) {
    HANDLER.store(
        handler.map_or(core::ptr::null_mut(), |h| h as *mut ()),
        Ordering::Release,
    );
}
/// # Safety
/// Installed vectors; sole CPU, no live mutable borrows or locks across suspension.
pub unsafe fn suspend() {
    // SAFETY: EL1 SVC enters the installed scheduler vector and preserves all context.
    unsafe {
        asm!("svc #0", options(nostack));
    }
}
extern "C" fn dispatch(context: &mut Context, kind: u64, esr: u64, address: u64) -> *const Context {
    use super::{interrupts, memory::tables, user};
    let kernel = user::ENTRY_ROOT.load(Ordering::Acquire);
    if kernel != 0 {
        // SAFETY: Session owns the kernel root; entry mappings remain available.
        unsafe {
            tables::activate(kernel);
        }
    }
    let ec = esr >> 26;
    if kind > 1 {
        interrupts::fatal(Fault {
            vector: kind,
            error: esr,
            address,
            instruction: context.instruction,
            stack: context.stack,
        });
    }
    let cause = if kind == 1 {
        interrupts::acknowledge();
        SwitchCause::Timer
    } else if ec == 0x15 {
        if context.is_user() {
            SwitchCause::Syscall
        } else {
            SwitchCause::Yield
        }
    } else if ec == 0x3c && !context.is_user() {
        context.instruction += 4;
        return context;
    } else if context.is_user() {
        SwitchCause::Fault(UserFault {
            vector: ec,
            error: esr,
            address,
            instruction: context.instruction,
        })
    } else {
        interrupts::fatal(Fault {
            vector: ec,
            error: esr,
            address,
            instruction: context.instruction,
            stack: context.stack,
        })
    };
    let handler = HANDLER.load(Ordering::Acquire);
    let next = if handler.is_null() {
        context as *const Context
    } else {
        // SAFETY: Setter's exact callback type and vector-owned aligned saved context.
        unsafe {
            let callback: Handler = core::mem::transmute(handler);
            callback(context, cause, interrupts::ticks())
        }
    };
    if kernel != 0 {
        // SAFETY: Core selected an admitted root retaining entry code, stacks and contexts.
        unsafe {
            tables::activate(user::RETURN_ROOT.load(Ordering::Acquire));
        }
    }
    next
}
global_asm!(include_str!("entry.S"),dispatch=sym dispatch,irq_stack=sym super::interrupts::IRQ_STACK);
unsafe extern "C" {
    pub(super) fn cathedral_vectors();
}

/// Give pending maskable interrupts an opportunity at a consistent boot boundary.
/// # Safety
/// Boot context, IRQs initially masked, installed vectors/handler, and no mutable
/// session borrow or non-suspendable lock. A callback may switch to userspace.
/// Returns with IRQs masked; this is not a delay or an interrupt-drain guarantee.
pub unsafe fn service_pending_interrupts() {
    // SAFETY: Caller establishes a reentrant boundary before this short IRQ window.
    unsafe {
        asm!(
            "msr daifclr, #2",
            "isb",
            "nop",
            "msr daifset, #2",
            "isb",
            options(nostack)
        );
    }
}
