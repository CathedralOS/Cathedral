//! Sparse, independently owned task tables; no firmware mapping tree is copied.
//! User code is RX; private data and guarded stack are RW/NX. Kernel mappings
//! are supervisor-only. Physical aliases are available only in the kernel root.
mod device;

use super::super::{BootLayout, MemoryError, StackFrames};
use ::x86_64::{
    PhysAddr, VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags as Flags,
        PhysFrame, Size4KiB, Translate, mapper::TranslateResult,
    },
};

const PAGE: u64 = 4096;
pub const USER_CODE: u64 = 0x0000_0080_0000_0000;
pub const USER_DATA: u64 = USER_CODE + PAGE;
pub const USER_IMAGE_END: u64 = USER_CODE + 1024 * 1024;
pub const USER_STACK: u64 = USER_CODE + 2 * 1024 * 1024;
pub const USER_STACK_TOP: u64 = USER_STACK + 4 * PAGE;
pub const MAX_IMAGE_PAGES: usize = 64;
const _: () = assert!(MAX_IMAGE_PAGES + 4 <= u128::BITS as usize);
const MAX_FRAMES: usize = 128;

#[derive(Clone, Copy, Debug)]
pub struct UserSegment<'a> {
    pub address: u64,
    pub bytes: &'a [u8],
    pub memory_size: u64,
    pub writable: bool,
    pub executable: bool,
}

#[derive(Clone, Copy, Default)]
struct Backing {
    address: u64,
    physical: u64,
}

#[derive(Clone, Copy)]
pub struct ImageRange {
    pub base: u64,
    pub bytes: u64,
}

pub struct UserSpace {
    root: u64,
    owned: [u64; MAX_FRAMES],
    len: usize,
    backing: [Backing; MAX_IMAGE_PAGES + 4],
    pages: usize,
    writable: u128,
}
impl UserSpace {
    pub fn root(&self) -> u64 {
        self.root
    }
    pub fn frame_count(&self) -> usize {
        self.len
    }

    /// # Safety
    /// Kernel root active, IRQs off, exclusive frame custody. The supplied image
    /// and BootLayout are live supervisor resources from this boot. No mappings
    /// change until the resulting task and its entry paths have been retired.
    pub unsafe fn create(
        layout: &BootLayout,
        image: ImageRange,
        code: &[u8],
        frames: &mut impl StackFrames,
    ) -> Result<Self, MemoryError> {
        if code.is_empty() || code.len() > PAGE as usize {
            return Err(MemoryError::InvalidFrame);
        }
        let segments = [
            UserSegment {
                address: USER_CODE,
                bytes: code,
                memory_size: PAGE,
                writable: false,
                executable: true,
            },
            UserSegment {
                address: USER_DATA,
                bytes: &[],
                memory_size: PAGE,
                writable: true,
                executable: false,
            },
        ];
        // SAFETY: Same caller obligations; raw probes use two fixed disjoint pages.
        unsafe { Self::from_segments(layout, image, &segments, frames) }
    }

