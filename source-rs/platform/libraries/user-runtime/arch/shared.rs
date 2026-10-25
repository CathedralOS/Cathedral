//! Normal cacheable shared RAM only. Inline machine accesses deliberately avoid
//! Rust references/atomic objects whose aliasing discipline a hostile peer could
//! violate. Each call is a compiler memory barrier; aligned words use CPU
//! acquire/release ordering. This does not promise a coherent multiword snapshot.
pub(crate) unsafe fn load(address: u64) -> u64 {
    let value;
    // SAFETY: Caller retains an aligned readable normal-memory mapping.
    unsafe {
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov {value}, [{address}]", value = out(reg) value, address = in(reg) address, options(nostack, preserves_flags));
        #[cfg(target_arch = "aarch64")]
        core::arch::asm!("ldar {value}, [{address}]", value = out(reg) value, address = in(reg) address, options(nostack, preserves_flags));
    }
    value
}
pub(crate) unsafe fn store(address: u64, value: u64) {
    // SAFETY: Caller retains an aligned writable normal-memory mapping.
    unsafe {
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov [{address}], {value}", address = in(reg) address, value = in(reg) value, options(nostack, preserves_flags));
        #[cfg(target_arch = "aarch64")]
        core::arch::asm!("stlr {value}, [{address}]", address = in(reg) address, value = in(reg) value, options(nostack, preserves_flags));
    }
}
