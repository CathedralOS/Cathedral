// The validated exports borrow opaque state; no Rust layout mirrors Omega data.
use crate::Error;
use core::marker::PhantomData;
#[cfg(target_os = "none")]
core::arch::global_asm!(include_str!(env!("OMEGA_UI_ASSEMBLY")));
pub type Query = unsafe extern "sysv64" fn(u64) -> u64;
pub type Apply = unsafe extern "sysv64" fn(*mut u8, *mut i32) -> u64;
#[derive(Clone, Copy)]
pub struct Api<'code> {
    pub(crate) query: Query,
    pub(crate) apply: Apply,
    code: PhantomData<&'code [u8]>,
}
impl<'code> Api<'code> {
    /// # Safety
    /// Both exports must implement ABI 2 and retain validated executable code
    /// and relative code/data positions for the entire `'code` lifetime.
    pub unsafe fn from_entries(query: Query, apply: Apply) -> Self {
        Self {
            query,
            apply,
            code: PhantomData,
        }
    }
    pub fn layout(self) -> Result<(usize, usize), Error> {
        // SAFETY: The constructor requires the scalar query calling contract.
        let (bytes, alignment, version) =
            unsafe { ((self.query)(0), (self.query)(1), (self.query)(2)) };
        if version != 2
            || bytes == 0
            || bytes > 256 * 1024
            || alignment == 0
            || !alignment.is_power_of_two()
            || alignment > 4096
        {
            return Err(Error::Layout);
        }
        Ok((bytes as usize, alignment as usize))
    }
}
#[cfg(target_os = "none")]
pub fn native_api() -> Api<'static> {
    unsafe extern "sysv64" {
        fn omega_ui_layout(kind: u64) -> u64;
        fn omega_ui_frame(state: *mut u8, packet: *mut i32) -> u64;
    }
    // SAFETY: Packaging validates these pure exports and links their code/data
    // into the ordinary task image, whose executable mapping lasts for the task.
    unsafe { Api::from_entries(omega_ui_layout, omega_ui_frame) }
}
