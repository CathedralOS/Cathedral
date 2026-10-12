//! Construct a sparse root: borrow required supervisor mappings, then load private image and stack.
use super::paging::{Tracked, map};
use super::{
    Backing, BootLayout, MAX_BACKING, MemoryError, PAGE, StackFrames, USER_STACK, UserSegment,
    UserSpace,
};
use ::x86_64::{
    VirtAddr,
    structures::paging::{
        FrameAllocator, OffsetPageTable, PageTable, PageTableFlags as Flags, Size4KiB, Translate,
        mapper::TranslateResult,
    },
};

impl UserSpace {
    pub(super) unsafe fn build(
        &mut self,
        layout: &BootLayout,
        start: u64,
        end: u64,
        segments: &[UserSegment<'_>],
        frames: &mut impl StackFrames,
    ) -> Result<(), MemoryError> {
        let mut source = Tracked {
            frames,
            addresses: &mut self.owned,
            len: &mut self.len,
        };
        let root = source.allocate_frame().ok_or(MemoryError::OutOfFrames)?;
        self.root = root.start_address().as_u64();
        // SAFETY: New zeroed root, unique tables; boot root remains active and stable.
        let (mut target, boot) = unsafe {
            (
                OffsetPageTable::new(&mut *(self.root as *mut PageTable), VirtAddr::zero()),
                OffsetPageTable::new(
                    &mut *(layout.root_address() as *mut PageTable),
                    VirtAddr::zero(),
                ),
            )
        };
        // SAFETY: Inactive target and exclusive frame ledger; boot mappings stay live.
        unsafe {
            map_kernel(&mut target, &boot, layout, start, end, &mut source)?;
            map_program(
                &mut target,
                segments,
                &mut self.backing,
                &mut self.pages,
                &mut self.writable,
                &mut source,
            )?;
        }
        Ok(())
    }
}

unsafe fn map_kernel(
    target: &mut OffsetPageTable<'_>,
    boot: &OffsetPageTable<'_>,
    layout: &BootLayout,
    start: u64,
    end: u64,
    source: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MemoryError> {
    let ranges = [
        (start, end),
        (layout.stack_bottom, layout.stack_top),
        (
            layout.heap_start,
            layout.heap_start + layout.heap_bytes as u64,
        ),
    ];
    for (bottom, top) in ranges.into_iter().chain(
        layout
            .emergency_tops
            .iter()
            .map(|top| (top - 4 * PAGE, *top)),
    ) {
        for address in (bottom..top).step_by(PAGE as usize) {
            let TranslateResult::Mapped {
                frame,
                offset,
                flags,
            } = boot.translate(VirtAddr::new(address))
            else {
                return Err(MemoryError::MapFailed);
            };
            let physical = frame.start_address().as_u64() + offset;
            let flags = flags & (Flags::PRESENT | Flags::WRITABLE | Flags::NO_EXECUTE);
            // SAFETY: Explicit live kernel resource; only supervisor leaf permissions.
            unsafe {
                map(target, address, physical, flags, source)?;
            }
        }
    }
    Ok(())
}

unsafe fn map_program(
    target: &mut OffsetPageTable<'_>,
    segments: &[UserSegment<'_>],
    backing: &mut [Backing; MAX_BACKING],
    pages: &mut usize,
    writable: &mut u128,
    source: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MemoryError> {
    let stack = UserSegment {
        address: USER_STACK,
        bytes: &[],
        memory_size: 4 * PAGE,
        writable: true,
        executable: false,
    };
    for segment in segments.iter().chain(core::iter::once(&stack)) {
        let mut flags = Flags::PRESENT | Flags::USER_ACCESSIBLE;
        if segment.writable {
            flags |= Flags::WRITABLE;
        }
        if !segment.executable {
            flags |= Flags::NO_EXECUTE;
        }
        for offset in (0..segment.memory_size).step_by(PAGE as usize) {
            let frame = source.allocate_frame().ok_or(MemoryError::OutOfFrames)?;
            let physical = frame.start_address().as_u64();
            let address = segment.address + offset;
            backing[*pages] = Backing { address, physical };
            if segment.writable {
                *writable |= 1 << *pages;
            }
            *pages += 1;
            // SAFETY: Fresh zeroed backing, disjoint validated virtual page.
            // Only file bytes are copied; BSS and trailing page bytes stay zero.
            unsafe {
                map(target, address, physical, flags, source)?;
                if (offset as usize) < segment.bytes.len() {
                    let bytes = &segment.bytes[offset as usize..];
                    core::ptr::copy_nonoverlapping(
                        bytes.as_ptr(),
                        physical as *mut u8,
                        bytes.len().min(PAGE as usize),
                    );
                }
            }
        }
    }
    Ok(())
}
