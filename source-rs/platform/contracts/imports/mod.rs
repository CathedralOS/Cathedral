//! Platform import declarations. Start in imports.rs; image.rs locates the record.
mod image;
#[cfg(test)]
mod tests;
pub use image::{from_elf, record_from_elf};
include!("imports.rs");
