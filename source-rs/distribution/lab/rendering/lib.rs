#![no_std]
#![forbid(unsafe_code)]
//! Experimental rendering strategies, not an adopted surface or admission contract.
//! scene resolves geometry; rows delegates exact Rust borrows; pixels executes a
//! checked flattened plan. Host costs and hardware probes use the same workload.
pub mod pixels;
pub mod rows;
pub mod scene;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Bounds,
    Geometry,
    Overlap,
    Incomplete,
}
#[cfg(test)]
extern crate std;
#[cfg(test)]
mod tests;
