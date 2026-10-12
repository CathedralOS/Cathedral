#![no_std]

//! Compile-time CPU selection. Firmware handling belongs to the `uefi` crate;
//! memory allocation policy belongs to core. No core/driver dependencies here.

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;
#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use crate::x86_64::*;

#[cfg(target_arch = "aarch64")]
mod aarch64;
mod image;
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
compile_error!("Only x86-64 and AArch64 backends are implemented");
