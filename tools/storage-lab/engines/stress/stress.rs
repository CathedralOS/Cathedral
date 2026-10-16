// Stress orchestration: conflict checks, competing requests, crash cuts, then
// pinned-reader space pressure. Each experiment has its own fresh storage.
use super::Result;
use std::path::Path;

pub fn run(directory: &Path) -> Result<()> {
    std::fs::create_dir(directory)?;
    publication::check_conflict()?;
    contention::run(directory)?;
    faults::run(directory)?;
    reclamation::run(directory)?;
    println!("PASS: conflict, contention, crash cuts and reclamation");
    Ok(())
}
