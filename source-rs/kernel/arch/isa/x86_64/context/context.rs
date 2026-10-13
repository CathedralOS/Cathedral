// Enable eager CPU state saves, then dispatch traps to the installed core scheduler.
// state.rs owns register layout and initial contexts; interrupt entry.S owns transitions.
use super::interrupts;
use crate::{SwitchCause, UserFault};
use ::x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};
use core::{
    arch::asm,
    sync::atomic::{AtomicPtr, Ordering},
};
use state::ExceptionContext;
/// Enable architectural context saves before publishing the timer/yield gates.
/// # Safety
/// Boot CPU owns floating-point policy, with IRQs off and no tasks running.
pub(super) unsafe fn enable_context_save() {
    let features = core::arch::x86_64::__cpuid(1).edx;
    assert_eq!(features & ((1 << 24) | (1 << 25)), (1 << 24) | (1 << 25));
    // SAFETY: The selected CPU supports FXSAVE/SSE; no lazy FP owner exists.
    unsafe {
        Cr0::update(|flags| {
            flags.remove(Cr0Flags::EMULATE_COPROCESSOR | Cr0Flags::TASK_SWITCHED);
            flags.insert(Cr0Flags::MONITOR_COPROCESSOR);
        });
        Cr4::update(|flags| flags.insert(Cr4Flags::OSFXSR | Cr4Flags::OSXMMEXCPT_ENABLE));
    }
}
pub type SwitchHandler = unsafe fn(&Context, SwitchCause, u64) -> *const Context;
static HANDLER: AtomicPtr<()> = AtomicPtr::new(core::ptr::null_mut());

/// # Safety
/// IRQs are off on the sole CPU. Handler cannot allocate, suspend or enable IRQs.
/// Returned context stays live until assembly returns. Saved contexts cannot
/// refer to the shared IRQ stack after dispatch returns.
pub unsafe fn set_switch_handler(handler: Option<SwitchHandler>) {
    HANDLER.store(
        handler.map_or(core::ptr::null_mut(), |value| value as *mut ()),
        Ordering::Release,
    );
}
pub(super) extern "win64" fn dispatch(context: *const Context, cause: u64) -> *const Context {
    if cause == 4 {
        // SAFETY: Dedicated COM2 IRQ entry; mask before acknowledging the PIC.
        unsafe {
            crate::board::pc::serial::interrupt();
        }
    }
    if matches!(cause, 1 | 3 | 4) {
        // SAFETY: IRQ entry on the sole CPU; the selected board owns this route.
        unsafe {
            crate::board::pc::timer::acknowledge();
        }
    }
    let handler = HANDLER.load(Ordering::Acquire);
    if handler.is_null() {
        return context;
    }
    // SAFETY: Assembly supplies a fully initialized aligned context. Only the
    // setter publishes this exact callback type, with interrupts disabled.
    unsafe {
        let handler: SwitchHandler = core::mem::transmute(handler);
        handler(
            &*context,
            match cause {
                0 => SwitchCause::Yield,
                1 => SwitchCause::Timer,
                2 => SwitchCause::Syscall,
                3 | 4 => SwitchCause::Device,
                _ => unreachable!(),
            },
            interrupts::ticks(),
        )
    }
}

pub(super) extern "win64" fn dispatch_exception(
    raw: &ExceptionContext,
    address: u64,
) -> *const Context {
    let handler = HANDLER.load(Ordering::Acquire);
    // NMI, double fault and machine check remain machine-fatal regardless of CPL.
    if raw.frame.code & 3 == 3
        && matches!(raw.vector, 0 | 1 | 4..=7 | 10..=14 | 16 | 17 | 19)
        && !handler.is_null()
    {
        let context = Context {
            floating: raw.floating,
            registers: raw.registers.clone(),
            frame: raw.frame.clone(),
        };
        // SAFETY: Setter publishes this callback; IRQs are masked. The callback
        // must copy this temporary context into its own permanent storage.
        unsafe {
            let handler: SwitchHandler = core::mem::transmute(handler);
            return handler(
                &context,
                SwitchCause::Fault(UserFault {
                    vector: raw.vector,
                    error: raw.error,
                    address,
                    instruction: raw.frame.instruction,
                }),
                interrupts::ticks(),
            );
        }
    }
    interrupts::dispatch_fault(raw.vector, raw.error, raw.frame.instruction, address)
}
/// # Safety
/// Requires installed vector 48 and task-switch callback. Caller holds no
/// references into mutable scheduler state or non-suspendable locks.
pub unsafe fn suspend() {
    // SAFETY: Vector 48 saves/restores the same full context as the timer.
    unsafe {
        asm!("int 0x30");
    }
}
