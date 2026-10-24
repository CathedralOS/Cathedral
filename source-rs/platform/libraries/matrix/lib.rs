#![no_std]
#![forbid(unsafe_code)]
//! A local world: receive a parent grant, place children, route attenuated input.
pub mod funding;
pub mod reception;
pub mod world;
#[cfg(test)]
extern crate std;
