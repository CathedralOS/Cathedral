// Reserve before preparation, validate the prepared request, dispatch fairly,
// and retain its charge until completion. The workload owns engine interaction.
use super::publication;
use super::{Result, publication::Prepared};
use crate::engines::backend::Backend;
use budget::{Budget, Denied, Limits, Reservation};
use queue::Queue;
use std::{path::Path, time::Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Client {
    Interactive,
    Bulk,
}
impl Client {
    fn index(self) -> usize {
        match self {
            Self::Interactive => 0,
            Self::Bulk => 1,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Interactive => "small",
            Self::Bulk => "bulk",
        }
    }
}

pub struct Request {
    // Rust drops fields in declaration order: free payloads before releasing charge.
    changes: Vec<Prepared>,
    reservation: Reservation,
    started: Instant,
    reserved: Instant,
    queued: Instant,
}
impl Request {
    fn prepared(
        reservation: Reservation,
        changes: Vec<Prepared>,
        started: Instant,
        reserved: Instant,
    ) -> std::result::Result<Self, Denied> {
        // Prepared fields are private and owned; charge cannot change after this check.
        reservation.validate(
            &changes
                .iter()
                .map(Prepared::payload_len)
                .collect::<Vec<_>>(),
        )?;
        Ok(Self {
            reservation,
            changes,
            started,
            reserved,
            queued: Instant::now(),
        })
    }
}

struct Shutdown<'a>(&'a Budget, &'a Queue);
impl Drop for Shutdown<'_> {
    fn drop(&mut self) {
        self.0.close();
        self.1.close();
    }
}

pub fn run(directory: &Path) -> Result<()> {
    let mut samples = String::from(
        "trial,kind,objects,charge_bytes,admission_us,preparation_us,queue_us,writer_wait_us,writer_held_us,latency_us\n",
    );
    let mut summary = String::from(
        "trial,kind,requests,latency_p50_us,latency_p95_us,latency_p99_us,admission_p99_us,queue_p99_us,writer_held_max_us,peak_bytes,peak_requests,client_peak_bytes,client_peak_requests,final_bytes,final_requests\n",
    );
    for trial in 0..3 {
        let limits = Limits::default();
        let budget = Budget::new(limits);
        let queue = Queue::new(budget.clone());
        let path = directory.join(format!("admission-{trial}.db"));
        let db = publication::open(Backend::open(&path)?)?;
        publication::initialize(&db)?;
        workload::check_refusal(&db, &budget)?;
        workload::compete(trial, &db, &budget, &queue, &mut samples, &mut summary)?;
        let usage = budget.snapshot();
        assert_eq!(usage.current.bytes, 0);
        assert_eq!(usage.current.requests, 0);
        assert!(usage.peak.bytes <= limits.bytes && usage.peak.requests <= limits.requests);
        assert!(
            usage
                .client_peaks
                .iter()
                .all(|p| p.bytes <= limits.client_bytes && p.requests <= limits.client_requests)
        );
        drop(db);
        let reopened = publication::open(Backend::open(&path)?)?;
        workload::verify(&reopened)?;
    }
    std::fs::write(directory.join("admission.csv"), samples)?;
    std::fs::write(directory.join("admission-summary.csv"), summary)?;
    println!(
        "VERIFIED: byte/transaction admission, fair dispatch, reservation release and reopened contents"
    );
    Ok(())
}
