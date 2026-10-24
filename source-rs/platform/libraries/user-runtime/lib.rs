#![no_std]

//! Minimal Rust-lab userspace support. No kernel imports or OS policy; an explicit heap is opt-in.
//! Start at entry.rs for _start -> supplied environment -> application main -> exit.
//! task/ipc/memory expose explicit capabilities; server/server.rs owns service event ordering.
//! Protocol clients name their receiving service; arch/ alone performs the trap.

#[cfg(not(all(
    any(target_arch = "x86_64", target_arch = "aarch64"),
    target_os = "none"
)))]
compile_error!("Build the user runtime for x86_64-unknown-none or aarch64-unknown-none");

mod arch;
mod diagnostics;
mod entry;
pub use diagnostics::{report_panic, write};
pub use task::{exit, yield_now};
pub mod capture;
pub mod device;
pub mod disk;
pub mod display;
pub mod event;
#[cfg(feature = "heap")]
pub mod heap;
pub mod ipc;
pub mod keyboard;
pub mod link;
pub mod matrix;
pub mod memory;
pub mod resources;
pub mod server;
pub mod services;
pub mod storage;
pub mod task;
pub mod time;
use cathedral_kernel_contracts::user as abi;

#[derive(Debug, PartialEq, Eq)]
pub struct Error(pub i64);
