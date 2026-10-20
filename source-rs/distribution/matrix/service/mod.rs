//! Rust module plumbing. Start reading in `service.rs`.

mod children;
mod deferred;
#[cfg(feature = "interaction-delay")]
mod delay_probe;
mod parent;
mod requests;

include!("service.rs");
