//! One bounded row of users; a vertical strip of apps. Hit testing shares geometry.
use super::{ENTRIES, NEW, USER_ENTRIES, wire};
pub const WIDTH: usize = 128;
pub const HEIGHT: usize = 96;
pub const GAP: u64 = 24;
pub fn entries(count: usize, picker: bool) -> ([usize; ENTRIES], usize) {
    let mut items = [0; ENTRIES];
    let mut n = 0;
    for index in 1..count {
        items[n] = index;
        n += 1;
    }
    if picker && count < USER_ENTRIES {
        items[n] = NEW;
        n += 1;
    }
    (items, n)
}
pub fn origin(view: wire::Packet, count: usize, picker: bool, tile: usize) -> (u64, u64) {
    if picker {
        let total = count as u64 * (WIDTH as u64 + GAP) - GAP;
        (
            view[2].saturating_sub(total) / 2 + tile as u64 * (WIDTH as u64 + GAP),
            view[3].saturating_sub(HEIGHT as u64) / 2,
        )
    } else {
        (
            view[2].saturating_sub(WIDTH as u64) / 2,
            24 + tile as u64 * (HEIGHT as u64 + GAP),
        )
    }
}
pub fn hit(view: wire::Packet, count: usize, picker: bool, x: u64, y: u64) -> Option<usize> {
    let (items, n) = entries(count, picker);
    (0..n).find_map(|i| {
        let (left, top) = origin(view, n, picker, i);
        ((left..left + WIDTH as u64).contains(&x) && (top..top + HEIGHT as u64).contains(&y))
            .then_some(items[i])
    })
}
pub fn step(selected: usize, count: usize, picker: bool, forward: bool) -> usize {
    let (items, n) = entries(count, picker);
    if n == 0 {
        return 0;
    }
    let at = items[..n].iter().position(|&i| i == selected).unwrap_or(0);
    items[(at + if forward { 1 } else { n - 1 }) % n]
}
