//! Rust module plumbing. Start reading in `service.rs`.

mod children;
mod deferred;
mod parent;
mod requests;

include!("service.rs");
