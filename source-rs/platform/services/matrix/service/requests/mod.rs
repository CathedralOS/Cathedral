//! Rust module plumbing. Start reading in `requests.rs`.

mod images;
mod owner;
pub(super) use images::image;
mod surfaces;
pub(super) use surfaces::flush;

include!("requests.rs");
