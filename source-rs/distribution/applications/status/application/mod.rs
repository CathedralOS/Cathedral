//! Rust module plumbing. Start reading in `application.rs`.

mod connection;
mod input;
#[cfg(feature = "recovery-lab")]
mod lab;
mod state;
mod storage;
mod view;

include!("application.rs");
