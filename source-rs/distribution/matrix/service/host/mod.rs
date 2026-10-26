//! Rust plumbing. Start at host.rs; service/service.rs drives the Omega session.
mod events;
mod initialization;
mod progress;
mod wait;
include!("host.rs");
