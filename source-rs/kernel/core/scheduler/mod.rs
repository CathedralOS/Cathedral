//! Rust module plumbing. Start reading in `scheduler.rs`.

mod budget;
mod shares;
pub use budget::{CpuReport, PERIOD_US as CPU_PERIOD_US};
#[cfg(test)]
mod tests;

include!("scheduler.rs");
