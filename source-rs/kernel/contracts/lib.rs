#![no_std]
#![forbid(unsafe_code)]
//! Public kernel boundaries: syscall operations, page capabilities and framebuffer access.
//! Experimental lab ABI; no kernel implementation or platform service dependencies.
pub mod framebuffer;
pub mod lab;
pub mod memory;
pub mod user;
