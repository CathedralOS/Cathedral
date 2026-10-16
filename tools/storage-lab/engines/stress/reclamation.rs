//! A pinned reader retains deleted engine pages. Release it, cycle replacements
//! under the SAME capacity cap, then check deliberate capacity refusal and recovery.
use super::{
    Result,
    device::Device,
    publication::{self, Prepared},
};
use crate::workloads::fixture;
use redb::ReadableDatabase;
use std::path::Path;

pub fn run(directory: &Path) -> Result<()> {
    let mut report = String::from("phase,revision,extent_bytes,capacity_bytes\n");
    // Control: same replacements without a long-lived reader.
    let control = Device::new(Vec::new());
    let control_db = publication::open(control.clone())?;
    publication::initialize(&control_db)?;
    for revision in 1..=17 {
        assert!(
            publication::publish(
                &control_db,
                &[Prepared::new(
                    1,
                    revision - 1,
                    fixture(256 * 1024, revision)
                )],
                true
            )?
            .accepted
        );
        report.push_str(&format!("unpinned,{revision},{},0\n", control.extent()));
    }
    drop(control_db);
    let device = Device::new(Vec::new());
    let db = publication::open(device.clone())?;
    publication::initialize(&db)?;
    let payload = 256 * 1024;
    assert!(publication::publish(&db, &[Prepared::new(1, 0, fixture(payload, 1))], true)?.accepted);
    let pinned = db.begin_read()?;
    let original = publication::read(&pinned, 1)?;
    for revision in 2..=17u64 {
        assert!(
            publication::publish(
                &db,
                &[Prepared::new(1, revision - 1, fixture(payload, revision))],
                true
            )?
            .accepted
        );
        assert_eq!(publication::read(&pinned, 1)?, original);
        report.push_str(&format!("pinned,{revision},{},0\n", device.extent()));
    }
    let cap = device.extent();
    device.limit(cap);
    drop(pinned);
    // Enough replacements to exceed the entire extent several times unless old
    // backing is reused. No compact(), larger cap, or reopen during this phase.
    let cycles = 4 * cap / payload + 32;
    for revision in 18..18 + cycles as u64 {
        assert!(
            publication::publish(
                &db,
                &[Prepared::new(1, revision - 1, fixture(payload, revision))],
                true
            )?
            .accepted
        );
        assert!(device.extent() <= cap);
        assert_eq!(
            publication::read(&db.begin_read()?, 1)?,
            (revision, fixture(payload, revision))
        );
        report.push_str(&format!("released,{revision},{},{cap}\n", device.extent()));
    }
    let last = 17 + cycles as u64;
    let expected = publication::records(&db)?;
    let oversized = Prepared::new(1, last, fixture(cap + payload, last + 1));
    assert!(publication::publish(&db, &[oversized], true).is_err());
    assert!(
        device.denied() > 0,
        "failure must come from the capacity cap"
    );
    let image = device.crash();
    drop(db);
    let device = Device::new(image);
    device.limit(cap); // The limit survives simulated reboot; never lift it.
    let db = publication::open(device.clone())?;
    assert_eq!(publication::records(&db)?, expected);
    assert!(
        publication::publish(
            &db,
            &[Prepared::new(1, last, fixture(payload, last + 1))],
            true
        )?
        .accepted
    );
    assert_eq!(
        publication::read(&db.begin_read()?, 1)?,
        (last + 1, fixture(payload, last + 1))
    );
    assert!(device.extent() <= cap);
    report.push_str(&format!(
        "recovered,{},{},{cap}\n",
        last + 1,
        device.extent()
    ));
    std::fs::write(directory.join("reclamation.csv"), report)?;
    println!(
        "VERIFIED: {cycles} replacements reuse backing below {cap} bytes; capacity refusal and recovery retain cap"
    );
    Ok(())
}
