//! Admission, allocation-failure rollback, stale IDs and live-peer stress.

use crate::{heap::HEAP, memory::PreparedMemory};
use alloc::vec::Vec;
use cathedral_arch as arch;
use cathedral_core::tasks::{self, Config, SpawnError};
use cathedral_uart_16550::SerialPort;
use core::{
    fmt::Write,
    sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};

const ROUNDS: usize = 32;
static RELEASE: AtomicBool = AtomicBool::new(false);
static DONE: AtomicBool = AtomicBool::new(false);
static PEER_READY: AtomicBool = AtomicBool::new(false);
static PEER_PROGRESS: AtomicU64 = AtomicU64::new(0);
static CHILDREN: AtomicUsize = AtomicUsize::new(0);

pub fn exercise(memory: &mut PreparedMemory, console: &mut SerialPort) {
    // SAFETY: Boot CPU owns the initialized heap; no session is active.
    let heap = unsafe { HEAP.used() };
    let frames = memory.frames.allocated();
    // The first stack requires 16 data + 3 table frames. Fail at every allocation
    // boundary, including after some leaves or only intermediate tables exist.
    for budget in 0..19 {
        let config = Config {
            task_limit: 2,
            frame_limit: budget,
            ..Default::default()
        };
        // SAFETY: Boot context owns allocator and mappings with IRQs masked.
        let error = unsafe { tasks::run(&mut memory.frames, config, &[never_run]) }.unwrap_err();
        assert_eq!(error, SpawnError::OutOfFrames);
        assert_eq!(memory.frames.allocated(), frames);
        // SAFETY: Failed admission returned after complete teardown.
        assert_eq!(unsafe { HEAP.used() }, heap);
    }
    // Also roll back an already admitted first task when the second cannot fit.
    assert_eq!(
        // SAFETY: Serialized boot context; neither task runs before admission completes.
        unsafe {
            tasks::run(
                &mut memory.frames,
                Config {
                    frame_limit: 19,
                    ..Default::default()
                },
                &[never_run, never_run],
            )
        }
        .unwrap_err(),
        SpawnError::OutOfFrames
    );
    assert_eq!(memory.frames.allocated(), frames);
    // SAFETY: No task session remains.
    assert_eq!(unsafe { HEAP.used() }, heap);
    writeln!(
        console,
        "Cathedral Rust lab: stack allocation rollback passed all frame boundaries"
    )
    .ok();

    // The first two stacks consume 36 frames; fail partway through a third while
    // an existing non-yielding peer remains alive. Rollback must preserve its tree.
    reset();
    // SAFETY: Owned mappings/allocator, boot context with interrupts masked.
    let stats = unsafe {
        tasks::run(
            &mut memory.frames,
            Config {
                task_limit: 4,
                frame_limit: 40,
                ..Default::default()
            },
            &[frame_failure, survivor],
        )
    }
    .unwrap();
    assert_eq!(stats.spawned, 2);
    assert_eq!(stats.exits, 2);
    assert_eq!(memory.frames.allocated(), frames);
    // SAFETY: Failed spawns and live tasks have all been reclaimed.
    assert_eq!(unsafe { HEAP.used() }, heap);

    // Repeated sessions also prove callback/context teardown is complete.
    for _ in 0..2 {
        reset();
        // SAFETY: Owned mappings and reclaiming allocator, sole CPU, IRQs masked.
        let stats = unsafe {
            tasks::run(
                &mut memory.frames,
                Config {
                    task_limit: 4,
                    ..Default::default()
                },
                &[controller, survivor],
            )
        }
        .unwrap();
        assert_eq!(stats.spawned, 2 + ROUNDS * 2);
        assert_eq!(stats.exits, stats.spawned);
        assert!(stats.preemptions[1] > 0);
        assert_eq!(CHILDREN.load(Ordering::Acquire), ROUNDS * 2);
        assert_eq!(memory.frames.allocated(), frames);
        // SAFETY: No session, stack or saved task context remains live.
        assert_eq!(unsafe { HEAP.used() }, heap);
    }
    writeln!(
        console,
        "Cathedral Rust lab: dynamic spawn limit stale IDs and live-peer progress passed"
    )
    .ok();
    writeln!(
        console,
        "Cathedral Rust lab: task stacks unmapped; heap and physical frames returned to baseline"
    )
    .ok();
}

fn reset() {
    RELEASE.store(false, Ordering::Release);
    DONE.store(false, Ordering::Release);
    PEER_READY.store(false, Ordering::Release);
    PEER_PROGRESS.store(0, Ordering::Release);
    CHILDREN.store(0, Ordering::Release);
}

