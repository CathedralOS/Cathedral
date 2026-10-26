//! Rust module plumbing. Start at configuration.rs.
mod background;
mod layout;
mod objects;
mod workspace;
pub(super) use objects::read;
include!("configuration.rs");
