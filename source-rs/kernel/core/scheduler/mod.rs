//! Rust module plumbing. Start reading in `scheduler.rs`.

mod budget;
pub(crate) mod deadlines;
mod shares;
pub use budget::{CpuReport, PERIOD_US as CPU_PERIOD_US};
#[cfg(test)]
mod tests;

include!("scheduler.rs");
