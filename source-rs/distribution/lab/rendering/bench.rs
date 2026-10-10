//! Native scalar comparison. Run with --features host --bin cathedral-rendering-bench.
#[path = "bench/measure.rs"]
mod measure;
use measure::{Strategy, measure};
fn main() {
    println!(
        "width,height,depth,strategy,reserved_bytes,padding_bytes,zeroed_bytes,rendered_bytes,copied_bytes,allocations,metadata_nodes,median_us"
    );
    for (width, height) in [(64, 16), (1920, 1080), (1919, 1079)] {
        for depth in [1, 4, 8] {
            for strategy in [
                Strategy::Rows,
                Strategy::CopyTree,
                Strategy::FlatPacked,
                Strategy::FlatPages,
            ] {
                let (cost, time) = measure(width, height, depth, strategy);
                println!(
                    "{width},{height},{depth},{},{},{},{},{},{},{},{},{time}",
                    strategy.name(),
                    cost.reserved,
                    cost.padding,
                    cost.zeroed,
                    cost.rendered,
                    cost.copied,
                    cost.allocations,
                    cost.nodes
                );
            }
        }
    }
}