    /// Build a bounded image from page-disjoint, immutable segment descriptions.
    /// # Safety
    /// Same machine/ownership obligations as create. Segment bytes remain live
    /// for this call; kernel image and layout retain their boot-time mappings.
    pub unsafe fn from_segments(
        layout: &BootLayout,
        image: ImageRange,
        segments: &[UserSegment<'_>],
        frames: &mut impl StackFrames,
    ) -> Result<Self, MemoryError> {
        if image.bytes == 0 || !image.base.is_multiple_of(PAGE) || !valid_segments(segments) {
            return Err(MemoryError::InvalidFrame);
        }
        let end = image
            .base
            .checked_add(image.bytes)
            .and_then(|end| end.checked_add(PAGE - 1))
            .ok_or(MemoryError::InvalidFrame)?
            & !(PAGE - 1);
        // Keep the low identity-loaded image disjoint from the user branch.
        if end > USER_CODE {
            return Err(MemoryError::OccupiedVirtualRange);
        }
        let mut space = Self {
            root: 0,
            owned: [0; MAX_FRAMES],
            len: 0,
            backing: [Backing::default(); MAX_IMAGE_PAGES + 4],
            pages: 0,
            writable: 0,
        };
        // SAFETY: New frames are exclusively owned and identity-mapped in kernel CR3.
        let result = unsafe { space.build(layout, image.base, end, segments, frames) };
        if let Err(error) = result {
            // SAFETY: This root was never activated, including partially built tables.
            unsafe {
                space.release(frames);
            }
            return Err(error);
        }
        Ok(space)
    }

    unsafe fn build(
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
                    map(&mut target, address, physical, flags, &mut source)?;
                }
            }
        }
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
                self.backing[self.pages] = Backing { address, physical };
                if segment.writable {
                    self.writable |= 1 << self.pages;
                }
                self.pages += 1;
                // SAFETY: Fresh zeroed backing, disjoint validated virtual page.
                // Only file bytes are copied; BSS and trailing page bytes stay zero.
                unsafe {
                    map(&mut target, address, physical, flags, &mut source)?;
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

    /// Copy through owned physical backing; never dereference an unchecked user VA.
    /// This deliberately bounded ABI accepts a buffer within a single user page.
    /// # Safety
    /// Kernel root active, IRQs off, this space live and not concurrently running.
    pub unsafe fn copy_from_user(&self, address: u64, output: &mut [u8]) -> bool {
        let Some(end) = address.checked_add(output.len() as u64) else {
            return false;
        };
        for &Backing {
            address: base,
            physical,
        } in &self.backing[..self.pages]
        {
            if address >= base && address < base + PAGE && end <= base + PAGE {
                // SAFETY: Checked full range is inside one live, owned user frame.
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        (physical + address - base) as *const u8,
                        output.as_mut_ptr(),
                        output.len(),
                    );
                }
                return true;
            }
        }
        false
    }

    /// Validate the entire destination before consuming a message or parking.
    pub fn writable_range(&self, address: u64, length: usize) -> bool {
        self.write_backing(address, length).is_some()
    }
    fn write_backing(&self, address: u64, length: usize) -> Option<u64> {
        let end = address.checked_add(length as u64)?;
        self.backing[..self.pages]
            .iter()
            .enumerate()
            .find(|(index, page)| {
                self.writable & (1 << index) != 0
                    && address >= page.address
                    && address < page.address + PAGE
                    && end <= page.address + PAGE
            })
            .map(|(_, page)| page.physical + address - page.address)
    }
    /// # Safety
    /// Kernel root active, IRQs off, this space live and not concurrently running.
    pub unsafe fn copy_to_user(&self, address: u64, bytes: &[u8]) -> bool {
        let Some(physical) = self.write_backing(address, bytes.len()) else {
            return false;
        };
        // SAFETY: Full range is in a live owned writable user page; kernel buffer is disjoint.
        unsafe {
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), physical as *mut u8, bytes.len());
        }
        true
    }

    /// # Safety
    /// Kernel root active, IRQs off. Task can never resume; no saved references
    /// or mappings outside this retired root refer to these owned frames.
    pub unsafe fn release(self, frames: &mut impl StackFrames) {
        assert_ne!(Cr3::read().0.start_address().as_u64(), self.root);
        for address in self.owned[..self.len].iter().rev() {
            // SAFETY: Inactive root; CR3 switch flushed its non-global translations.
            unsafe {
                frames.release(*address);
            }
        }
    }
}

/// Geometry/permission check, independent of live paging and physical allocation.
pub fn valid_segments(segments: &[UserSegment<'_>]) -> bool {
    if segments.is_empty() || segments.len() > 8 {
        return false;
    }
    let mut pages = 0;
    for (index, segment) in segments.iter().enumerate() {
        if segment.memory_size == 0
            || segment.memory_size > USER_IMAGE_END - USER_CODE
            || segment.bytes.len() as u64 > segment.memory_size
            || segment.address < USER_CODE
            || !segment.address.is_multiple_of(PAGE)
            || (segment.writable && segment.executable)
        {
            return false;
        }
        let Some(end) = segment
            .address
            .checked_add(segment.memory_size.div_ceil(PAGE) * PAGE)
        else {
            return false;
        };
        if end > USER_IMAGE_END {
            return false;
        }
        pages += segment.memory_size.div_ceil(PAGE) as usize;
        if pages > MAX_IMAGE_PAGES {
            return false;
        }
        for previous in &segments[..index] {
            let previous_end = previous.address + previous.memory_size.div_ceil(PAGE) * PAGE;
            if segment.address < previous_end && previous.address < end {
                return false;
            }
        }
    }
    true
}

struct Tracked<'a, F> {
    frames: &'a mut F,
    addresses: &'a mut [u64; MAX_FRAMES],
    len: &'a mut usize,
}
// SAFETY: StackFrames supplies unique unused physical frames. Ledger retains all
// backing and intermediate tables, including those from partial failed mappings.
unsafe impl<F: StackFrames> FrameAllocator<Size4KiB> for Tracked<'_, F> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        if *self.len == MAX_FRAMES {
            return None;
        }
        let address = self.frames.allocate()?;
        assert!(address != 0 && address < (1 << 48) && address.is_multiple_of(PAGE));
        self.addresses[*self.len] = address;
        *self.len += 1;
        // SAFETY: Fresh physical frame is writable through kernel identity mapping.
        unsafe {
            core::ptr::write_bytes(address as *mut u8, 0, PAGE as usize);
        }
        Some(PhysFrame::from_start_address(PhysAddr::new(address)).unwrap())
    }
}
unsafe fn map(
    mapper: &mut OffsetPageTable<'_>,
    address: u64,
    physical: u64,
    flags: Flags,
    frames: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MemoryError> {
    // SAFETY: Caller owns the inactive hierarchy and provides valid backing and permissions.
    unsafe {
        mapper.map_to(
            Page::from_start_address(VirtAddr::new(address)).unwrap(),
            PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(physical)).unwrap(),
            flags,
            frames,
        )
    }
    .map_err(|_| MemoryError::OutOfFrames)?
    .ignore();
    Ok(())
}
