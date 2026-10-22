//! Host image integration, refusal/bounds checks, then exhaustive boundary cuts
//! for these three traces using the existing volatile/durable device model.
use super::*;
use crate::engines::stress::device::{Cut, Device};
use redb::ReadableTable;
use std::collections::BTreeMap;

const START: u64 = header::BLOCK;
const SIZE: u64 = 24 * 1024 * 1024;
const ID: [u8; 16] = [0x42; 16];
fn blank() -> Vec<u8> {
    let mut bytes = vec![0; (SIZE + 2 * START) as usize];
    bytes[..START as usize].fill(0x5a);
    bytes[(START + SIZE) as usize..].fill(0xa5);
    bytes
}
fn region(device: &Device) -> Region {
    Region::new(Arc::new(device.clone()), START, SIZE).unwrap()
}
fn guards(bytes: &[u8]) {
    assert_eq!(bytes.len(), (SIZE + 2 * START) as usize);
    assert!(bytes[..START as usize].iter().all(|b| *b == 0x5a));
    assert!(bytes[(START + SIZE) as usize..].iter().all(|b| *b == 0xa5));
}
fn write(db: &Database, bytes: &[u8]) -> Result<()> {
    let digest = blake3::hash(bytes);
    let mut tx = db.begin_write()?;
    tx.set_durability(redb::Durability::Immediate)?;
    {
        let mut table = tx.open_table(TABLE)?;
        table.insert(b"payload".as_slice(), bytes)?;
        table.insert(b"head".as_slice(), digest.as_bytes().as_slice())?;
        table.insert(b"event".as_slice(), digest.as_bytes().as_slice())?;
    }
    tx.commit()?;
    Ok(())
}
fn records(db: &Database) -> Result<BTreeMap<Vec<u8>, Vec<u8>>> {
    let tx = db.begin_read()?;
    let table = tx.open_table(TABLE)?;
    let records: BTreeMap<Vec<u8>, Vec<u8>> = table
        .iter()?
        .map(|e| {
            let (k, v) = e?;
            Ok((k.value().to_vec(), v.value().to_vec()))
        })
        .collect::<Result<_>>()?;
    if let Some(bytes) = records.get(b"payload".as_slice()) {
        assert_eq!(records[b"head".as_slice()], blake3::hash(bytes).as_bytes());
        assert_eq!(records[b"head".as_slice()], records[b"event".as_slice()]);
    }
    Ok(records)
}

pub fn run(directory: &Path) -> Result<()> {
    std::fs::create_dir(directory)?;
    native_image(directory)?;
    let device = Device::new(blank());
    format(region(&device), ID)?;
    let baseline = device.crash();
    refusals_and_bounds(&baseline)?;
    crash_cuts(directory, &baseline)?;
    println!("PASS: volume format/open, bounds/refusal, redundant headers and crash cuts");
    Ok(())
}

fn native_image(directory: &Path) -> Result<()> {
    let path = directory.join("volume.raw");
    let image = Arc::new(Image::create(&path, SIZE + 2 * START)?);
    assert!(
        Image::create(&path, SIZE).is_err(),
        "existing path was replaced"
    );
    assert!(Image::open(&path).is_err(), "second owner acquired image");
    image.write(0, &vec![0x5a; START as usize])?;
    image.write(START + SIZE, &vec![0xa5; START as usize])?;
    let assigned = Region::new(image.clone(), START, SIZE)?;
    format(assigned.clone(), ID)?;
    assert!(format(assigned.clone(), ID).is_err());
    let db = open(assigned)?;
    write(&db, &vec![9; 2 * 1024 * 1024])?;
    let expected = records(&db)?;
    drop(db);
    drop(image);
    let image = Arc::new(Image::open(&path)?);
    let db = open(Region::new(image.clone(), START, SIZE)?)?;
    assert_eq!(records(&db)?, expected);
    // Oversized engine growth must fail; the image and its neighbours stay fixed.
    assert!(write(&db, &vec![3; SIZE as usize]).is_err());
    drop(db);
    drop(image);
    let image = Arc::new(Image::open(&path)?);
    let db = open(Region::new(image.clone(), START, SIZE)?)?;
    assert_eq!(records(&db)?, expected);
    drop(db);
    drop(image);
    guards(&std::fs::read(path)?);
    println!("VERIFIED: locked native image, reopen, capacity refusal, neighbour preservation");
    Ok(())
}

