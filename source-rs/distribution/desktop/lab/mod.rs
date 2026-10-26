//! Rust module plumbing for the probe desktop. Start reading in `desktop.rs`.

mod commands;
#[cfg(feature = "database-probes")]
pub(crate) mod database_recovery;
mod funding;
mod probes;
mod providers;
mod recovery;
mod stalls;
mod stalls_peer;
pub(crate) use commands::command;

use crate::session::{BACKGROUND, KEYS, admission, control::RootControl, owner, persistence};
mod source;
use crate::*;
use cathedral_colors as app;
use cathedral_platform_contracts::input;

include!("desktop.rs");
