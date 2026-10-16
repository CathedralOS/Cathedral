// Host engine comparison: materialize traces, commit, inspect file extents,
// checkpoint and reopen. Recovery subprocesses enter through the same adapters.
use crate::chunks::Layout;
use adapters::{Engine, file_bytes};
use std::{path::Path, time::Instant};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(2).collect();
    match args.first().map(String::as_str) {
        Some("versions") if args.len() == 1 => {
            println!("SQLite {}", rusqlite::version());
            Ok(())
        }
        Some("measure") if args.len() == 2 => measure(Path::new(&args[1])),
        Some("stress") if args.len() == 2 => stress::run(Path::new(&args[1])),
        Some("recovery") if args.len() == 5 => {
            recovery::run(&args[1], &args[2], Path::new(&args[3]), &args[4])
        }
        _ => Err(
            "usage: engines versions | engines measure NEW_DIRECTORY | engines stress NEW_DIRECTORY | engines recovery MODE ENGINE FILE PHASE".into(),
        ),
    }
}

fn measure(directory: &Path) -> Result<()> {
    std::fs::create_dir(directory)?; // Refuse overwrite of any previous experiment.
    println!(
        "engine,layout,chunk_bytes,workload,transactions,submitted_bytes,before_file_bytes,before_sidecar_bytes,after_file_bytes,after_sidecar_bytes,peak_observed_bytes,checkpoint_file_bytes,checkpoint_sidecar_bytes,commit_us,scan_us,checkpoint_us,reopen_us,records,backend_write_bytes,backend_writes,backend_syncs"
    );
    for chunk_bytes in [4096, 16384] {
        for layout in [Layout::Flat, Layout::Tree { fanout: 32 }] {
            for scenario in ["tiny", "single", "separate", "batch"] {
                let trace = trace::make(layout, chunk_bytes, scenario);
                for kind in ["redb", "sqlite_delete", "sqlite_wal"] {
                    let path =
                        directory.join(format!("{kind}-{layout}-{chunk_bytes}-{scenario}.db"));
                    let mut engine = Engine::open(kind, &path)?;
                    engine.apply(&trace.initial.clone().into_iter().collect(), || {})?;
                    engine.checkpoint()?;
                    let (before, before_sidecar) = file_bytes(&path)?;
                    engine.reset_counts();
                    let mut peak = before + before_sidecar;
                    let mut elapsed = 0;
                    let mut submitted = 0;
                    for batch in &trace.batches {
                        submitted += batch.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>();
                        let start = Instant::now();
                        engine.apply(batch, || {})?;
                        elapsed += start.elapsed().as_micros();
                        let (main, sidecar) = file_bytes(&path)?;
                        peak = peak.max(main + sidecar);
                    }
                    let counts = engine.counts();
                    let (after, sidecar) = file_bytes(&path)?;
                    let start = Instant::now();
                    let events = engine.scan(b"event/", b"event0")?;
                    let scan_us = start.elapsed().as_micros();
                    let expected_events = trace
                        .expected
                        .range(b"event/".to_vec()..b"event0".to_vec())
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    assert_eq!(events, expected_events);
                    assert_eq!(engine.scan(b"", &[255])?, trace.expected);
                    let start = Instant::now();
                    engine.checkpoint()?;
                    let checkpoint_us = start.elapsed().as_micros();
                    let (checkpoint, checkpoint_sidecar) = file_bytes(&path)?;
                    drop(engine);
                    let start = Instant::now();
                    let engine = Engine::open(kind, &path)?;
                    let reopen_us = start.elapsed().as_micros();
                    assert_eq!(engine.scan(b"", &[255])?, trace.expected);
                    let io = counts
                        .map(|c| format!("{},{},{}", c.write_bytes, c.writes, c.syncs))
                        .unwrap_or_else(|| "NA,NA,NA".to_string());
                    println!(
                        "{kind},{layout},{chunk_bytes},{scenario},{},{submitted},{before},{before_sidecar},{after},{sidecar},{peak},{checkpoint},{checkpoint_sidecar},{elapsed},{scan_us},{checkpoint_us},{reopen_us},{},{io}",
                        trace.batches.len(),
                        trace.expected.len()
                    );
                }
            }
        }
    }
    Ok(())
}
