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
/// Any framebuffer is a reserved boot aperture with exclusive custody for the fixture.
/// Output must be bounded, allocation-free and unable to suspend.
pub unsafe fn exercise(
    frames: &mut FrameAllocator,
    layout: &BootLayout,
    image: ImageRange,
    framebuffer: Option<cathedral_kernel_contracts::framebuffer::Framebuffer>,
    output: fn(&[u8]) -> bool,
) {
    // SAFETY: Independent fixtures return every frame before the next starts.
    // Keep their large session temporaries in separate, non-inlined stack frames.
    unsafe {
        admission::rollback::exercise(frames, layout, image, framebuffer, output);
        page_objects(frames, layout, image, output);
    }
}

#[inline(never)]
unsafe fn page_objects(
    frames: &mut FrameAllocator,
    layout: &BootLayout,
    image: ImageRange,
    output: fn(&[u8]) -> bool,
) {
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
            private_pages: wire::MAX_PRIVATE_PAGES,
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
    // Reserving the same empty window twice must not allocate tables or publish
    // leaves. Subsequent map_region calls use NoFrames and cover every slot.
    // SAFETY: Prepared inactive tree; no runtime region has been installed yet.
    unsafe {
        session.tasks[0]
            .space
            .as_mut()
            .unwrap()
            .prepare_regions(&mut source)
    }
    .unwrap();
    assert_eq!(source.allocated(), admitted);
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
    // Cancel at every suspension boundary of a maximum-size allocation. A
    // staged object reserves all its funding but exposes no partially zeroed map.
    for steps in 1..wire::MAX_PAGES.div_ceil(allocation::PAGE_BATCH) {
        source.remaining = usize::MAX;
        for step in 1..=steps {
            assert_eq!(
                // SAFETY: Inactive prepared root and exclusive frame custody.
                unsafe { allocation::step(&mut session, &mut source, 0, wire::MAX_PAGES) },
                None
            );
            assert_eq!(source.allocated(), admitted + step * allocation::PAGE_BATCH);
            assert_eq!(
                session.memory.funding.charged(),
                table_budget + wire::MAX_PAGES
            );
            assert_eq!(
                session.memory.reserved_unbacked(),
                wire::MAX_PAGES - step * allocation::PAGE_BATCH
            );
            assert!(
                !session.tasks[0]
                    .space
                    .as_ref()
                    .unwrap()
                    .writable_range(wire::address(0), 1)
            );
        }
        // SAFETY: Same cleanup used for a dead blocked caller; staging has no aliases.
        unsafe {
            closing::close(&mut session, 0, &mut source);
        }
        assert_eq!(session.memory.reserved_unbacked(), 0);
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
    }
    // SAFETY: Prior objects retired; same inactive root and real frame custody.
    unsafe { super::references_test::exercise(&mut session, &mut source, output) };
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

/// Exercise every reserved slot, including regions crossing leaf-table boundaries.
/// These inactive aliases are removed before the model releases the backing.
unsafe fn mapping_boundaries(session: &mut Session, handle: u64) {
    let index = session.memory.model.access(0, handle).unwrap();
    let backing = session.memory.backing[index].clone();
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
    for alias in (0..wire::MAX_MAPPING_SLOTS).filter(|&slot| slot != mapping) {
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
        for page in 0..wire::MAX_PAGES {
            let offset = (page * wire::PAGE_BYTES) as u64;
            // SAFETY: Validation must reject every removed page, not just the first.
            assert!(!unsafe { space.copy_from_user(wire::address(alias) + offset, &mut [0]) });
            let mut byte = [0];
            // SAFETY: The original sealed owner alias must survive compaction intact.
            assert!(unsafe { space.copy_from_user(wire::address(mapping) + offset, &mut byte) });
            assert_eq!(byte, [page as u8 + 1]);
            assert!(!space.writable_range(wire::address(mapping) + offset, 1));
        }
        assert!(space.writable_range(super::super::address_space::USER_STACK, 1));
    }
}
