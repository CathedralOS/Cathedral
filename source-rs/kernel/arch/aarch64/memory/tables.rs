//! Bounded 4-level translation construction. No policy or page ownership lives here.
use super::MemoryError;
use core::arch::asm;
pub const ADDRESS: u64 = 0x0000_ffff_ffff_f000;
pub const NORMAL: u64 = (1 << 10) | (3 << 8);
pub const NX: u64 = (1 << 53) | (1 << 54);
pub const USER: u64 = 1 << 6;
pub const RO: u64 = 1 << 7;
pub struct PageTables<'a, F> {
    next: &'a mut F,
}
impl<'a, F: FnMut() -> Option<u64>> PageTables<'a, F> {
    pub fn new(next: &'a mut F) -> Self {
        Self { next }
    }
    pub fn frame(&mut self) -> Result<u64, MemoryError> {
        let p = (self.next)().ok_or(MemoryError::OutOfFrames)?;
        if !p.is_multiple_of(4096) || !(0x4000_0000..0x8000_0000).contains(&p) {
            return Err(MemoryError::InvalidFrame);
        }
        // SAFETY: The enclosing construction contract guarantees fresh, writable identity RAM.
        unsafe {
            core::ptr::write_bytes(p as *mut u8, 0, 4096);
        }
        Ok(p)
    }
}
pub unsafe fn leaf(
    root: u64,
    va: u64,
    next: &mut impl FnMut() -> Result<u64, MemoryError>,
) -> Result<*mut u64, MemoryError> {
    let mut table = root;
    for shift in [39, 30, 21] {
        let entry = (table as *mut u64).wrapping_add(((va >> shift) & 511) as usize);
        // SAFETY: Caller owns the hierarchy; each child comes from its exclusive frame source.
        unsafe {
            let mut descriptor = entry.read();
            if descriptor & 1 == 0 {
                descriptor = next()? | 3;
                entry.write(descriptor);
            }
            if descriptor & 3 != 3 {
                return Err(MemoryError::OccupiedVirtualRange);
            }
            table = descriptor & ADDRESS;
        }
    }
    Ok((table as *mut u64).wrapping_add(((va >> 12) & 511) as usize))
}
pub unsafe fn map(
    root: u64,
    va: u64,
    pa: u64,
    flags: u64,
    next: &mut impl FnMut() -> Result<u64, MemoryError>,
) -> Result<(), MemoryError> {
    // SAFETY: Caller owns inactive tables and unique or explicitly borrowed backing.
    unsafe {
        let entry = leaf(root, va, next)?;
        if entry.read() & 1 != 0 {
            return Err(MemoryError::OccupiedVirtualRange);
        }
        entry.write(pa | flags | 3);
    }
    Ok(())
}
pub unsafe fn activate(root: u64) {
    // SAFETY: Caller supplies a live tree retaining this code/stack; no other CPU exists.
    unsafe {
        asm!("dsb ishst", "msr ttbr0_el1, {}", "isb", "tlbi vmalle1", "dsb ish", "isb", in(reg) root, options(nostack));
    }
}
pub fn active() -> u64 {
    let root;
    // SAFETY: Privileged read under the architecture's EL1 calling contract.
    unsafe {
        asm!("mrs {}, ttbr0_el1",out(reg) root,options(nostack));
    }
    root
}
