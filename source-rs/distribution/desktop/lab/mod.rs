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

use crate::session::{BACKGROUND, KEYS, admission, app, owner, persistence};
use crate::*;
use cathedral_platform_contracts::input;
use cathedral_user_runtime::task::Launch;

include!("desktop.rs");
