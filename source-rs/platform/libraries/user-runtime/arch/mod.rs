#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub(super) use x86_64::{call, call3, stop};
#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "aarch64")]
pub(super) use aarch64::{call, call3, stop};

pub(crate) mod shared;
