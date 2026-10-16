//! Retrieve only this task's boot-installed display mapping; no mapping request API.
use crate::{Error, abi, arch};
use cathedral_kernel_contracts::framebuffer;

pub fn mapping() -> Result<[u64; 6], Error> {
    #[repr(align(64))]
    struct Info([u8; framebuffer::INFO_BYTES]);
    let mut info = Info([0; framebuffer::INFO_BYTES]);
    let result = arch::call(
        abi::DISPLAY_INFO,
        info.0.as_mut_ptr() as u64,
        framebuffer::INFO_BYTES as u64,
    );
    if (result as i64) < 0 {
        return Err(Error(result as i64));
    }
    framebuffer::decode(&info.0).ok_or(Error(abi::IO_ERROR as i64))
}
