//! Rust module plumbing. Start reading in `service.rs`.

#[cfg(feature = "interaction-delay")]
mod cancellation_probe;
mod children;
mod deferred;
#[cfg(feature = "interaction-delay")]
mod delay_probe;
mod host;
mod parent;
mod requests;
#[cfg(feature = "interaction-delay")]
mod storage_probe;

include!("service.rs");
