#![no_std]
//! Native Omega app boundary. The app owns state and chooses each operation;
//! the guest host supplies only scoped service transport and buffer custody.
mod component;
#[cfg(target_os = "none")]
mod host;
#[cfg(any(target_os = "none", test))]
mod transfer;
pub use component::{Instance, Step};
#[cfg(target_os = "none")]
pub use host::run;
pub const STATE_WORDS: usize = 512;
pub const PACKET_WORDS: usize = 4096;
pub const PAYLOAD_WORD: usize = 16;
pub const PAYLOAD_BYTES: usize = (PACKET_WORDS - PAYLOAD_WORD) * 8;
