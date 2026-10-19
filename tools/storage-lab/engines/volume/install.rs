//! Prepare the complete desktop upgrade in host scratch space. The saved disk
//! stays locked and unchanged until every migration and final capacity check pass.
use super::*;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

pub fn run(args: &[String]) -> Result<()> {
    if args.len() < 6 {
        return Err("install IMAGE OFFSET BYTES MIGRATION_OR_- APP_CLOSE COUNT (PARENT FILE)... [PATH FILE PREVIOUS_MANIFEST]...".into());
    }
    let count: usize = args[5].parse()?;
    let end = count
        .checked_mul(2)
        .and_then(|n| n.checked_add(6))
        .ok_or("program count overflow")?;
    if end > args.len() {
        return Err("missing program pairs".into());
    }
    let programs = seed::load(&args[6..end])?;
    let refresh = match args[4].as_str() {
        "true" => true,
        "false" => false,
        _ => return Err("invalid app-close flag".into()),
    };
    let legacy = if args[3] == "-" {
        None
    } else {
        Some(labels::legacy(Path::new(&args[3]))?)
    };
    if legacy.is_none() && end != args.len() {
        return Err("named additions require the stock migration".into());
    }
    compact::rewrite(
        Path::new(&args[0]),
        args[1].parse()?,
        args[2].parse()?,
        |records, header| {
            let directory = Scratch::new()?;
            let path = directory.0.join("volume.raw");
            let mut scratch = header.clone();
            scratch.region_bytes = header
                .region_bytes
                .checked_mul(4)
                .ok_or("scratch overflow")?;
            let data = compact::rebuild(&records, &scratch)?;
            let image = Image::create(&path, scratch.region_bytes)?;
            image.write(0, &data)?;
            image.sync_data()?;
            drop(image);
            seed::seed(&path, 0, scratch.region_bytes, &programs)?;
            if let Some(legacy) = legacy {
                labels::upgrade(&path, 0, scratch.region_bytes, &legacy, refresh)?;
                hierarchy::upgrade(&path, 0, scratch.region_bytes, &args[end..])?;
            }
            let db = open(Region::new(
                Arc::new(Image::open(&path)?),
                0,
                scratch.region_bytes,
            )?)?;
            compact::records(&db)
        },
    )
}

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        loop {
            let path = std::env::temp_dir().join(format!(
                "cathedral-install-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match std::fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
                Err(e) => return Err(e.into()),
            }
        }
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        // Only this invocation's exclusively created private scratch directory.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