fn refusals_and_bounds(baseline: &[u8]) -> Result<()> {
    for case in [
        "both_headers",
        "future_version",
        "conflicting_id",
        "geometry",
        "zero_length",
        "incomplete",
        "engine_magic",
    ] {
        let mut bytes = baseline.to_vec();
        match case {
            "both_headers" => {
                bytes[START as usize] ^= 1;
                bytes[(START + header::BLOCK) as usize] ^= 1;
            }
            "engine_magic" => bytes
                [(START + header::DATA_START) as usize..(START + header::DATA_START + 16) as usize]
                .fill(0),
            _ => {
                for slot in 0..2 {
                    let start = (START + slot * header::BLOCK) as usize;
                    let mut copy: [u8; header::BLOCK as usize] = bytes
                        [start..start + header::BLOCK as usize]
                        .try_into()
                        .unwrap();
                    match case {
                        "future_version" if slot == 0 => {
                            copy[16..24].copy_from_slice(&2u64.to_le_bytes())
                        }
                        "conflicting_id" if slot == 0 => copy[32] ^= 1,
                        "geometry" => copy[72..80].copy_from_slice(&u64::MAX.to_le_bytes()),
                        "zero_length" => copy[80..88].fill(0),
                        "incomplete" => copy[24..32].copy_from_slice(&1u64.to_le_bytes()),
                        _ => {}
                    }
                    header::seal(&mut copy);
                    bytes[start..start + copy.len()].copy_from_slice(&copy);
                }
            }
        }
        let device = Device::new(bytes.clone());
        assert!(open(region(&device)).is_err(), "accepted {case}");
        assert!(device.operations().is_empty(), "refusal mutated {case}");
        assert_eq!(device.crash(), bytes);
    }
    for slot in 0..2 {
        let mut bytes = baseline.to_vec();
        bytes[(START + slot * header::BLOCK) as usize] ^= 1;
        let device = Device::new(bytes);
        let db = open(region(&device))?;
        assert_eq!(records(&db)?.len(), 1);
    }
    let short = Device::new(baseline[..(START + SIZE - 1) as usize].to_vec());
    assert!(Region::new(Arc::new(short), START, SIZE).is_err());
    let device = Device::new(blank());
    assert!(Region::new(Arc::new(device.clone()), u64::MAX - 4095, SIZE).is_err());
    assert!(Region::new(Arc::new(device.clone()), 1, SIZE).is_err());
    let assigned = region(&device);
    assigned.write(SIZE - 1, &[1])?;
    assigned.sync()?;
    let before = device.crash();
    let device = Device::new(before.clone());
    assert!(format(region(&device), ID).is_err());
    assert!(device.operations().is_empty());
    assert_eq!(device.crash(), before);

    let device = Device::new(baseline.to_vec());
    let assigned = region(&device);
    let header = assigned.headers()?;
    let backend = Backend::new(assigned, header.clone());
    assert!(backend.write(header.length, &[1]).is_err());
    assert!(backend.read(u64::MAX, &mut [0]).is_err());
    assert!(backend.set_len(header.capacity() + 1).is_err());
    assert!(device.operations().is_empty());
    // Standalone adapter shrink/regrow changes only the envelope: the engine
    // bytes it hands back are whatever the engine last wrote there.
    backend.write(0, &[0xab; 4096])?;
    backend.set_len(1024)?;
    assert!(backend.read(1024, &mut [0]).is_err());
    backend.set_len(4096)?;
    let mut bytes = [0; 4096];
    backend.read(0, &mut bytes)?;
    assert!(bytes.iter().all(|b| *b == 0xab));
    guards(&device.crash());
    println!(
        "VERIFIED: corrupt/unknown/incomplete/truncated refusal, single-copy recovery, bounds and untouched regrowth"
    );
    Ok(())
}

