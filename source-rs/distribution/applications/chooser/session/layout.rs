//! Rust painting adapter for the geometry used by Omega input selection.
use super::{ENTRIES, policy, wire};
pub const WIDTH: usize = 128;
pub const HEIGHT: usize = 96;
pub const ICON: usize = 48;
pub fn tasks_left(count: usize) -> u64 {
    policy::geometry(false, count, 0, 0, 0).tasks_left
}
pub fn dimensions(picker: bool) -> (usize, usize) {
    policy::geometry(picker, 0, 0, 0, 0).dimensions
}
pub fn entries(count: usize, picker: bool) -> ([usize; ENTRIES], usize) {
    let geometry = policy::geometry(picker, count, 0, 0, 0);
    (geometry.items, geometry.count)
}
pub fn origin(view: wire::Packet, realm_count: usize, picker: bool, tile: usize) -> (u64, u64) {
    policy::geometry(picker, realm_count, view[2], view[3], tile).origin
}
