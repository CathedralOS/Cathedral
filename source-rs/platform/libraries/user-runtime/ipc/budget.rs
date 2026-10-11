//! Only this process's persistent channel account and dynamic-handle charges.
use super::{Error, abi, arch, result};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget {
    pub channel_limit: u64,
    pub charged_channels: u64,
    pub handle_limit: u64,
    pub charged_handles: u64,
}
pub fn budget() -> Result<Budget, Error> {
    #[repr(align(32))]
    struct Record([u64; 4]);
    let mut record = Record([0; 4]);
    result(arch::call3(
        abi::IPC_BUDGET,
        record.0.as_mut_ptr() as u64,
        0,
        0,
    ))?;
    Ok(Budget {
        channel_limit: record.0[0],
        charged_channels: record.0[1],
        handle_limit: record.0[2],
        charged_handles: record.0[3],
    })
}
