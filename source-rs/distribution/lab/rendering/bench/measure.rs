//! Allocation/zeroing, scalar rendering and copying are timed together; validation is outside timing.
use cathedral_rendering_lab::{
    pixels::{self, Source},
    scene,
};
use std::{hint::black_box, time::Instant};
#[derive(Clone, Copy)]
pub enum Strategy {
    Rows,
    CopyTree,
    FlatPacked,
    FlatPages,
}
impl Strategy {
    pub fn name(self) -> &'static str {
        match self {
            Self::Rows => "rows",
            Self::CopyTree => "copy-tree",
            Self::FlatPacked => "flat-packed",
            Self::FlatPages => "flat-pages",
        }
    }
}
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Cost {
    pub reserved: usize,
    pub padding: usize,
    pub zeroed: usize,
    pub rendered: usize,
    pub copied: usize,
    pub allocations: usize,
    pub nodes: usize,
}
impl Cost {
    fn allocate(&mut self, payload: usize, pages: bool) -> Vec<u8> {
        let length = if pages {
            payload.div_ceil(4096) * 4096
        } else {
            payload
        };
        self.reserved += length;
        self.padding += length - payload;
        self.zeroed += length;
        self.allocations += 1;
        vec![0; length]
    }
}
fn frame(width: usize, height: usize, depth: usize, strategy: Strategy) -> (Cost, Vec<Vec<u8>>) {
    let mut cost = Cost::default();
    let (nodes, count) = scene::fixture(width, height, depth).unwrap();
    let plan = scene::resolve(&nodes[..count], width, height).unwrap();
    cost.nodes = plan.visited;
    let bytes = width * height * 4;
    let pages = matches!(strategy, Strategy::FlatPages);
    let mut allocations = Vec::with_capacity(4 + depth);
    if matches!(strategy, Strategy::Rows) {
        let mut target = cost.allocate(bytes, false);
        cost.rendered = pixels::direct(&mut target, width, height, depth, None).unwrap();
        allocations.push(target);
    } else {
        for node in &nodes[depth..count] {
            let mut leaf = cost.allocate(node.rect.bytes().unwrap(), pages);
            cost.rendered += pixels::leaf(&mut leaf, node.rect, node.rect.width * 4).unwrap();
            allocations.push(leaf);
        }
        let sources = core::array::from_fn(|i| Source {
            bytes: &allocations[i],
            stride: nodes[depth + i].rect.width * 4,
        });
        let mut target = cost.allocate(bytes, pages);
        cost.copied += pixels::compose(&plan, &sources, &mut target).unwrap();
        allocations.push(target);
        if matches!(strategy, Strategy::CopyTree) {
            for _ in 1..depth {
                let mut parent = cost.allocate(bytes, false);
                parent.copy_from_slice(allocations.last().unwrap());
                cost.copied += bytes;
                allocations.push(parent);
            }
        }
    }
    (cost, allocations)
}
pub fn measure(width: usize, height: usize, depth: usize, strategy: Strategy) -> (Cost, u128) {
    let mut times = [0; 5];
    let mut result = Cost::default();
    for time in &mut times {
        let start = Instant::now();
        let (cost, allocations) = black_box(frame(
            black_box(width),
            black_box(height),
            black_box(depth),
            strategy,
        ));
        *time = start.elapsed().as_micros();
        assert!(pixels::verify(
            &allocations.last().unwrap()[..width * height * 4],
            width,
            height
        ));
        result = cost;
        // Keep every baseline ancestor alive through measurement; drops excluded.
        black_box(&allocations);
    }
    times.sort_unstable();
    (result, times[2])
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accounting_and_output_match_actual_work() {
        for depth in [1, 4, 8] {
            for strategy in [
                Strategy::Rows,
                Strategy::CopyTree,
                Strategy::FlatPacked,
                Strategy::FlatPages,
            ] {
                let (cost, allocations) = frame(65, 17, depth, strategy);
                let bytes = 65 * 17 * 4;
                assert!(pixels::verify(
                    &allocations.last().unwrap()[..bytes],
                    65,
                    17
                ));
                assert_eq!(cost.reserved, allocations.iter().map(Vec::len).sum());
                assert_eq!(cost.allocations, allocations.len());
                assert_eq!(cost.rendered, bytes);
                assert_eq!(
                    cost.copied,
                    match strategy {
                        Strategy::Rows => 0,
                        Strategy::CopyTree => bytes * depth,
                        _ => bytes,
                    }
                );
                assert_eq!(
                    cost.padding,
                    if matches!(strategy, Strategy::FlatPages) {
                        4 * 4096 + 8192 - 2 * bytes
                    } else {
                        0
                    }
                );
            }
        }
    }
}
