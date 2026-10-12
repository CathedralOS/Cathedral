//! Rust module plumbing. Start reading in `interrupts.rs`.

mod entries;
mod tables;
pub(super) use tables::user_selectors;

include!("interrupts.rs");
