# Bounded publication admission

This host experiment reserves request resources before payload preparation and
keeps them charged through publication. Start at [admission.rs](admission.rs):
create a fresh budget/queue/database, check oversized refusal, run competing
producers, assert final accounting and verify reopened contents. Descend into
[budget.rs](budget.rs) for reservation/return, [queue.rs](queue.rs) for ready-work
dispatch, and [workload.rs](workload.rs) for preparation, commits and measurements.
[tests.rs](tests.rs) checks boundaries and failure paths. `mod.rs` is wiring only.

Run with the existing [stress suite](../README.md):

```powershell
python tools/storage-lab/engines/run.py --stress
```

The suite adds `admission.csv`, `admission-summary.csv` and three fresh databases
to its timestamped artifact directory. Source hashes include this folder. From
`tools/storage-lab/`, run `cargo test --locked --features engines` and explicitly
format-check the included body with
`rustfmt --edition 2024 --check engines/stress/admission/admission.rs`.

## Charge and ownership

A transaction reserves the sum of declared payload lengths plus **256 bytes per
object**. The overhead is an experimental accounting unit, not measured heap
overhead. Both an object count and a byte limit apply, including empty payloads.
Length addition is checked for overflow. These are lab settings, not OS limits:

| Limit | Setting |
| --- | ---: |
| Total in-flight charge | 512 KiB |
| Charge per assigned producer | 256 KiB |
| Total in-flight requests | 16 |
| Requests per assigned producer | 8 |
| Charge per atomic transaction | 128 KiB |
| Objects per atomic transaction | 8 |

In-flight means **preparing + queued + active**, rather than queued alone.
The two producers wait for a reservation before allocating fixture bytes or
hashing. A request owns immutable prepared bytes and the reservation. Validation
checks prepared lengths/count against the reservation; a queue accepts reservations
only from its own budget. Dequeue transfers ownership to the publisher without
returning charge. Payload fields are dropped before the reservation is released.

A reservation is returned on completion, conflict rejection, preparation failure,
queue refusal/cancellation and engine error. Closing stops admission, wakes waiters
and discards queued work. An active request retains its charge until its owner
drops it. Admission and dequeue waits have a 30-second lab timeout; this does not
interrupt an already running database commit or change its durability outcome.

The 4 MiB atomic import is rejected from its length descriptors, before payload
allocation or hashing, with no database changes. Admission never silently converts
it into smaller transactions. The bulk producer explicitly submits independent
64 KiB objects in separate publications. Larger atomic operations would need a
separately designed funded path or a clear refusal.

The budget bounds logical request contents and cardinality, not total RSS, vector
capacity, allocator fragmentation, stacks, caches, database pages or durable space.
Producer code is trusted in this lab and constructs exact declared lengths;
post-preparation validation cannot undo an allocation a malicious producer already
made. A guest implementation must admit/copy or map untrusted input against real
backing budgets before accepting it. This is not a new kernel quota mechanism.

## Dispatch and authority

Each of two assigned producer identities has a FIFO ready queue. When both queues
remain nonempty, the publisher alternates; if only one is ready it proceeds without
waiting for the other to finish preparation. A ready producer can therefore have
at most one other producer's dispatch between its own dispatches in that condition.
Its own earlier FIFO requests can still delay a particular request.

This is request-count fairness, not equal CPU time, IO bytes, latency, or interactive
priority. A transaction already committing remains non-preemptible by this queue.
The transaction cap bounds admitted content size, not engine IO amplification or
worst-case execution time. Three 64 KiB bulk requests fit the producer byte cap;
eight 4 KiB requests fit the interactive producer's count cap.

Producer identities are assigned by the experiment. Their names are workload labels,
not permission to claim higher priority. A future service must bind budgets to
authenticated capability/account ownership and bound the number of identities;
letting an app mint more producers would defeat per-producer isolation. No distro
priority or matrix sponsorship contract is selected here.

## Evidence and metrics

Eight policy tests cover transaction boundaries, overflow, empty/many objects,
global/per-producer byte/count limits, active reservation lifetime, ready-queue
alternation, mismatched payloads/budgets, cancellation/shutdown, preparation panic,
stale revisions and engine IO failure. The wider suite retains the existing
publication, crash and reclamation checks.

Each of three host trials publishes 64 interactive replacements and 64 independent
bulk objects. An original read transaction remains pinned and is checked again
after publication. Reopen checks every final object. Unlike the older FIFO contention
run, this phase does not repeatedly poll fresh snapshots on a separate reader
thread, so its timings are not a controlled queue-policy comparison.

`admission.csv` records producer, object count, charged bytes and these boundaries:

- `admission_us`: request start through obtaining its reservation, including waits.
- `preparation_us`: reservation acquisition through insertion into the ready queue,
  including fixture construction, hashing, validation and queue-lock acquisition.
- `queue_us`: ready-queue insertion through dequeue.
- `writer_wait_us`, `writer_held_us`: engine writer acquisition and publication.
- `latency_us`: request start through durable commit, including admission delay.

The summary reports nearest-rank p50/p95/p99 latency, p99 admission/queue delay,
maximum writer occupancy, total/per-producer high-water charges/counts and final
accounting. With 64 samples, p99 is the maximum observation, not a reliable estimate
of a production tail distribution. Latency includes waiting for admission rather
than hiding backpressure behind queue metrics.

Recorded run `20261016T184924232962Z` (Windows x86-64, Rust 1.94.0):

| Trial | Small-request p99 | Bulk-request p99 | Peak total charge | Peak requests | Final charge/count |
| --- | ---: | ---: | ---: | ---: | --- |
| 0 | 23.421 ms | 13.734 ms | 232,192 bytes | 11 | 0 / 0 |
| 1 | 26.370 ms | 16.377 ms | 232,192 bytes | 11 | 0 / 0 |
| 2 | 25.051 ms | 15.050 ms | 232,192 bytes | 11 | 0 / 0 |

Each trial peaked at 34,816 bytes/eight requests for interactive work and 197,376
bytes/three requests for bulk. All 384 publications completed. Tests check the
policy bounds and ready-queue selection; these timings do not establish a speedup.
Small requests can wait behind their own eight outstanding requests, while bulk
has fewer outstanding requests and gets an alternating turn.

Next measure responsiveness under controlled arrival rates and different producer
windows. A future priority/weight policy must state its fairness and authority
rules. Engine caches, reader pins, maintenance space and external blob durability
remain separate obligations; this admission budget does not fund them.
