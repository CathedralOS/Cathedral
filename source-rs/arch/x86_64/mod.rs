//! x86-64 machine bring-up. Exports are selected by `arch/lib.rs`.

mod memory;
pub use crate::x86::{disable_interrupts, halt_forever, in8, out8, qemu_exit};
pub use memory::{BootLayout, MemoryError, prepare_memory};

use core::arch::asm;

pub const CPU_NAME: &str = "x86_64";

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
pub unsafe fn enter_stack(
    stack_top: u64,
    entry: unsafe extern "win64" fn(*mut ()) -> !,
    context: *mut (),
) -> ! {
    // SAFETY: Caller supplies a live stack/entry/context. Establish Win64 shadow
    // space and 16-byte call-site alignment, then call the non-returning entry.
    unsafe {
        asm!(
            "mov rsp, rcx", "and rsp, -16", "sub rsp, 32", "xor rbp, rbp",
            "mov rcx, r8", "call rdx", "ud2",
            in("rcx") stack_top, in("rdx") entry, in("r8") context,
            options(noreturn)
        );
    }
}
