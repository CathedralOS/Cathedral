//! Enumerate mutation boundaries from a successful publication, then replay each
//! with an isolated durable image. Recovery checks the entire record set.
use super::{
    Result,
    device::{Cut, Device},
    publication::{self, Prepared},
};
use redb::ReadableDatabase;
use std::path::Path;

pub fn run(directory: &Path) -> Result<()> {
    let mut report =
        String::from("retire,payload_bytes,operation,kind,offset,bytes,cut,recovered\n");
    let mut cases = 0;
    // Small replacement and allocation/cache-pressure publication take different paths.
    for size in [4096, 2 * 1024 * 1024] {
        for retire in [false, true] {
            let device = Device::new(Vec::new());
            let db = publication::open(device.clone())?;
            publication::initialize(&db)?;
            publication::publish(&db, &[Prepared::new(1, 0, vec![7; 4096])], false)?;
            let old = publication::records(&db)?;
            let baseline = device.crash();
            drop(db);

            let device = Device::new(baseline.clone());
            let db = publication::open(device.clone())?;
            device.reset();
            assert!(
                publication::publish(&db, &[Prepared::new(1, 1, vec![9; size])], retire)?.accepted
            );
            let operations = device.operations();
            let new = publication::records(&db)?;
            let completed = device.crash();
            drop(db);
            let reopened = publication::open(Device::new(completed))?;
            assert_eq!(
                publication::records(&reopened)?,
                new,
                "acknowledged commit lost"
            );
            drop(reopened);
            for (index, operation) in operations.iter().enumerate() {
                for cut in [Cut::Before, Cut::Partial, Cut::After] {
                    let device = Device::new(baseline.clone());
                    let db = publication::open(device.clone())?;
                    device.arm(index + 1, cut);
                    let outcome =
                        publication::publish(&db, &[Prepared::new(1, 1, vec![9; size])], retire);
                    assert!(device.crashed(), "fault point not reached");
                    let image = device.crash(); // Snapshot BEFORE destructors can touch storage.
                    drop(db);
                    let reopened = publication::open(Device::new(image)).map_err(|e| {
                    format!(
                        "reopen retire={retire} size={size} operation={} {operation:?} cut={cut:?}: {e}",
                        index + 1
                    )
                })?;
                    let recovered = publication::records(&reopened)?;
                    assert!(
                        recovered == old || recovered == new,
                        "mixed publication at {} {cut:?}",
                        index + 1
                    );
                    if outcome.is_ok() {
                        assert_eq!(recovered, new, "acknowledged commit lost");
                    }
                    let (revision, bytes) = publication::read(&reopened.begin_read()?, 1)?;
                    assert!(
                        revision == 1 && bytes == vec![7; 4096]
                            || revision == 2 && bytes == vec![9; size]
                    );
                    let state = if recovered == old { "old" } else { "new" };
                    report.push_str(&format!(
                        "{retire},{size},{},{},{},{},{cut:?},{state}\n",
                        index + 1,
                        operation.kind,
                        operation.offset,
                        operation.bytes
                    ));
                    cases += 1;
                }
            }
        }
    }
    std::fs::write(directory.join("faults.csv"), report)?;
    println!("VERIFIED: {cases} deterministic crash cuts, plus acknowledged commits");
    Ok(())
}
