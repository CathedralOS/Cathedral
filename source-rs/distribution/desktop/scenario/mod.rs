//! Rust module plumbing. Start reading in `scenario.rs`.

mod checks;
mod input_capacity;
mod loss;
mod producers;
mod recovery;
mod registration;

include!("scenario.rs");
