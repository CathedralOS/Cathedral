//! Retrieve only this task's boot-installed display mapping; no mapping request API.
use crate::{Error, abi, arch};
use cathedral_contracts::display;

pub fn mapping() -> Result<[u64; 6], Error> {
    #[repr(align(64))]
    struct Info([u8; display::INFO_BYTES]);
    let mut info = Info([0; display::INFO_BYTES]);
    let result = arch::call(
        abi::DISPLAY_INFO,
        info.0.as_mut_ptr() as u64,
        display::INFO_BYTES as u64,
    );
    if (result as i64) < 0 {
        return Err(Error(result as i64));
    }
    display::decode(&info.0).ok_or(Error(abi::IO_ERROR as i64))
}
