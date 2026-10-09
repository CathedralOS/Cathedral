//! On-demand 64-KiB stacks in a dedicated PML4 branch. Each slot occupies its
//! own 2-MiB range, so its leaf table can be reclaimed independently of peers.

use super::{MemoryError, StackRange};
use ::x86_64::{
    PhysAddr, VirtAddr,
    instructions::tlb,
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, FrameDeallocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags,
        PhysFrame, Size4KiB, Translate, mapper::CleanUp,
    },
};

pub(super) const TASK_BASE: u64 = 0xffff_a000_0000_0000;
pub const MAX_TASK_SLOTS: usize = 64;
const STRIDE: u64 = 2 * 1024 * 1024;
const PAGE_BYTES: u64 = 4096;
const STACK_PAGES: u64 = 16;

/// # Safety
/// Allocations are unique, unused, identity-mapped writable RAM. Released frames
/// become available again only after the caller has removed every live use.
pub unsafe trait StackFrames {
    fn allocate(&mut self) -> Option<u64>;
    /// # Safety
    /// Address came from this source. All live users and stack/table translations
    /// are retired; the permanent identity alias remains under allocator custody.
    unsafe fn release(&mut self, address: u64);
}

struct Tables<'a, F>(&'a mut F);
// SAFETY: StackFrames guarantees unique unused physical frames.
unsafe impl<F: StackFrames> FrameAllocator<Size4KiB> for Tables<'_, F> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.0.allocate().map(frame)
    }
}
fn frame(address: u64) -> PhysFrame {
    assert!(address != 0 && address < (1 << 48));
    PhysFrame::from_start_address(PhysAddr::new(address)).expect("unaligned frame")
}

/// Geometry only; a slot has no backing until allocate_stack succeeds.
pub fn task_stack_range(slot: usize) -> StackRange {
    assert!(slot < MAX_TASK_SLOTS);
    let guard = TASK_BASE + slot as u64 * STRIDE;
    StackRange {
        guard,
        bottom: guard + PAGE_BYTES,
        top: guard + (STACK_PAGES + 1) * PAGE_BYTES,
    }
}

unsafe fn mapper<'a>() -> OffsetPageTable<'a> {
    // SAFETY: Callers serialize access to the active, owned, identity-mapped tree.
    unsafe {
        let root = &mut *(Cr3::read().0.start_address().as_u64() as *mut PageTable);
        OffsetPageTable::new(root, VirtAddr::zero())
    }
}
fn page(address: u64) -> Page<Size4KiB> {
    Page::from_start_address(VirtAddr::new(address)).unwrap()
}

/// Map fresh NX backing, leaving a guard at both ends. Failure rolls back both
/// data and intermediate table frames, including partial page-table creation.
/// # Safety
/// Sole CPU, IRQs off, owned active tree with identity mapping. This slot belongs
/// exclusively to the caller and has no mappings or users. No competing mapper.
pub unsafe fn allocate_stack(
    slot: usize,
    frames: &mut impl StackFrames,
) -> Result<StackRange, MemoryError> {
    let stack = task_stack_range(slot);
    // SAFETY: Caller owns serialized access to the active tree.
    let mut mapper = unsafe { mapper() };
    for address in (stack.guard..=stack.top).step_by(PAGE_BYTES as usize) {
        if mapper.translate_addr(VirtAddr::new(address)).is_some() {
            return Err(MemoryError::OccupiedVirtualRange);
        }
    }
    let mut mapped = 0;
    let result = (|| {
        for address in (stack.bottom..stack.top).step_by(PAGE_BYTES as usize) {
            let backing = frames.allocate().ok_or(MemoryError::OutOfFrames)?;
            // SAFETY: Fresh RAM, no live users; inherited identity mapping is valid.
            unsafe {
                core::ptr::write_bytes(backing as *mut u8, 0, PAGE_BYTES as usize);
            }
            let flags =
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;
            // SAFETY: Vacant virtual page and fresh exclusively owned frame.
            match unsafe {
                mapper.map_to(page(address), frame(backing), flags, &mut Tables(frames))
            } {
                Ok(flush) => {
                    flush.flush();
                    mapped += 1;
                }
                Err(_) => {
                    // SAFETY: map_to failed without installing this leaf.
                    unsafe {
                        frames.release(backing);
                    }
                    return Err(MemoryError::OutOfFrames);
                }
            }
        }
        Ok(stack)
    })();
    if result.is_err() {
        // SAFETY: Only this call's successfully installed leaves are removed.
        unsafe {
            retire(&mut mapper, stack, mapped, frames);
        }
    }
    result
}

/// # Safety
/// Same machine guarantees as allocate_stack. The task has switched away and
/// will never resume; no reference or saved context may still use this stack.
pub unsafe fn release_stack(slot: usize, frames: &mut impl StackFrames) {
    // SAFETY: Caller has retired all stack users and owns the mapping tree.
    unsafe {
        retire(&mut mapper(), task_stack_range(slot), STACK_PAGES, frames);
    }
}

// A single 2-MiB slot can leave at most three empty intermediate tables.
#[derive(Default)]
struct RetiredTables {
    addresses: [u64; 3],
    len: usize,
}
impl FrameDeallocator<Size4KiB> for RetiredTables {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame) {
        self.addresses[self.len] = frame.start_address().as_u64();
        self.len += 1;
    }
}

unsafe fn retire(
    mapper: &mut OffsetPageTable<'_>,
    stack: StackRange,
    count: u64,
    frames: &mut impl StackFrames,
) {
    for index in 0..count {
        let (backing, flush) = mapper
            .unmap(page(stack.bottom + index * PAGE_BYTES))
            .expect("lost stack mapping");
        flush.flush();
        // SAFETY: Leaf removed and local TLB invalidated; no other CPU or users.
        unsafe {
            frames.release(backing.start_address().as_u64());
        }
    }
    let mut tables = RetiredTables::default();
    // SAFETY: Only this dedicated slot's exclusively owned tables are traversed.
    // Shared nonempty ancestors are retained. No inherited table is in this branch.
    unsafe {
        mapper.clean_up_addr_range(
            Page::range_inclusive(page(stack.guard), page(stack.top)),
            &mut tables,
        );
    }
    tlb::flush_all();
    for address in &tables.addresses[..tables.len] {
        // SAFETY: Empty tables are detached and paging-structure caches flushed.
        unsafe {
            frames.release(*address);
        }
    }
    for address in (stack.guard..=stack.top).step_by(PAGE_BYTES as usize) {
        assert!(mapper.translate_addr(VirtAddr::new(address)).is_none());
    }
}
