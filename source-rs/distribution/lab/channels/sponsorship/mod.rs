//! Rust module plumbing. Start reading in `sponsorship.rs`.

mod worker;
pub use worker::run as worker;

include!("sponsorship.rs");
