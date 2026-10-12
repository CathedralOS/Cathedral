//! Single-CPU QEMU virt: GICv2 timer PPI 27, nominally 100 Hz.
use super::{Fault, fatal};
use crate::board::virt::{TIMER_IRQ, gic};
use core::{
    arch::asm,
    sync::atomic::{AtomicU64, Ordering},
};
static TICKS: AtomicU64 = AtomicU64::new(0);
static LAST_IRQ: AtomicU64 = AtomicU64::new(0);
fn frequency() -> u64 {
    let f;
    // SAFETY: Architectural counter-frequency read at EL1.
    unsafe {
        asm!("mrs {},cntfrq_el0",out(reg) f,options(nostack));
    }
    f
}
pub fn cpu_time_us() -> u64 {
    let c: u64;
    // SAFETY: Architectural virtual counter read at EL1.
    unsafe {
        asm!("mrs {},cntvct_el0",out(reg) c,options(nostack));
    }
    (u128::from(c) * 1_000_000 / u128::from(frequency())) as u64
}
/// # Safety
/// Timer initialized on the sole EL1 CPU.
pub unsafe fn calibrate_cpu_clock() {
    assert!(frequency() >= 100);
}
/// # Safety
/// Reserved QEMU virt GICv2 MMIO, exclusively owned; vectors already installed.
pub unsafe fn start_timer() {
    // SAFETY: Fixed-board distributor/CPU interface; enable only virtual timer PPI 27.
    unsafe {
        gic::start_timer_route();
        asm!("msr cntp_ctl_el0,xzr","msr cntv_tval_el0, {}","msr cntv_ctl_el0, {}","isb",in(reg) frequency()/100,in(reg) 1u64,options(nostack));
    }
}
pub(in crate::isa::aarch64) fn acknowledge() {
    // SAFETY: Sole CPU, IRQs masked in vector entry; MMIO belongs to this backend.
    unsafe {
        let id = gic::acknowledge();
        if id & 1023 != TIMER_IRQ {
            fatal(Fault {
                vector: 0xffff,
                error: id as u64,
                address: 0,
                instruction: 0,
                stack: crate::isa::aarch64::stack_pointer(),
            });
        }
        asm!("msr cntv_tval_el0, {}","isb",in(reg) frequency()/100,options(nostack));
        gic::complete(id);
    }
    TICKS.fetch_add(1, Ordering::Relaxed);
    LAST_IRQ.store(crate::isa::aarch64::stack_pointer(), Ordering::Relaxed);
}
pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}
pub fn last_irq_stack() -> u64 {
    LAST_IRQ.load(Ordering::Relaxed)
}
/// # Safety
/// Installed timer/vectors; no mutable borrow or lock can span an interrupt.
pub unsafe fn wait_for_ticks(count: u64) -> u64 {
    let goal = ticks().wrapping_add(count);
    // SAFETY: Temporarily unmask IRQs only while the caller is quiescent, then restore masked state.
    unsafe {
        while ticks().wrapping_sub(goal) >= (1 << 63) {
            asm!("msr daifclr,#2", "wfi", "msr daifset,#2", options(nostack));
        }
    }
    ticks()
}
