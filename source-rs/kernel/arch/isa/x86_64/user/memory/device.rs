//! Device leaves borrow external backing; only their page tables are reclaimed.
use super::paging::{Tables, map};
use super::*;
use ::x86_64::{
    VirtAddr,
    structures::paging::{OffsetPageTable, PageTable, PageTableFlags as Flags},
};

use cathedral_kernel_contracts::framebuffer::{Framebuffer, USER_ADDRESS};

impl UserMappings {
    /// Map a page-aligned slice of the boot-approved aperture into an inactive task.
    /// # Safety
    /// Kernel CR3, IRQs off. Caller owns this inactive space and exclusive device
    /// access. Firmware has exited; the aperture is reserved from allocation.
    /// No other CPU/alias accesses it. On failure, release the entire space.
    pub unsafe fn map_framebuffer(
        &mut self,
        fb: Framebuffer,
        offset: u64,
        bytes: u64,
        frames: &mut impl FrameSource,
    ) -> Result<(), MemoryError> {
        if !fb.valid()
            || !offset.is_multiple_of(PAGE)
            || bytes == 0
            || !bytes.is_multiple_of(PAGE)
            || offset.checked_add(bytes).is_none_or(|end| end > fb.bytes)
        {
            return Err(MemoryError::InvalidFrame);
        }
        self.assert_inactive();
        if core::arch::x86_64::__cpuid(1).edx & (1 << 16) == 0 {
            return Err(MemoryError::UnsupportedCpu);
        }
        // Use PAT index 3 (PCD|PWT) only when firmware left it as UC. Do not
        // silently reinterpret arbitrary firmware PAT state or enable WC here.
        // SAFETY: Privileged boot CPU; x86-64 lab profile has the PAT MSR.
        let pat = unsafe { ::x86_64::registers::model_specific::Msr::new(0x277).read() };
        if (pat >> 24) & 0xff != 0 {
            return Err(MemoryError::UnsupportedCpu);
        }
        let mut source = Tables(frames);
        // SAFETY: Exclusively owned inactive hierarchy, table frames identity-mapped.
        let mut target =
            unsafe { OffsetPageTable::new(&mut *(self.root as *mut PageTable), VirtAddr::zero()) };
        let flags = Flags::PRESENT
            | Flags::USER_ACCESSIBLE
            | Flags::WRITABLE
            | Flags::NO_EXECUTE
            | Flags::NO_CACHE
            | Flags::WRITE_THROUGH;
        for offset in (offset..offset + bytes).step_by(PAGE as usize) {
            // SAFETY: Validated reserved aperture, disjoint bounded user range.
            // Device frames are deliberately absent from owned/backing ledgers:
            // they cannot be freed as RAM or used by normal syscall copy helpers.
            unsafe {
                map(
                    &mut target,
                    USER_ADDRESS + offset,
                    fb.physical + offset,
                    flags,
                    &mut source,
                )?;
            }
        }
        Ok(())
    }
}
