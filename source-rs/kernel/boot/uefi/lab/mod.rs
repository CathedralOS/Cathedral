//! Rust module plumbing. Start reading in `lab.rs`.

#[cfg(all(feature = "bundled-user", target_arch = "x86_64"))]
mod applications;
#[cfg(all(feature = "bundled-user", target_arch = "x86_64"))]
mod display;
#[cfg(all(feature = "bundled-user", target_arch = "x86_64"))]
mod ipc;
#[cfg(target_arch = "x86_64")]
mod kernel_tasks;
#[cfg(all(feature = "bundled-user", target_arch = "x86_64"))]
mod memory;
#[cfg(all(feature = "bundled-user", target_arch = "x86_64"))]
mod rendering;
#[cfg(all(feature = "bundled-user", target_arch = "x86_64"))]
mod supervision;
#[cfg(target_arch = "x86_64")]
mod user_tasks;

#[cfg(target_arch = "x86_64")]
include!("lab.rs");
#[cfg(target_arch = "aarch64")]
mod arm64;
#[cfg(target_arch = "aarch64")]
pub use arm64::run;
