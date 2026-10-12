#![no_std]
#![deny(unsafe_code)]

//! Firmware-neutral kernel owners. Start at `user_tasks/user_tasks.rs` for the ordinary
//! prepare -> admit -> execute -> collect flow, or `kernel_tasks/kernel_tasks.rs` for trusted tasks.
//! Those coordinators use the memory, IPC and scheduling policies below; architecture
//! supplies CPU mechanics and boot supplies firmware custody.
extern crate alloc;
pub mod ipc;
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
pub mod kernel_tasks;
pub mod memory;
pub(crate) mod names;
pub mod scheduler;
#[allow(unsafe_code)]
pub mod user_tasks;
