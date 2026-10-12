//! Smoke-only deterministic failure injection against real physical frames and page tables.
use super::super::{Config, admission, startup};
use super::*;
use crate::{
    memory::FrameAllocator,
    user_tasks::{Executable, Program},
};
use cathedral_arch::{BootLayout, ImageRange};
/// Exercise runtime allocation rollback and admission of the reserved region table.
/// # Safety
/// Sole boot CPU, IRQs off, no active tasks, live kernel layout/image, initialized heap.
pub unsafe fn exercise(frames: &mut FrameAllocator, layout: &BootLayout, image: ImageRange) {
    let baseline = frames.allocated();
    let config = Config {
        executables: &[],
        dynamic_tasks: 0,
        frame_limit: usize::MAX,
        endpoints: &[],
        links: &[],
        supervision: &[],
        channel_limits: &[],
        cpu_limits: &[],
        clock_readers: &[],
        memory: &[wire::Grant {
            task: 0,
            private_pages: wire::MAX_PAGES,
            shared_pages: 0,
        }],
    };
    let program = Program {
        executable: Executable::Probe(&[0xcc]),
        arguments: [0, 0],
    };
    let (mut session, _) = startup::prepare(1, &config, baseline, |_| true).unwrap();
    let mut source = Frames::new(frames, session.memory.funding.capacity(), usize::MAX).unwrap();
    // SAFETY: Same machine custody; root remains inactive throughout this fixture.
    unsafe { admission::admit(&mut session, layout, image, &program, &mut source, 0) }.unwrap();
    let admitted = source.allocated();
    let table_budget = admitted - baseline;
    for budget in 0..=wire::MAX_PAGES {
        source.remaining = budget;
        // SAFETY: Inactive prepared root and exclusive backing ownership.
        let result = unsafe { allocation::allocate(&mut session, &mut source, 0, wire::MAX_PAGES) };
        if budget < wire::MAX_PAGES {
            assert_eq!(result, Err(abi::NO_MEMORY));
        } else {
            let handle = result.unwrap();
            assert_eq!(source.allocated(), admitted + wire::MAX_PAGES);
            // SAFETY: Root has never executed and all frames remain pinned here.
            unsafe { mapping_boundaries(&mut session, handle) };
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
        assert_eq!(source.allocated(), admitted);
        assert_eq!(session.memory.funding.charged(), table_budget);
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
    unsafe { admission::retire(&mut session, 0, &mut source) };
    assert_eq!(source.allocated(), baseline);
    let capacity = session.memory.funding.capacity();
    for budget in 0..table_budget {
        session
            .memory
            .funding
            .assign(0, 1, capacity - budget)
            .unwrap();
        source.remaining = usize::MAX;
        assert!(
            // SAFETY: Inactive account, serialized bank; physical funding alone limits the load.
            unsafe { admission::admit(&mut session, layout, image, &program, &mut source, 0) }
                .is_err()
        );
        assert_eq!(source.allocated(), baseline);
        assert_eq!(session.memory.funding.charged(), 0);
    }
    session.memory.funding.assign(0, 1, 0).unwrap();
    // Exercise every admission boundary too, including all reserved region PTs.
    for budget in 0..table_budget {
        source.remaining = budget;
        assert!(
            // SAFETY: Previous admission completely retired; no published context.
            unsafe { admission::admit(&mut session, layout, image, &program, &mut source, 0) }
                .is_err()
        );
        assert_eq!(source.allocated(), baseline);
        assert!(session.tasks[0].space.is_none());
    }
}

/// Exercise a region crossing a 2-MiB table boundary and the last virtual slot.
/// These inactive aliases are removed before the model releases the backing.
unsafe fn mapping_boundaries(session: &mut Session, handle: u64) {
    let index = session.memory.model.access(0, handle).unwrap();
    let backing = session.memory.backing[index];
    let mapping = session.memory.model.mapping(0, index);
    let space = session.tasks[0].space.as_mut().unwrap();
    for page in 0..wire::MAX_PAGES {
        // SAFETY: Full byte range in the prepared owner's writable mapping.
        assert!(unsafe {
            space.copy_to_user(
                wire::address(mapping) + (page * wire::PAGE_BYTES) as u64,
                &[page as u8 + 1],
            )
        });
    }
    // SAFETY: No executing user or Rust borrow; remove writer before reader aliases.
    unsafe { space.seal_region(mapping, wire::MAX_PAGES) };
    assert!(!space.writable_range(wire::address(mapping), 1));
    for alias in [30, wire::MAX_REGIONS - 1] {
        // SAFETY: Prepared empty slot; immutable frames stay owned until all aliases are gone.
        unsafe { space.map_region(alias, &backing, false) }.unwrap();
        for page in 0..wire::MAX_PAGES {
            let address = wire::address(alias) + (page * wire::PAGE_BYTES) as u64;
            let mut byte = [0];
            // SAFETY: Inactive live reader mapping into the retained allocation.
            assert!(unsafe { space.copy_from_user(address, &mut byte) });
            assert_eq!(byte, [page as u8 + 1]);
            assert!(!space.writable_range(address, 1));
        }
        let guard = wire::address(alias) + (wire::MAX_PAGES * wire::PAGE_BYTES) as u64;
        // SAFETY: copy validates the unmapped guard and does not dereference it.
        assert!(!unsafe { space.copy_from_user(guard, &mut [0]) });
        // SAFETY: No reader can execute; remove borrowed aliases before owner release.
        unsafe { space.unmap_region(alias, wire::MAX_PAGES) };
        // SAFETY: Validation must reject the entire retired alias.
        assert!(!unsafe { space.copy_from_user(wire::address(alias), &mut [0]) });
    }
}
