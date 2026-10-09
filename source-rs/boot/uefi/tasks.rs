//! Bring-up workloads. Each task owns heap data across yields and sleeps;
//! orchestration and reporting happen back on the boot stack after teardown.

use crate::heap::HEAP;
use alloc::vec::Vec;
use cathedral_arch as arch;
use cathedral_core::tasks;
use cathedral_uart_16550::SerialPort;
use core::{
    fmt::Write,
    sync::atomic::{AtomicU64, Ordering},
};

static PROGRESS: [AtomicU64; 2] = [const { AtomicU64::new(0) }; 2];
static STACKS: [AtomicU64; 2] = [const { AtomicU64::new(0) }; 2];

pub fn exercise(layout: &arch::BootLayout, console: &mut SerialPort) {
    // SAFETY: We remain on the privileged boot CPU, with the heap initialized.
    let baseline = unsafe { HEAP.used() };
    // Run twice to verify task exit releases heap resources and the stack pool
    // can be reused without stale contexts, wait state or registered callbacks.
    for _ in 0..2 {
        for progress in &PROGRESS {
            progress.store(0, Ordering::Relaxed);
        }
        // SAFETY: Two disjoint mapped stacks, IRQs masked, timer and IDT installed.
        let stats = unsafe { tasks::run(layout.task_stacks, [first, second], false) };
        assert_eq!(stats.exits, 2);
        assert_eq!(stats.preemptions, [0, 0]);
        assert!(stats.switches >= 8);
        for (index, stack) in layout.task_stacks.iter().enumerate() {
            assert!((stack.bottom..stack.top).contains(&STACKS[index].load(Ordering::Acquire)));
        }
        // SAFETY: The task callback has been removed and both tasks have exited.
        assert_eq!(unsafe { HEAP.used() }, baseline);
    }
    writeln!(
        console,
        "Cathedral Rust lab: cooperative tasks yielded slept woke and reclaimed"
    )
    .ok();
}

fn first() {
    worker(0, 3);
}
fn second() {
    worker(1, 5);
}

fn worker(index: usize, delay: u64) {
    STACKS[index].store(arch::stack_pointer(), Ordering::Release);
    let values: Vec<u64> = (0..512).map(|value| value ^ index as u64).collect();
    let canary = [0xa55a_1234_5678_4321u64; 16];
    for _ in 0..4 {
        PROGRESS[index].fetch_add(1, Ordering::Relaxed);
        tasks::yield_now();
    }
    let start = arch::ticks();
    tasks::sleep(delay);
    assert!(arch::ticks().wrapping_sub(start) >= delay);
    assert_eq!(values[511], 511 ^ index as u64);
    assert_eq!(core::hint::black_box(canary), [0xa55a_1234_5678_4321; 16]);
    assert!(PROGRESS[1 - index].load(Ordering::Acquire) >= 4);
}
