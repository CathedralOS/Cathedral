//! Single-CPU address-space transitions for the Rust containment experiment.
//! Entry assembly switches to the kernel root before calling Rust. Only the
//! image, entry stacks and context-storage heap are mapped in both roots.

pub(super) mod memory;
pub use memory::{ImageRange, UserSpace};
mod probe;
use ::x86_64::registers::control::{Cr3, Cr4, Cr4Flags};
use core::sync::atomic::{AtomicU64, Ordering};
pub use probe::user_probe;

pub(super) static ENTRY_ROOT: AtomicU64 = AtomicU64::new(0);
pub(super) static RETURN_ROOT: AtomicU64 = AtomicU64::new(0);

/// # Safety
/// Sole CPU, IRQs off, no other task session. All user roots must map the entry
/// code/tables, emergency stacks and heap contexts supervisor-only.
pub unsafe fn begin_user_session() {
    assert_eq!(ENTRY_ROOT.load(Ordering::Acquire), 0);
    // SAFETY: PCID was rejected during boot. Disable global translations so no
    // inherited firmware translation survives a switch to a restricted root.
    unsafe {
        Cr4::update(|flags| flags.remove(Cr4Flags::PAGE_GLOBAL));
    }
    let root = Cr3::read().0.start_address().as_u64();
    ENTRY_ROOT.store(root, Ordering::Release);
    RETURN_ROOT.store(root, Ordering::Release);
}

/// Select a root for assembly's eventual IRET, not for the running Rust handler.
/// # Safety
/// IRQs off in an active user session; root belongs to its live selected context.
/// None selects the boot/kernel context. Returned context storage must be mapped.
pub unsafe fn select_user_root(root: Option<u64>) {
    let kernel = ENTRY_ROOT.load(Ordering::Acquire);
    assert_ne!(kernel, 0);
    RETURN_ROOT.store(root.unwrap_or(kernel), Ordering::Release);
}

/// # Safety
/// Back on the boot context with IRQs off, every user task retired, callback removed.
pub unsafe fn end_user_session() {
    assert_eq!(
        Cr3::read().0.start_address().as_u64(),
        ENTRY_ROOT.load(Ordering::Acquire)
    );
    RETURN_ROOT.store(0, Ordering::Release);
    ENTRY_ROOT.store(0, Ordering::Release);
}
