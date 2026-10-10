//! Smoke-only deterministic failure injection against real physical frames and page tables.
use super::super::{Config, admission, prepare};
use super::*;
use crate::{
    extent::FrameAllocator,
    user_tasks::{Executable, Program},
};
use cathedral_arch::{BootLayout, ImageRange};
/// Exercise runtime allocation rollback and admission of the reserved region table.
/// # Safety
/// Sole boot CPU, IRQs off, no active tasks, live kernel layout/image, initialized heap.
pub unsafe fn exercise(frames: &mut FrameAllocator, layout: &BootLayout, image: ImageRange) {
    let baseline = frames.allocated();
    let config = Config {
        frame_limit: usize::MAX,
        endpoints: &[],
        links: &[],
        supervision: &[],
        clock_readers: &[],
        memory: &[wire::Grant {
            task: 0,
            private_pages: 4,
            shared_pages: 0,
        }],
    };
    let program = Program {
        executable: Executable::Probe(&[0xcc]),
        arguments: [0, 0],
    };
    let (mut session, _) = prepare::session(1, 1, &config, baseline, |_| true).unwrap();
    let mut source = Frames {
        frames,
        remaining: usize::MAX,
    };
    // SAFETY: Same machine custody; root remains inactive throughout this fixture.
    unsafe { admission::admit(&mut session, layout, image, &program, &mut source, 0) }.unwrap();
    let admitted = source.frames.allocated();
    let table_budget = admitted - baseline;
    for budget in 0..=wire::MAX_PAGES {
        source.remaining = budget;
        // SAFETY: Inactive prepared root and exclusive backing ownership.
        let result = unsafe { allocation::allocate(&mut session, &mut source, 0, wire::MAX_PAGES) };
        if budget < wire::MAX_PAGES {
            assert_eq!(result, Err(abi::NO_MEMORY));
        } else {
            let handle = result.unwrap();
            assert_eq!(source.frames.allocated(), admitted + wire::MAX_PAGES);
            // SAFETY: Root never ran; no user borrows or outstanding peer references.
            unsafe {
                dispatch(
                    &mut session,
                    &mut source,
                    0,
                    [abi::MEMORY_RELEASE, handle, 0],
                )
            }
            .unwrap();
        }
        assert_eq!(source.frames.allocated(), admitted);
        assert!(
            session
                .memory
                .model
                .entries
                .iter()
                .all(|region| !region.live())
        );
        assert_eq!(session.memory.frames(), 0);
    }
    // SAFETY: Never activated, no remaining borrowed regions.
    unsafe { admission::retire(&mut session.tasks[0], &mut source) };
    assert_eq!(source.frames.allocated(), baseline);
    // Exercise every admission boundary too, including the newly reserved PT.
    for budget in 0..table_budget {
        source.remaining = budget;
        assert!(
            // SAFETY: Previous admission completely retired; no published context.
            unsafe { admission::admit(&mut session, layout, image, &program, &mut source, 0) }
                .is_err()
        );
        assert_eq!(source.frames.allocated(), baseline);
        assert!(session.tasks[0].space.is_none());
    }
}
