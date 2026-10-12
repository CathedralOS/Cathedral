//! Build private user mappings beneath a disjoint root branch, tracking every owned frame.
use super::*;
use tables::{NORMAL, NX, RO, USER};
impl UserSpace {
    pub(super) fn frame(&mut self, frames: &mut impl StackFrames) -> Result<u64, MemoryError> {
        if self.len == MAX_FRAMES {
            return Err(MemoryError::OutOfFrames);
        }
        let address = frames.allocate().ok_or(MemoryError::OutOfFrames)?;
        self.owned[self.len] = address;
        self.len += 1;
        if !address.is_multiple_of(PAGE) || !(0x4000_0000..0x8000_0000).contains(&address) {
            return Err(MemoryError::InvalidFrame);
        }
        // SAFETY: The frame source contract supplies exclusively owned identity RAM.
        unsafe {
            core::ptr::write_bytes(address as *mut u8, 0, PAGE as usize);
        }
        Ok(address)
    }
    pub(super) unsafe fn build(
        &mut self,
        layout: &BootLayout,
        _start: u64,
        _end: u64,
        segments: &[UserSegment<'_>],
        frames: &mut impl StackFrames,
    ) -> Result<(), MemoryError> {
        self.root = self.frame(frames)?;
        // SAFETY: Exclusive new root borrows only the live supervisor branch, never its ownership.
        unsafe {
            (self.root as *mut u64).write((layout.root_address() as *const u64).read());
        }
        let stack = UserSegment {
            address: USER_STACK,
            bytes: &[],
            memory_size: 4 * PAGE,
            writable: true,
            executable: false,
        };
        for segment in segments.iter().chain(core::iter::once(&stack)) {
            let flags = NORMAL
                | USER
                | (1 << 53)
                | if segment.writable { 0 } else { RO }
                | if segment.executable { 0 } else { NX };
            for offset in (0..segment.memory_size).step_by(PAGE as usize) {
                let physical = self.frame(frames)?;
                let address = segment.address + offset;
                // SAFETY: Inactive owned tables, disjoint validated user range, fresh physical backing.
                unsafe {
                    tables::map(self.root, address, physical, flags, &mut || {
                        self.frame(frames)
                    })?;
                    if (offset as usize) < segment.bytes.len() {
                        let bytes = &segment.bytes[offset as usize..];
                        core::ptr::copy_nonoverlapping(
                            bytes.as_ptr(),
                            physical as *mut u8,
                            bytes.len().min(PAGE as usize),
                        );
                    }
                    if segment.executable {
                        synchronize_code(physical);
                    }
                }
                self.backing[self.pages] = Backing { address, physical };
                if segment.writable {
                    self.writable |= 1 << self.pages;
                }
                self.pages += 1;
            }
        }
        Ok(())
    }
    /// # Safety
    /// Inactive exclusively owned space; device backing remains reserved by the caller.
    pub unsafe fn map_framebuffer(
        &mut self,
        _fb: cathedral_contracts::display::Framebuffer,
        _frames: &mut impl StackFrames,
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
