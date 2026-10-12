//! Single-CPU QEMU virt: GICv2 timer PPI 27, nominally 100 Hz.
use super::{Fault, fatal};
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
        (0x0800_0000 as *mut u32).write_volatile(0);
        let banks = ((0x0800_0004 as *const u32).read_volatile() & 31) + 1;
        for bank in 0..banks {
            ((0x0800_0180 + bank * 4) as *mut u32).write_volatile(u32::MAX);
            ((0x0800_0280 + bank * 4) as *mut u32).write_volatile(u32::MAX);
        }
        let group = 0x0800_0080 as *mut u32;
        group.write_volatile(group.read_volatile() & !(1 << 27));
        (0x0800_041b as *mut u8).write_volatile(0x80);
        (0x0800_0100 as *mut u32).write_volatile(1 << 27);
        (0x0801_0004 as *mut u32).write_volatile(0xff);
        (0x0801_0008 as *mut u32).write_volatile(0);
        (0x0801_0000 as *mut u32).write_volatile(1);
        (0x0800_0000 as *mut u32).write_volatile(1);
        asm!("dsb sy", options(nostack));
        asm!("msr cntp_ctl_el0,xzr","msr cntv_tval_el0, {}","msr cntv_ctl_el0, {}","isb",in(reg) frequency()/100,in(reg) 1u64,options(nostack));
    }
}
pub(in crate::aarch64) fn acknowledge() {
    // SAFETY: Sole CPU, IRQs masked in vector entry; MMIO belongs to this backend.
    unsafe {
        let id = (0x0801_000c as *const u32).read_volatile();
        if id & 1023 != 27 {
            fatal(Fault {
                vector: 0xffff,
                error: id as u64,
                address: 0,
                instruction: 0,
                stack: crate::aarch64::stack_pointer(),
            });
        }
        asm!("msr cntv_tval_el0, {}","isb",in(reg) frequency()/100,options(nostack));
        (0x0801_0010 as *mut u32).write_volatile(id);
    }
    TICKS.fetch_add(1, Ordering::Relaxed);
    LAST_IRQ.store(crate::aarch64::stack_pointer(), Ordering::Relaxed);
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
