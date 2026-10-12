#![no_std]

//! Compile-time CPU selection and privileged board adapters. Firmware handling
//! belongs to the `uefi` crate; frame ownership and process layout belong to core.
//! Backends own translations and saved registers; board/ owns concrete controllers.

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;
#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use crate::x86_64::*;

#[cfg(target_arch = "aarch64")]
mod aarch64;
mod events;
pub use events::{Fault, SwitchCause, UserFault};
pub mod board;
pub mod memory;
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;
pub use memory::{FrameSource, ImageRange, MemoryError, StackRange};
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
compile_error!("Only x86-64 and AArch64 backends are implemented");
