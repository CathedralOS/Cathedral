#![no_std]
//! Colors' event loop, document and surfaces. The desktop lab supplies a probe
//! callback; the installed executable selects the ordinary app at compile time.
#[cfg(target_os = "none")]
mod session;
#[cfg(target_os = "none")]
pub use session::{LEFT, Probe, RIGHT, run, run_services};
