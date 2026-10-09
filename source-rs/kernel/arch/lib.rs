#![no_std]

//! Compile-time CPU selection. Firmware handling belongs to the `uefi` crate;
//! memory allocation policy belongs to core. No core/driver dependencies here.

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;
#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use crate::x86_64::*;

#[cfg(not(target_arch = "x86_64"))]
compile_error!("Only the x86-64 backend is implemented; 32-bit x86 is not a boot target yet");
