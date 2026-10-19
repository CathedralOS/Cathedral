//! Rust module plumbing. Start in `admission.rs`.
mod attempt;
mod display;
mod funding;
mod input;
mod labels;
#[cfg(feature = "bootstrap-probes")]
mod probes;
mod root;
mod state;
pub(crate) use state::Session;
include!("admission.rs");
