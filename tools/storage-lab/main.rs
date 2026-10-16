//! Host layout experiment: choose policies, execute byte-checked workloads, report costs.
//! Optional engine experiments persist records; neither mode defines a guest format or API.
mod chunk_tree;
mod chunks;
#[cfg(feature = "engines")]
mod engines;
mod workloads;

#[cfg(test)]
mod tests;

fn main() {
    #[cfg(feature = "engines")]
    if std::env::args().nth(1).as_deref() == Some("engines") {
        engines::run().expect("engine experiment failed");
        return;
    }
    println!(
        "layout,chunk_bytes,inline_limit,workload,logical_bytes,hashed_payload_bytes,hashed_manifest_bytes,new_payload_bytes,new_manifest_bytes,live_payload_bytes,live_manifest_bytes,live_chunks,live_versions,live_tree_nodes,read_metadata_bytes,read_metadata_records,read_payload_bytes,read_payload_records,elapsed_us"
    );
    for chunk_bytes in [4096, 16384, 65536] {
        for inline_limit in [0, 128] {
            for layout in [
                chunks::Layout::Flat,
                chunks::Layout::Tree { fanout: 32 },
                chunks::Layout::Tree { fanout: 128 },
            ] {
                for row in workloads::measure(chunks::Policy {
                    chunk_bytes,
                    inline_limit,
                    layout,
                }) {
                    println!("{layout},{chunk_bytes},{inline_limit},{row}");
                }
            }
        }
    }
}
