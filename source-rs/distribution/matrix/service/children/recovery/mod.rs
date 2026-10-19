//! Rust plumbing; start at recovery.rs.
mod fallback;
mod state;
use state::Phase;
pub(super) use state::{Fallback, Launcher};
include!("recovery.rs");
