#![no_std]
#![deny(unsafe_code)]

//! Firmware-neutral core policies, independently testable on the host.

extern crate alloc;

pub mod extent;
#[allow(unsafe_code)]
pub mod heap;
pub mod scheduler;
#[allow(unsafe_code)]
pub mod tasks;
