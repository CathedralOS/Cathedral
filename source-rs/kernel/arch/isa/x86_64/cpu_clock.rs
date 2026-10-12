//! Single-CPU lab execution clock. Calibrate a fixed-rate TSC against the PIT.
//! QEMU supplies the reference time model; this is not native clock discovery,
//! frequency-change compensation, multicore synchronization or a WCET clock.
use core::{
    arch::asm,
    sync::atomic::{AtomicU64, Ordering},
};
static ORIGIN: AtomicU64 = AtomicU64::new(0);
static HZ: AtomicU64 = AtomicU64::new(0);

fn counter() -> u64 {
    let low: u32;
    let high: u32;
    // SAFETY: x86-64 lab requires TSC/SSE2. Fences order both sides of the sample;
    // no nomem annotation, so compiler memory operations cannot cross it either.
    unsafe {
        asm!("lfence", "rdtsc", "lfence", out("eax") low, out("edx") high,
            options(nostack, preserves_flags));
    }
    (u64::from(high) << 32) | u64::from(low)
}
/// # Safety
/// Sole boot CPU, fixed-rate TSC, installed 100 Hz timer/IDT, no active session or locks.
/// Temporarily enables interrupts while sampling; returns with IRQs masked.
pub unsafe fn calibrate_cpu_clock() {
    assert!(
        core::arch::x86_64::__cpuid(1).edx & (1 << 4) != 0,
        "TSC required"
    );
    assert_eq!(HZ.load(Ordering::Relaxed), 0);
    // SAFETY: Caller supplies installed timer and permits idle interrupt delivery.
    unsafe { super::interrupts::wait_for_ticks(1) };
    let start = counter();
    // SAFETY: Same timer/idle conditions; five full nominal PIT periods.
    unsafe { super::interrupts::wait_for_ticks(5) };
    let elapsed = counter().checked_sub(start).expect("non-monotonic TSC");
    let hz = elapsed.checked_mul(20).expect("TSC calibration overflow");
    assert!(hz >= 1_000_000, "invalid TSC calibration");
    ORIGIN.store(start, Ordering::Relaxed);
    HZ.store(hz, Ordering::Release);
}
/// Kernel-internal microseconds; does not grant an application a clock capability.
pub fn cpu_time_us() -> u64 {
    let hz = HZ.load(Ordering::Acquire);
    assert_ne!(hz, 0, "execution clock not initialized");
    let elapsed = counter()
        .checked_sub(ORIGIN.load(Ordering::Relaxed))
        .expect("TSC went backwards");
    (u128::from(elapsed) * 1_000_000 / u128::from(hz)) as u64
}
