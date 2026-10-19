//! Rust plumbing. Start at services.rs; offer.rs owns server-side custody.
mod offer;
pub use offer::Offer;
include!("services.rs");