fn frame_failure() {
    while !PEER_READY.load(Ordering::Acquire) {
        tasks::yield_now();
    }
    let frames = tasks::allocated_frames();
    // SAFETY: Heap usage is sampled through its interrupt-masked lock.
    let heap = unsafe { HEAP.used() };
    for _ in 0..4 {
        let progress = PEER_PROGRESS.load(Ordering::Acquire);
        assert_eq!(tasks::spawn(never_run), Err(SpawnError::OutOfFrames));
        tasks::sleep(1);
        assert!(PEER_PROGRESS.load(Ordering::Acquire) > progress);
        assert_eq!(tasks::allocated_frames(), frames);
        // SAFETY: Peer allocations are stable, failed admission has rolled back.
        assert_eq!(unsafe { HEAP.used() }, heap);
    }
    DONE.store(true, Ordering::Release);
}

fn heap_failure() {
    use core::alloc::{GlobalAlloc, Layout};
    let layout = Layout::from_size_align(128, 16).unwrap();
    let mut blocks = [core::ptr::null_mut(); 512];
    let frames = tasks::allocated_frames();
    // SAFETY: Keep each successful allocation uniquely owned and free using the
    // same layout. Context switches allocate nothing; the peer's data is stable.
    unsafe {
        for block in &mut blocks {
            *block = HEAP.alloc(layout);
            if block.is_null() {
                break;
            }
        }
        assert!(
            blocks.last().unwrap().is_null(),
            "heap exhaustion probe too small"
        );
        assert_eq!(tasks::spawn(never_run), Err(SpawnError::OutOfHeap));
        assert_eq!(tasks::allocated_frames(), frames);
        for block in blocks {
            if !block.is_null() {
                HEAP.dealloc(block, layout);
            }
        }
    }
}

fn never_run() {
    panic!("failed admission started a task");
}

fn survivor() {
    let retained: Vec<u64> = (0..512).map(|i| i ^ 0x55aa).collect();
    PEER_READY.store(true, Ordering::Release);
    // Deliberately never yields: controller and children require timer preemption.
    while !DONE.load(Ordering::Acquire) {
        PEER_PROGRESS.fetch_add(1, Ordering::Relaxed);
        core::hint::black_box(&retained);
    }
    assert_eq!(retained[511], 511 ^ 0x55aa);
}

fn controller() {
    while !PEER_READY.load(Ordering::Acquire) {
        tasks::yield_now();
    }
    let self_id = tasks::current_id();
    let frames = tasks::allocated_frames();
    // SAFETY: Heap reports usage under its IRQ-masked allocator lock.
    let heap = unsafe { HEAP.used() };
    heap_failure();
    // SAFETY: Probe returned every raw allocation; the peer's allocation is stable.
    assert_eq!(unsafe { HEAP.used() }, heap);
    let mut previous = None;
    for _ in 0..ROUNDS {
        let before = PEER_PROGRESS.load(Ordering::Acquire);
        RELEASE.store(false, Ordering::Release);
        let first = tasks::spawn(child).unwrap();
        let second = tasks::spawn(child).unwrap();
        assert_eq!(tasks::spawn(never_run), Err(SpawnError::TaskLimit));
        assert!(tasks::is_alive(first) && tasks::is_alive(second));
        if let Some(old) = previous {
            assert_ne!(first, old);
            assert!(!tasks::is_alive(old));
        }
        tasks::sleep(2);
        assert!(PEER_PROGRESS.load(Ordering::Acquire) > before);
        RELEASE.store(true, Ordering::Release);
        while tasks::is_alive(first) || tasks::is_alive(second) {
            tasks::yield_now();
        }
        assert_eq!(tasks::current_id(), self_id);
        assert_eq!(tasks::allocated_frames(), frames);
        // SAFETY: Survivor's allocation is stable; children have returned and reaped.
        assert_eq!(unsafe { HEAP.used() }, heap);
        previous = Some(first);
    }
    DONE.store(true, Ordering::Release);
}

fn child() {
    let stack = tasks::current_stack();
    assert!((stack.bottom..stack.top).contains(&arch::stack_pointer()));
    let values: Vec<u64> = (0..256).map(|value| value ^ stack.bottom).collect();
    let canary = [0x9e37_79b9_7f4a_7c15u64; 16];
    while !RELEASE.load(Ordering::Acquire) {
        tasks::sleep(1);
    }
    assert_eq!(values[255], 255 ^ stack.bottom);
    assert_eq!(core::hint::black_box(canary), [0x9e37_79b9_7f4a_7c15; 16]);
    CHILDREN.fetch_add(1, Ordering::Release);
}
