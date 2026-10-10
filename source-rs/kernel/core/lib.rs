#![no_std]
#![deny(unsafe_code)]

//! Firmware-neutral core policies, independently testable on the host.
//! Execution entrances: [`kernel_tasks::run`] for trusted functions and
//! [`user_tasks::run_configured`] for isolated executables and boot-issued grants.

extern crate alloc;

pub mod byte_queue;
pub mod deadline;
pub mod extent;
#[allow(unsafe_code)]
pub mod heap;
pub mod ipc;
#[allow(unsafe_code)]
pub mod kernel_tasks;
pub mod link;
pub mod scheduler;
pub mod supervision;
#[allow(unsafe_code)]
pub mod user_tasks;
