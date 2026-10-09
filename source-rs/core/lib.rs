#![no_std]
#![deny(unsafe_code)]

//! Firmware-neutral core policies, independently testable on the host.

pub mod extent;
#[allow(unsafe_code)]
pub mod heap;
