//! Guarded EL1 task stacks in a dedicated root branch. Each slot owns a 2-MiB leaf table.
//! Remove mappings and flush before refunding backing; retain nonempty shared ancestors.
use super::memory::tables;
use crate::{FrameSource, MemoryError, StackRange, memory::PAGE};
const BASE: u64 = 0x0000_0100_0000_0000;
const STRIDE: u64 = 1 << 21;
const PAGES: u64 = 16;
pub const MAX_TASK_SLOTS: usize = 64;

pub fn task_stack_range(slot: usize) -> StackRange {
    assert!(slot < MAX_TASK_SLOTS);
    let guard = BASE + slot as u64 * STRIDE;
    StackRange {
        guard,
        bottom: guard + PAGE,
        top: guard + (PAGES + 1) * PAGE,
    }
}

/// # Safety
/// Sole EL1 CPU, IRQs masked, owned active tree; vacant slot with no possible users.
pub unsafe fn allocate_stack(
    slot: usize,
    frames: &mut impl FrameSource,
) -> Result<StackRange, MemoryError> {
    let stack = task_stack_range(slot);
    let root = tables::active();
    // SAFETY: Inspect owned hierarchy without allocating; guards and all leaves must be vacant.
    unsafe {
        for address in (stack.guard..=stack.top).step_by(PAGE as usize) {
            match tables::leaf(root, address, &mut || Err(MemoryError::OutOfFrames)) {
                Ok(entry) if entry.read() & 1 == 0 => {}
                Err(MemoryError::OutOfFrames) => {}
                _ => return Err(MemoryError::OccupiedVirtualRange),
            }
        }
    }
    let mut mapped = 0;
    let result = (|| {
        for address in (stack.bottom..stack.top).step_by(PAGE as usize) {
            let backing = frames.allocate().ok_or(MemoryError::OutOfFrames)?;
            // SAFETY: FrameSource supplies exclusive identity-mapped writable RAM.
            unsafe {
                core::ptr::write_bytes(backing as *mut u8, 0, PAGE as usize);
            }
            let mut next = || tables::PageTables::new(&mut || frames.allocate()).frame();
            // SAFETY: Vacant leaf, owned hierarchy; fresh tables are zeroed before linking.
            let result = unsafe {
                tables::map(
                    root,
                    address,
                    backing,
                    tables::NORMAL | tables::NX,
                    &mut next,
                )
            };
            if let Err(error) = result {
                // SAFETY: Failed map never installed this leaf; partial tables are retired below.
                unsafe {
                    frames.release(backing);
                }
                return Err(error);
            }
            mapped += 1;
        }
        // SAFETY: All mappings installed before the context can run.
        unsafe {
            tables::synchronize();
        }
        Ok(stack)
    })();
    if result.is_err() {
        // SAFETY: New stack never ran; retire partial leaves and any empty intermediate tables.
        unsafe {
            retire(root, stack, mapped, frames);
        }
    }
    result
}

/// # Safety
/// Sole CPU, IRQs off, owned tree. Task has switched away and can never resume.
pub unsafe fn release_stack(slot: usize, frames: &mut impl FrameSource) {
    // SAFETY: Caller retires every saved context before stack backing is returned.
    unsafe {
        retire(tables::active(), task_stack_range(slot), PAGES, frames);
    }
}

unsafe fn retire(root: u64, stack: StackRange, pages: u64, frames: &mut impl FrameSource) {
    // SAFETY: Only this slot's live leaves are removed; backing stays owned through invalidation.
    unsafe {
        for page in 0..pages {
            let entry = tables::leaf(root, stack.bottom + page * PAGE, &mut || {
                Err(MemoryError::OutOfFrames)
            })
            .unwrap();
            let backing = entry.read() & tables::ADDRESS;
            entry.write(0);
            tables::synchronize();
            frames.release(backing);
        }
        let mut path = [(core::ptr::null_mut::<u64>(), 0u64); 3];
        let mut len = 0;
        let mut table = root;
        for shift in [39, 30, 21] {
            let parent = (table as *mut u64).add(((stack.guard >> shift) & 511) as usize);
            let descriptor = parent.read();
            if descriptor & 1 == 0 {
                break;
            }
            assert_eq!(descriptor & 3, 3);
            table = descriptor & tables::ADDRESS;
            path[len] = (parent, table);
            len += 1;
        }
        for &(parent, child) in path[..len].iter().rev() {
            if (0..512).any(|i| (child as *const u64).add(i).read() & 1 != 0) {
                break;
            }
            parent.write(0);
            tables::synchronize();
            frames.release(child);
        }
        for address in (stack.guard..=stack.top).step_by(PAGE as usize) {
            if let Ok(entry) = tables::leaf(root, address, &mut || Err(MemoryError::OutOfFrames)) {
                assert_eq!(entry.read() & 1, 0);
            }
        }
    }
}
