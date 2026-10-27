#[link(name = "kernel32")]
unsafe extern "system" {
    fn VirtualAlloc(address: *mut u8, bytes: usize, allocation: u32, protection: u32) -> *mut u8;
    fn VirtualProtect(address: *mut u8, bytes: usize, protection: u32, previous: *mut u32) -> i32;
    fn VirtualFree(address: *mut u8, bytes: usize, operation: u32) -> i32;
    fn FlushInstructionCache(process: isize, address: *const u8, bytes: usize) -> i32;
}
pub(super) struct Code {
    pub address: *mut u8,
}
impl Code {
    pub fn new(bytes: &[u8]) -> Self {
        // SAFETY: The test owns this fresh mapping, copies the checked compiled
        // component, then removes write access before invoking any entry.
        unsafe {
            let address = VirtualAlloc(core::ptr::null_mut(), bytes.len(), 0x3000, 4);
            assert!(!address.is_null());
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), address, bytes.len());
            let mut previous = 0;
            assert_ne!(VirtualProtect(address, bytes.len(), 0x20, &mut previous), 0);
            assert_ne!(FlushInstructionCache(-1, address, bytes.len()), 0);
            Self { address }
        }
    }
}
impl Drop for Code {
    fn drop(&mut self) {
        // SAFETY: Every API/instance using this test-owned mapping is gone.
        unsafe {
            assert_ne!(VirtualFree(self.address, 0, 0x8000), 0);
        }
    }
}
