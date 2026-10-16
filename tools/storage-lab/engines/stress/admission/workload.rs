//! Same small edits and explicitly separate bulk publications as the FIFO control.
//! Producers reserve declared lengths before allocating payloads; failures unwind leases.
use super::{Budget, Client, Denied, Prepared, Queue, Request, Result, Shutdown, publication};
use crate::workloads::fixture;
use redb::{Database, ReadableDatabase};
use std::{
    sync::Barrier,
    time::{Duration, Instant},
};

pub fn check_refusal(db: &Database, budget: &Budget) -> Result<()> {
    let before = publication::records(db)?;
    // The rejected 4 MiB atomic import is described only by lengths: no payload
    // is built and no smaller transactions are substituted by admission.
    assert_eq!(
        budget.reserve(Client::Bulk, &[64 * 1024; 64]).unwrap_err(),
        Denied::TooLarge
    );
    assert_eq!(
        budget
            .reserve(Client::Bulk, &[4 * 1024 * 1024])
            .unwrap_err(),
        Denied::TooLarge
    );
    assert_eq!(budget.snapshot().current.bytes, 0);
    assert_eq!(publication::records(db)?, before);
    Ok(())
}

struct Sample {
    client: Client,
    latency: u128,
    admission: u128,
    queue: u128,
    held: u128,
}

pub fn compete(
    trial: usize,
    db: &Database,
    budget: &Budget,
    queue: &Queue,
    report: &mut String,
    summary: &mut String,
) -> Result<()> {
    assert!(publication::publish(db, &[Prepared::new(1, 0, fixture(4096, 1))], false)?.accepted);
    let pinned = db.begin_read()?;
    let original = publication::read(&pinned, 1)?;
    let barrier = Barrier::new(3);
    let mut samples = Vec::new();
    std::thread::scope(|scope| -> Result<()> {
        // Shutdown wakes both reservation waiters and queue consumers on errors.
        // It also cancels queued work; a live active request keeps its reservation.
        let shutdown = Shutdown(budget, queue);
        let producers: Vec<_> = [Client::Interactive, Client::Bulk]
            .into_iter()
            .map(|client| {
                let barrier = &barrier;
                scope.spawn(move || -> Result<()> {
                    barrier.wait();
                    for i in 0..64u64 {
                        let started = Instant::now();
                        let (object, expected, bytes, seed) = match client {
                            Client::Interactive => (1, i + 1, 4096, i + 2),
                            Client::Bulk => (1000 + i, 0, 64 * 1024, i + 20),
                        };
                        let reservation = budget.wait(client, &[bytes], Duration::from_secs(30))?;
                        let reserved = Instant::now();
                        let changes = vec![Prepared::new(object, expected, fixture(bytes, seed))];
                        queue.push(Request::prepared(reservation, changes, started, reserved)?)?;
                    }
                    Ok(())
                })
            })
            .collect();
        barrier.wait();
        for _ in 0..128 {
            let request = queue.pop(Duration::from_secs(30))?;
            let queue_us = request.queued.elapsed().as_micros();
            let admission = request.reserved.duration_since(request.started).as_micros();
            let preparation = request.queued.duration_since(request.reserved).as_micros();
            let published = publication::publish(db, &request.changes, false)?;
            assert!(published.accepted);
            let latency = request.started.elapsed().as_micros();
            let client = request.reservation.client();
            report.push_str(&format!(
                "{trial},{},{},{},{admission},{preparation},{queue_us},{},{},{latency}\n",
                client.name(),
                request.changes.len(),
                request.reservation.bytes(),
                published.writer_wait_us,
                published.writer_held_us
            ));
            samples.push(Sample {
                client,
                latency,
                admission,
                queue: queue_us,
                held: published.writer_held_us,
            });
            // Keeps preparation + queued + active requests charged through commit.
            drop(request);
        }
        for producer in producers {
            producer.join().unwrap()?;
        }
        drop(shutdown);
        Ok(())
    })?;
    assert_eq!(publication::read(&pinned, 1)?, original);
    let usage = budget.snapshot();
    for client in [Client::Interactive, Client::Bulk] {
        let rows: Vec<_> = samples.iter().filter(|row| row.client == client).collect();
        let percentile = |mut values: Vec<u128>, percent: usize| {
            values.sort_unstable();
            values[(values.len() * percent).div_ceil(100) - 1]
        };
        let latency = |p| percentile(rows.iter().map(|r| r.latency).collect(), p);
        let own = usage.client_peaks[client.index()];
        summary.push_str(&format!(
            "{trial},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            client.name(),
            rows.len(),
            latency(50),
            latency(95),
            latency(99),
            percentile(rows.iter().map(|r| r.admission).collect(), 99),
            percentile(rows.iter().map(|r| r.queue).collect(), 99),
            rows.iter().map(|r| r.held).max().unwrap(),
            usage.peak.bytes,
            usage.peak.requests,
            own.bytes,
            own.requests,
            usage.current.bytes,
            usage.current.requests
        ));
    }
    Ok(())
}

pub fn verify(db: &Database) -> Result<()> {
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
    Ok(())
}
