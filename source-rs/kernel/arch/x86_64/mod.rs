//! x86-64 machine bring-up. Exports are selected by `arch/lib.rs`.

mod context;
mod interrupts;
mod memory;
mod stacks;
mod user;
pub use crate::x86::{disable_interrupts, halt_forever, in8, out8, qemu_exit};
pub use context::{Context, SwitchCause, UserFault, probe_registers, set_switch_handler, suspend};
pub use interrupts::{
    Fault, install_interrupts, last_irq_stack, probe_double_fault, probe_invalid_opcode,
    start_timer, test_breakpoint, ticks, wait_for_ticks,
};
pub use memory::{BootLayout, MemoryError, StackRange, prepare_memory};
pub use stacks::{MAX_TASK_SLOTS, StackFrames, allocate_stack, release_stack, task_stack_range};
pub use user::memory::{USER_CODE, USER_DATA, USER_STACK, USER_STACK_TOP};
pub use user::{
    ImageRange, UserSpace, begin_user_session, end_user_session, select_user_root, user_probe,
};

use core::arch::asm;

pub const CPU_NAME: &str = "x86_64";

/// # Safety
/// Requires kernel privilege. Body must not change IF or wait for an interrupt.
/// Explicit task suspension is permitted only with no mutable shared borrows or
/// locks spanning it. NMI/fault paths must not acquire locks used by the body.
pub unsafe fn without_interrupts<R>(body: impl FnOnce() -> R) -> R {
    ::x86_64::instructions::interrupts::without_interrupts(body)
}

pub fn stack_pointer() -> u64 {
    let value;
    // SAFETY: Reading the current stack pointer has no privileged side effects.
    unsafe {
        asm!("mov {}, rsp", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// # Safety
/// Stack must be writable, exclusively owned, mapped, and have enough capacity.
/// Entry and context must remain mapped. Context is moved to entry; this never
/// returns, unwinds or destroys values left on the previous stack. IRQs are off.
pub unsafe fn enter_stack(stack_top: u64, entry: unsafe fn(*mut ()) -> !, context: *mut ()) -> ! {
    // SAFETY: Caller supplies a live stack/entry/context. Establish Win64 shadow
    // space and 16-byte call-site alignment, then call the non-returning entry.
    unsafe {
        asm!(
            "mov rsp, rcx", "and rsp, -16", "sub rsp, 32", "xor rbp, rbp",
            "mov rcx, rdx", "mov rdx, r8", "call r9", "ud2",
            in("rcx") stack_top, in("rdx") entry as usize, in("r8") context,
            in("r9") invoke_entry as unsafe extern "win64" fn(usize, *mut ()) -> !,
            options(noreturn)
        );
    }
}

unsafe extern "win64" fn invoke_entry(entry: usize, context: *mut ()) -> ! {
    // SAFETY: enter_stack passes only the exact Rust function-pointer type below.
    let entry: unsafe fn(*mut ()) -> ! = unsafe { core::mem::transmute(entry) };
    // SAFETY: enter_stack's caller supplies the live context and non-returning entry.
    unsafe { entry(context) }
}
