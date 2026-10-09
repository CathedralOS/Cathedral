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

pub fn exercise(memory: &mut crate::memory::PreparedMemory, console: &mut SerialPort) {
    memory.frames.enable_reclamation();
    // SAFETY: We remain on the privileged boot CPU, with the heap initialized.
    let baseline = unsafe { HEAP.used() };
    // Run twice to verify task exit releases heap/stack backing and virtual slots
    // can be reused without stale contexts, wait state or registered callbacks.
    for _ in 0..2 {
        for progress in &PROGRESS {
            progress.store(0, Ordering::Relaxed);
        }
        // SAFETY: Owned reclaiming allocator and mappings, IRQs masked, timer/IDT live.
        let stats = unsafe {
            tasks::run(
                &mut memory.frames,
                tasks::Config {
                    task_limit: 2,
                    preempt: false,
                    ..Default::default()
                },
                &[first, second],
            )
        }
        .unwrap();
        assert_eq!(stats.exits, 2);
        assert!(stats.preemptions.iter().all(|count| *count == 0));
        assert!(stats.switches >= 8);
        for (index, saved) in STACKS.iter().enumerate() {
            let stack = arch::task_stack_range(index);
            assert!((stack.bottom..stack.top).contains(&saved.load(Ordering::Acquire)));
        }
        // SAFETY: The task callback has been removed and both tasks have exited.
        assert_eq!(unsafe { HEAP.used() }, baseline);
    }
    writeln!(
        console,
        "Cathedral Rust lab: cooperative tasks yielded slept woke and reclaimed"
    )
    .ok();
    exercise_preemption(memory, console, baseline);
}

fn exercise_preemption(
    memory: &mut crate::memory::PreparedMemory,
    console: &mut SerialPort,
    baseline: usize,
) {
    for progress in &PROGRESS {
        progress.store(0, Ordering::Relaxed);
    }
    // SAFETY: The cooperative session returned both stack slots, removed its
    // callback and reclaimed contexts. Admission can map fresh backing in those slots.
    let stats = unsafe {
        tasks::run(
            &mut memory.frames,
            tasks::Config {
                task_limit: 2,
                ..Default::default()
            },
            &[busy_first, busy_second],
        )
    }
    .unwrap();
    assert_eq!(stats.exits, 2);
    assert!(stats.preemptions[..2].iter().all(|count| *count > 0));
    for (index, saved) in STACKS.iter().enumerate() {
        let stack = arch::task_stack_range(index);
        assert!((stack.bottom..stack.top).contains(&saved.load(Ordering::Acquire)));
    }
    // SAFETY: No tasks or scheduler callback remain; allocation usage is stable.
    assert_eq!(unsafe { HEAP.used() }, baseline);
    writeln!(
        console,
        "Cathedral Rust lab: preempted non-yielding tasks counts={:?}; GPR SSE x87 MXCSR preserved",
        &stats.preemptions[..2]
    )
    .ok();
    writeln!(
        console,
        "Cathedral Rust lab: task heap reclaimed and stack slots reusable"
    )
    .ok();
}

fn busy_first() {
    busy_worker(0);
}
fn busy_second() {
    busy_worker(1);
}

fn busy_worker(index: usize) {
    STACKS[index].store(arch::stack_pointer(), Ordering::Release);
    let retained: Vec<u64> = (0..512).map(|value| value ^ index as u64).collect();
    // Exercise allocator lock masking under actual timer-driven scheduling.
    let start = arch::ticks();
    while arch::ticks().wrapping_sub(start) < 3 {
        let scratch: Vec<u64> = (0..128).map(|value| value + index as u64).collect();
        assert_eq!(core::hint::black_box(scratch[127]), 127 + index as u64);
    }
    // SAFETY: IRQs are enabled in the task's saved flags, with a live timer.
    // Distinct patterns force stale register/FP state from the other task to fail.
    let preserved = unsafe {
        arch::probe_registers(
            &PROGRESS[index],
            arch::ticks().wrapping_add(6),
            if index == 0 { 17 } else { 34 },
        )
    };
    assert!(preserved, "register state changed across timer preemption");
    assert!(
        PROGRESS[1 - index].load(Ordering::Acquire) > 0,
        "peer never ran while this non-yielding task was alive"
    );
    assert_eq!(retained[511], 511 ^ index as u64);
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
    if index == 1 {
        // Task zero has just gone to sleep. Do observable work while it waits.
        for _ in 0..4 {
            PROGRESS[index].fetch_add(1, Ordering::Relaxed);
            tasks::yield_now();
        }
    }
    let start = arch::ticks();
    tasks::sleep(delay);
    assert!(arch::ticks().wrapping_sub(start) >= delay);
    assert_eq!(values[511], 511 ^ index as u64);
    assert_eq!(core::hint::black_box(canary), [0xa55a_1234_5678_4321; 16]);
    assert!(PROGRESS[1 - index].load(Ordering::Acquire) >= 4);
    if index == 0 {
        assert_eq!(PROGRESS[1].load(Ordering::Acquire), 8);
    }
}
