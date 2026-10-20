//! Centered login tiles and a compact horizontal taskbar share their hit geometry.
use super::{ENTRIES, NEW, USER_ENTRIES, wire};
pub const WIDTH: usize = 128;
pub const HEIGHT: usize = 96;
pub const GAP: u64 = 24;
pub const ICON: usize = 48;
pub fn dimensions(picker: bool) -> (usize, usize) {
    if picker {
        (WIDTH, HEIGHT)
    } else {
        (ICON, ICON)
    }
}
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
            8 + tile as u64 * 56,
            view[3].saturating_sub(ICON as u64) / 2,
        )
    }
}
pub fn hit(view: wire::Packet, count: usize, picker: bool, x: u64, y: u64) -> Option<usize> {
    let (items, n) = entries(count, picker);
    let (width, height) = dimensions(picker);
    (0..n).find_map(|i| {
        let (left, top) = origin(view, n, picker, i);
        ((left..left + width as u64).contains(&x) && (top..top + height as u64).contains(&y))
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
