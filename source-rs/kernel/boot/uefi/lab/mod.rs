//! Rust module plumbing. Start reading in `lab.rs`.

#[cfg(feature = "bundled-user")]
mod applications;
#[cfg(feature = "bundled-user")]
mod display;
#[cfg(feature = "bundled-user")]
mod ipc;
mod kernel_tasks;
#[cfg(feature = "bundled-user")]
mod memory;
#[cfg(feature = "bundled-user")]
mod rendering;
#[cfg(feature = "bundled-user")]
mod supervision;
mod user_tasks;

include!("lab.rs");
