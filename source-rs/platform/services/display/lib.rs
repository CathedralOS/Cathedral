#![no_std]
#![forbid(unsafe_code)]
//! Retained scene policy: validate whole snapshots, then resolve local drawing.
pub mod pointer;
pub mod scene;

pub mod focus;
pub mod hosting;
