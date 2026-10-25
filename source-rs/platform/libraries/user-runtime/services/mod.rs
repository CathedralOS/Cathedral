//! Rust plumbing. Start at services.rs, then transport.rs or server-side offer.rs.
mod offer;
mod transport;
pub use offer::Offer;
include!("services.rs");
