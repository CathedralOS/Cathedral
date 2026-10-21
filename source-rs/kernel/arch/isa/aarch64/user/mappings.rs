//! AArch64 inactive translations; regions batches ordinary RAM permission/removal edits.
//! Core owns frame custody and virtual layout.
use super::super::memory::tables;
use crate::{
    BootLayout, FrameSource, ImageRange, MemoryError,
    memory::{PAGE, Permissions},
};
pub struct UserMappings {
    root: u64,
}
fn table_frame(frames: &mut impl FrameSource) -> Result<u64, MemoryError> {
    let p = frames.allocate().ok_or(MemoryError::OutOfFrames)?;
    if !p.is_multiple_of(PAGE)
        || !(crate::board::virt::RAM_START..crate::board::virt::RAM_END).contains(&p)
    {
        return Err(MemoryError::InvalidFrame);
    }
    // SAFETY: FrameSource guarantees fresh exclusively owned identity RAM.
    unsafe {
        core::ptr::write_bytes(p as *mut u8, 0, PAGE as usize);
    }
    Ok(p)
}
impl UserMappings {
    pub fn root(&self) -> u64 {
        self.root
    }
    pub fn assert_inactive(&self) {
        assert_ne!(tables::active(), self.root);
    }
    /// # Safety
    /// Sole CPU, IRQs off, kernel root active; live supervisor boot branch. Caller
    /// owns all returned frames and reclaims them if this construction fails.
    pub unsafe fn create(
        layout: &BootLayout,
        _image: ImageRange,
        frames: &mut impl FrameSource,
    ) -> Result<Self, MemoryError> {
        let root = table_frame(frames)?;
        // SAFETY: New root borrows only the live supervisor branch; never its ownership.
        unsafe {
            (root as *mut u64).write((layout.root_address() as *const u64).read());
        }
        Ok(Self { root })
    }
    /// ARM64 borrows its permanent supervisor branch in create; no leaf copy is needed.
    /// # Safety
    /// Same private inactive root and permanent boot-branch custody as create.
    pub unsafe fn prepare_kernel(
        &mut self,
        _frames: &mut impl FrameSource,
        budget: usize,
    ) -> Result<bool, MemoryError> {
        assert!(budget != 0);
        self.assert_inactive();
        Ok(true)
    }
    /// # Safety
    /// Inactive owned tree, IRQs off; disjoint valid aligned VA/backing. Caller retains
    /// backing and every new table frame, retiring the tree on allocation failure.
    pub unsafe fn map_page(
        &mut self,
        address: u64,
        physical: u64,
        permissions: Permissions,
        frames: &mut impl FrameSource,
    ) -> Result<(), MemoryError> {
        if !physical.is_multiple_of(PAGE)
            || !(crate::board::virt::RAM_START..crate::board::virt::RAM_END).contains(&physical)
        {
            return Err(MemoryError::InvalidFrame);
        }
        let flags = tables::NORMAL
            | tables::USER
            | (1 << 53)
            | if permissions.writable { 0 } else { tables::RO }
            | if permissions.executable {
                0
            } else {
                tables::NX
            };
        // SAFETY: Caller owns inactive translations and all allocated frames.
        unsafe {
            tables::map(self.root, address, physical, flags, &mut || {
                table_frame(frames)
            })
        }
    }
    /// # Safety
    /// Inactive owned tree, IRQs off; vacant page-aligned range in its user branch.
    pub unsafe fn reserve(
        &mut self,
        start: u64,
        end: u64,
        _scratch: u64,
        frames: &mut impl FrameSource,
    ) -> Result<(), MemoryError> {
        // With 4-KiB granules, each leaf table covers 512 pages (2 MiB).
        // One walk creates that whole table; visiting its other leaves adds no
        // reservation. Advance to the boundary, even for an unaligned start.
        const TABLE_BYTES: u64 = 512 * PAGE;
        let mut address = start;
        while address < end {
            // SAFETY: Reserve only tables; no leaf backing is installed. The
            // caller's valid user VA range keeps boundary arithmetic in range.
            unsafe {
                tables::leaf(self.root, address, &mut || table_frame(frames))?;
            }
            address = (address | (TABLE_BYTES - 1)) + 1;
        }
        Ok(())
    }
    /// # Safety
    /// Inactive tree, IRQs off; existing leaf, backing retained until all aliases retire.
    pub unsafe fn unmap_page(&mut self, address: u64) {
        // SAFETY: Remove translation only, never release the caller's backing.
        unsafe {
            self.entry(address).write(0);
        }
    }
    unsafe fn entry(&self, address: u64) -> *mut u64 {
        // SAFETY: Caller requires an existing leaf in the owned inactive tree.
        unsafe { tables::leaf(self.root, address, &mut || Err(MemoryError::OutOfFrames)).unwrap() }
    }
    /// # Safety
    /// Newly loaded executable RAM, no running users or writers until publication.
    pub unsafe fn publish_code(physical: u64) {
        // SAFETY: Caller keeps code private until instruction/data cache synchronization finishes.
        unsafe {
            synchronize_code(physical);
        }
    }
    /// # Safety
    /// Same inactive-tree custody; caller exclusively owns the reserved device aperture.
    pub unsafe fn map_framebuffer(
        &mut self,
        _fb: cathedral_kernel_contracts::framebuffer::Framebuffer,
        _offset: u64,
        _bytes: u64,
        _frames: &mut impl FrameSource,
    ) -> Result<(), MemoryError> {
        Err(MemoryError::UnsupportedCpu)
    }
}
unsafe fn synchronize_code(physical: u64) {
    let ctr: u64;
    // SAFETY: EL1, exclusively owned newly copied code; clean to PoU before invalidating instructions.
    unsafe {
        core::arch::asm!("mrs {}, ctr_el0",out(reg) ctr,options(nostack));
        let line = 4 << ((ctr >> 16) & 15);
        for address in (physical..physical + PAGE).step_by(line as usize) {
            core::arch::asm!("dc cvau, {}",in(reg) address,options(nostack));
        }
        core::arch::asm!("dsb ish", "ic iallu", "dsb ish", "isb", options(nostack));
    }
}
