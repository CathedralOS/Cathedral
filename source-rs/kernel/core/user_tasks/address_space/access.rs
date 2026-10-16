//! Checked user-buffer access through owned physical backing; never dereference a user VA.
use super::{Backing, PAGE, UserSpace};

impl UserSpace {
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
            ..
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
            .find(|page| {
                page.writable
                    && address >= page.address
                    && address < page.address + PAGE
                    && end <= page.address + PAGE
            })
            .map(|page| page.physical + address - page.address)
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
}
