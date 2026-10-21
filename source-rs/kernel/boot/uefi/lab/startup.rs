//! Refuse startup under heap pressure without publishing a task or leaking arenas.
use crate::{console::Console, heap::HEAP, memory::PreparedMemory};
use cathedral_core::user_tasks::{self, Error, Executable, Program};
use core::{
    alloc::{GlobalAlloc, Layout},
    fmt::Write,
};

pub fn exercise(memory: &mut PreparedMemory, console: &mut Console) {
    let frames = memory.frames.allocated();
    let layout = Layout::from_size_align(4096, 16).unwrap();
    let mut blocks: *mut u8 = core::ptr::null_mut();
    let mut refusals = 0;
    // The code is never entered: zero frame funding stops admission after all
    // startup allocations succeed. Its bytes need not target the current ISA.
    let programs = [Program {
        executable: Executable::Probe(&[0xcc]),
        arguments: [0, 0],
    }];
    // SAFETY: Sole boot CPU, outside any task session. Own each raw block and
    // free it with its original layout; the intrusive list needs no allocation.
    unsafe {
        let heap = HEAP.used();
        loop {
            let block = HEAP.alloc(layout);
            if block.is_null() {
                break;
            }
            block.cast::<*mut u8>().write(blocks);
            blocks = block;
        }
        loop {
            let held = HEAP.used();
            let result = user_tasks::run(
                &mut memory.frames,
                &memory.layout,
                memory.image,
                &programs,
                |_| true,
                0,
            );
            let reached_admission = match result {
                Err(Error::OutOfHeap) => {
                    refusals += 1;
                    false
                }
                Err(Error::Memory(cathedral_arch::MemoryError::OutOfFrames)) => true,
                other => panic!("unexpected constrained startup result: {other:?}"),
            };
            assert_eq!(HEAP.used(), held, "startup refusal leaked heap storage");
            assert_eq!(memory.frames.allocated(), frames);
            if reached_admission {
                break;
            }
            assert!(!blocks.is_null(), "startup cannot fit the installed heap");
            let block = blocks;
            blocks = block.cast::<*mut u8>().read();
            HEAP.dealloc(block, layout);
        }
        while !blocks.is_null() {
            let block = blocks;
            blocks = block.cast::<*mut u8>().read();
            HEAP.dealloc(block, layout);
        }
        assert_eq!(HEAP.used(), heap);
    }
    assert!(refusals > 0);
    writeln!(
        console,
        "Cathedral Rust lab: session startup heap refusals={refusals}; exact reclamation passed"
    )
    .ok();
}
