//! Rust module plumbing. Start reading in `requests.rs`.

mod images;
mod owner;
mod presentation;
pub(in crate::service) mod source;
pub(super) use images::image;
mod surfaces;
pub(in crate::service) use surfaces::{flush, replay};

include!("requests.rs");
