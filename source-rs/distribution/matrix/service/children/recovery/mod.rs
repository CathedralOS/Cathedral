//! Rust plumbing; start at recovery.rs.
mod completion;
mod fallback;
mod state;
use cathedral_stock_matrix::policy::recovery::Action;
pub(super) use state::{Fallback, Launcher};
include!("recovery.rs");
