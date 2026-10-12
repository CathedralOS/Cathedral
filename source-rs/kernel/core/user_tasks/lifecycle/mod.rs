//! Rust module plumbing. Start reading in `lifecycle.rs`.

mod delegation;
mod grants;
mod links;
#[deny(unsafe_code)]
pub(crate) mod supervision;

include!("lifecycle.rs");
