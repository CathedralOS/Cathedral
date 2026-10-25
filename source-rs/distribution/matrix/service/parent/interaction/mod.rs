//! Rust plumbing. Read interaction.rs for dispatch and connection.rs for transport.
mod connection;
pub(super) use connection::{Connection, Lane};
include!("interaction.rs");
