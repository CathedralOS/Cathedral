//! Rust module plumbing. Start reading in `lifecycle.rs`.

mod delegation;
mod grants;
mod links;
mod reclamation;
mod spawn;
pub(super) use spawn::spawn;
#[deny(unsafe_code)]
pub(crate) mod supervision;

include!("lifecycle.rs");
