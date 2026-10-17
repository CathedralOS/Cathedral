#![no_std]
#![forbid(unsafe_code)]
//! Platform service protocols, independent of their providers and distribution.
//! display owns drawing/compositor protocols; input owns key transport; matrix
//! owns recursive local-world messages; storage owns private catalog messages.
//! Experimental contracts, not a frozen ABI.
pub mod database;
pub mod display;
pub mod input;
pub mod matrix;
pub mod storage;
