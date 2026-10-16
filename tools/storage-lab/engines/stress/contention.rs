//! Two preparing producers feed a bounded publisher queue. A concurrent reader
//! checks a pinned snapshot and current snapshots while bulk and small writes compete.
use super::{
    Result,
    publication::{self, Prepared},
};
use crate::{engines::backend::Backend, workloads::fixture};
use redb::ReadableDatabase;
use std::{
    path::Path,
    sync::{
        Barrier,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

struct Request {
    kind: &'static str,
    changes: Vec<Prepared>,
    started: Instant,
    queued: Instant,
    preparation_us: u128,
}
struct StopReader<'a>(&'a AtomicBool);
impl Drop for StopReader<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}

pub fn run(directory: &Path) -> Result<()> {
    let mut report = String::from(
        "bulk_batch,kind,objects,preparation_us,queue_us,writer_wait_us,writer_held_us,latency_us\n",
    );
    let mut summary = String::from(
        "bulk_batch,kind,requests,latency_p50_us,latency_p95_us,latency_p99_us,queue_p99_us,writer_held_max_us,reader_checks,pinned_extent_bytes,final_extent_bytes\n",
    );
    // Same 4 MiB of bulk records; the atomicity/transaction count intentionally differs.
    for bulk_batch in [64, 1] {
        let path = directory.join(format!("contention-{bulk_batch}.db"));
        let backend = Backend::open(&path)?;
        let db = publication::open(backend)?;
        publication::initialize(&db)?;
        assert!(
            publication::publish(&db, &[Prepared::new(1, 0, fixture(4096, 1))], false)?.accepted
        );
        let pinned = db.begin_read()?;
        let original = publication::read(&pinned, 1)?;
        let stop = AtomicBool::new(false);
        let start = Barrier::new(3);
        let (send, receive) = mpsc::sync_channel::<Request>(8);
        let mut samples = Vec::new();
        let checks = std::thread::scope(|scope| -> Result<usize> {
            // Drop the receiver on any early return so blocked producers wake.
            let receive = receive;
            let reader = scope.spawn(|| -> Result<usize> {
                let mut checks = 0;
                let mut last_revision = 1;
                loop {
                    assert_eq!(publication::read(&pinned, 1)?, original);
                    let (revision, bytes) = publication::read(&db.begin_read()?, 1)?;
                    assert!(revision >= last_revision);
                    assert_eq!(bytes, fixture(4096, revision));
                    last_revision = revision;
                    checks += 1;
                    if stop.load(Ordering::Acquire) {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
                Ok(checks)
            });
            let guard = StopReader(&stop);
            let small_send = send.clone();
            let small_start = &start;
            let small = scope.spawn(move || -> Result<()> {
                small_start.wait();
                for i in 0..64u64 {
                    let started = Instant::now();
                    let changes = vec![Prepared::new(1, i + 1, fixture(4096, i + 2))];
                    let preparation_us = started.elapsed().as_micros();
                    small_send.send(Request {
                        kind: "small",
                        changes,
                        started,
                        queued: Instant::now(),
                        preparation_us,
                    })?;
                }
                Ok(())
            });
            let bulk = scope.spawn(|| -> Result<()> {
                start.wait();
                for first in (0..64).step_by(bulk_batch) {
                    let started = Instant::now();
                    let changes = (first..first + bulk_batch)
                        .map(|i| {
                            Prepared::new(1000 + i as u64, 0, fixture(64 * 1024, i as u64 + 20))
                        })
                        .collect();
                    let preparation_us = started.elapsed().as_micros();
                    send.send(Request {
                        kind: "bulk",
                        changes,
                        started,
                        queued: Instant::now(),
                        preparation_us,
                    })?;
                }
                Ok(())
            });
            start.wait();
            // The number of messages is known; producer handles retain senders
            // until join, so do not wait for channel disconnection here.
            for _ in 0..64 + 64 / bulk_batch {
                let request = receive.recv_timeout(Duration::from_secs(30))?;
                let queue_us = request.queued.elapsed().as_micros();
                let publication = publication::publish(&db, &request.changes, false)?;
                assert!(publication.accepted);
                let latency = request.started.elapsed().as_micros();
                report.push_str(&format!(
                    "{bulk_batch},{},{},{},{queue_us},{},{},{latency}\n",
                    request.kind,
                    request.changes.len(),
                    request.preparation_us,
                    publication.writer_wait_us,
                    publication.writer_held_us
                ));
                samples.push((request.kind, latency, queue_us, publication.writer_held_us));
            }
            small.join().unwrap()?;
            bulk.join().unwrap()?;
            drop(guard);
            reader.join().unwrap()
        })?;
        assert_eq!(publication::read(&pinned, 1)?, original);
        let pinned_extent = std::fs::metadata(&path)?.len();
        drop(pinned);
        drop(db);
        let db = publication::open(Backend::open(&path)?)?;
        assert_eq!(
            publication::read(&db.begin_read()?, 1)?,
            (65, fixture(4096, 65))
        );
        for i in 0..64u64 {
            assert_eq!(
                publication::read(&db.begin_read()?, 1000 + i)?,
                (1, fixture(64 * 1024, i + 20))
            );
        }
        for kind in ["small", "bulk"] {
            let rows: Vec<_> = samples.iter().filter(|row| row.0 == kind).collect();
            let mut latency: Vec<_> = rows.iter().map(|row| row.1).collect();
            let mut queue: Vec<_> = rows.iter().map(|row| row.2).collect();
            latency.sort_unstable();
            queue.sort_unstable();
            let percentile = |v: &[u128], p: usize| v[(v.len() * p).div_ceil(100) - 1];
            summary.push_str(&format!(
                "{bulk_batch},{kind},{},{},{},{},{},{},{checks},{pinned_extent},{}\n",
                rows.len(),
                percentile(&latency, 50),
                percentile(&latency, 95),
                percentile(&latency, 99),
                percentile(&queue, 99),
                rows.iter().map(|row| row.3).max().unwrap(),
                std::fs::metadata(&path)?.len()
            ));
        }
    }
    std::fs::write(directory.join("contention.csv"), report)?;
    std::fs::write(directory.join("contention-summary.csv"), summary)?;
    println!("VERIFIED: bounded competing producers, pinned/current readers and reopened contents");
    Ok(())
}
