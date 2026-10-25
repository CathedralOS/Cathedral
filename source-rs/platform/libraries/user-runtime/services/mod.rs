//! Rust plumbing. Start at services.rs, then transport.rs or server-side offer.rs.
mod offer;
mod transport;
mod wake;
pub use offer::Offer;
pub use wake::{Wake, WakeSource};
include!("services.rs");
