//! Rust module plumbing. Start reading in `deferred.rs`.

mod pending;
#[cfg(test)]
mod tests;
pub(in crate::user_tasks) use pending::{Pending, Work};

include!("deferred.rs");
