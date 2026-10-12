// AArch64 EL1 execution on QEMU virt. UEFI remains the firmware entrance.
// memory prepares the machine; interrupts installs vectors/timer; context routes
// traps into shared core; user owns EL0 roots and frame lifetime.
use core::arch::asm;
pub const CPU_NAME: &str = "aarch64";
pub const ELF_MACHINE: u16 = 183;
pub const LEGACY_DEVICES: bool = false;
pub const INTERRUPT_READY: &str = "AArch64 vectors installed; breakpoint returned";
/// # Safety
/// Privileged EL1 execution and exclusive interrupt policy.
pub unsafe fn disable_interrupts() {
    // SAFETY: Caller owns DAIF on the sole CPU.
    unsafe {
        asm!("msr daifset, #15", options(nostack));
    }
}
/// # Safety
/// EL1; body cannot change interrupt policy or retain references across suspension.
pub unsafe fn without_interrupts<R>(body: impl FnOnce() -> R) -> R {
    let flags: u64;
    // SAFETY: Save and mask caller-owned DAIF.
    unsafe {
        asm!("mrs {}, daif", out(reg) flags, options(nostack));
        disable_interrupts();
    }
    let result = body();
    // SAFETY: Restore precisely the caller's interrupt state.
    unsafe {
        asm!("msr daif, {}", in(reg) flags, options(nostack));
    }
    result
}
pub fn stack_pointer() -> u64 {
    let value;
    // SAFETY: Reading SP has no side effects.
    unsafe {
        asm!("mov {}, sp", out(reg) value, options(nomem, nostack));
    }
    value
}
/// # Safety
/// Live exclusive writable stack and context; entry never returns, IRQs masked.
pub unsafe fn enter_stack(top: u64, entry: unsafe fn(*mut ()) -> !, context: *mut ()) -> ! {
    // SAFETY: Caller supplies mapped stack, code and context. AAPCS64 arguments.
    unsafe {
        asm!("mov sp, x9", "mov x29, xzr", "br x10", in("x9") top, in("x10") entry, in("x0") context, options(noreturn));
    }
}
/// # Safety
/// Caller abandons execution on the sole privileged CPU.
pub unsafe fn halt_forever() -> ! {
    // SAFETY: Terminal owner masks all interrupts before waiting.
    unsafe {
        disable_interrupts();
        loop {
            asm!("wfi", options(nostack));
        }
    }
}
/// # Safety
/// Only the lab harness enables semihosting; no guest-provided arguments reach it.
pub unsafe fn qemu_exit(status: u32) -> ! {
    let block = [0x20026u64, if status == 0x10 { 0 } else { 1 }];
    // SAFETY: QEMU SYS_EXIT_EXTENDED, with a live two-word host exit block.
    unsafe {
        asm!("hlt #0xf000", in("x0") 0x20u64, in("x1") block.as_ptr(), options(nostack));
        halt_forever()
    }
}
