// Single-CPU task root selection, consumed by the exception return path.
use super::memory::tables;
use core::sync::atomic::{AtomicU64, Ordering};
pub(in crate::isa::aarch64) static ENTRY_ROOT: AtomicU64 = AtomicU64::new(0);
pub(in crate::isa::aarch64) static RETURN_ROOT: AtomicU64 = AtomicU64::new(0);
/// # Safety
/// EL1, IRQs masked, no session active; every task retains the supervisor boot branch.
pub unsafe fn begin_user_session() {
    assert_eq!(ENTRY_ROOT.load(Ordering::Acquire), 0);
    let root = tables::active();
    ENTRY_ROOT.store(root, Ordering::Release);
    RETURN_ROOT.store(root, Ordering::Release);
}
/// # Safety
/// Active session, IRQs masked; root and its saved context remain live until retired.
pub unsafe fn select_user_root(root: Option<u64>) {
    let kernel = ENTRY_ROOT.load(Ordering::Acquire);
    assert_ne!(kernel, 0);
    RETURN_ROOT.store(root.unwrap_or(kernel), Ordering::Release);
}
/// # Safety
/// Kernel context restored, tasks retired and switch callback removed, IRQs masked.
pub unsafe fn end_user_session() {
    assert_eq!(tables::active(), ENTRY_ROOT.load(Ordering::Acquire));
    RETURN_ROOT.store(0, Ordering::Release);
    ENTRY_ROOT.store(0, Ordering::Release);
}