fn crash_cuts(directory: &Path, baseline: &[u8]) -> Result<()> {
    let mut csv = String::from("phase,operation,kind,cut,outcome\n");
    let mut extents = String::from("phase,before_length,after_length,capacity,device_mutations\n");
    let mut count = 0;
    let expanded = Device::new(baseline.to_vec());
    let db = open(region(&expanded))?;
    write(&db, &vec![7; 2 * 1024 * 1024])?;
    let large = expanded.crash();
    drop(db);
    for phase in ["format", "publish", "shrink"] {
        let initial = if phase == "format" {
            blank()
        } else if phase == "shrink" {
            large.clone()
        } else {
            baseline.to_vec()
        };
        let device = Device::new(initial.clone());
        let mut db = if phase != "format" {
            Some(open(region(&device))?)
        } else {
            None
        };
        let old = db.as_ref().map(records).transpose()?;
        let before_length = if phase == "format" {
            0
        } else {
            region(&device).headers()?.length
        };
        device.reset();
        action(phase, &device, &mut db, &mut false)?;
        let operations = device.operations();
        let complete = device.crash();
        drop(db);
        let reference = Device::new(complete);
        let header = region(&reference).headers()?;
        if phase == "shrink" {
            assert!(
                header.length < before_length,
                "compaction did not exercise shrink"
            );
        }
        extents.push_str(&format!(
            "{phase},{before_length},{},{},{}\n",
            header.length,
            header.capacity(),
            operations.len()
        ));
        let new = records(&open(region(&reference))?)?;
        for (index, operation) in operations.iter().enumerate() {
            for cut in [Cut::Before, Cut::Partial, Cut::After] {
                let device = Device::new(initial.clone());
                let mut db = if phase != "format" {
                    Some(open(region(&device))?)
                } else {
                    None
                };
                device.arm(index + 1, cut);
                let mut acknowledged = false;
                let result = action(phase, &device, &mut db, &mut acknowledged);
                assert!(device.crashed());
                let bytes = device.crash();
                drop(db);
                guards(&bytes);
                let recovered = Device::new(bytes);
                let assigned = region(&recovered);
                let outcome = if phase == "format" && !assigned.headers().is_ok_and(|h| h.ready) {
                    assert!(result.is_err());
                    assert!(open(assigned).is_err());
                    "incomplete"
                } else {
                    let db = open(assigned).map_err(|e| {
                        format!("{phase} cut {} {operation:?} {cut:?}: {e}", index + 1)
                    })?;
                    let contents = records(&db)?;
                    assert!(
                        contents == new || old.as_ref() == Some(&contents),
                        "mixed state {phase} {} {cut:?}",
                        index + 1
                    );
                    if result.is_ok() || acknowledged {
                        assert_eq!(contents, new);
                    }
                    if contents == new { "new" } else { "old" }
                };
                csv.push_str(&format!(
                    "{phase},{},{},{cut:?},{outcome}\n",
                    index + 1,
                    operation.kind
                ));
                count += 1;
            }
        }
    }
    std::fs::write(directory.join("volume-cuts.csv"), csv)?;
    std::fs::write(directory.join("volume-layout.csv"), extents)?;
    println!(
        "VERIFIED: {count} volume format/publication/reclamation cuts, including durable logical-length transitions"
    );
    Ok(())
}

fn action(
    phase: &str,
    device: &Device,
    db: &mut Option<Database>,
    acknowledged: &mut bool,
) -> Result<()> {
    if phase == "format" {
        return format(region(device), ID);
    }
    let db = db.as_mut().unwrap();
    if phase == "publish" {
        write(db, &vec![7; 2 * 1024 * 1024])?;
    } else {
        write(db, &[8; 1024])?;
    }
    *acknowledged = true;
    if phase == "shrink" {
        db.compact()?;
    }
    Ok(())
}
