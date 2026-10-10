#![no_std]
#![forbid(unsafe_code)]

//! Experimental, firmware-neutral Rust contracts. Not a frozen wire ABI.

pub mod boot;
pub mod display;
pub mod input;
pub mod memory;
pub mod user;

pub mod block;
pub mod storage;
