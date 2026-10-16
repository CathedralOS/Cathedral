# Publication, contention and recovery lab

This redb-only host experiment tests performance boundaries before guest integration.
Start at [stress.rs](stress.rs): check conflicting publications, run competing
requests, enumerate storage failure cuts, then exercise reader pressure and reuse.
The [parent comparison](../README.md) owns engine selection context and dependency
provenance. Neither experiment defines a production DBFS format or API.

From the repository root:

```powershell
python tools/storage-lab/engines/run.py --stress
```

The runner builds with locked dependencies, creates a fresh timestamped directory
under `build/storage-stress/`, validates CSV coverage and saves source hashes,
compiler/host/revision context and `checks.txt`. No existing files are overwritten.
As with the comparison runner, do not concurrently rebuild the same executable
with different features. Run host tests/lint in `tools/storage-lab/`; explicitly
format-check `engines/stress/stress.rs` because Rust's `include!` hides that body
from `cargo fmt` discovery.

## Preparation and publication

[publication.rs](publication.rs) owns a small experimental record protocol:
object ID to revision/digest head, digest to payload, and object/revision to change
record. Prepared requests own their bytes and precomputed BLAKE3 digest; callers
cannot mutate those fields after preparation. Hashing occurs before acquiring the
writer. Inside one immediate-durability transaction, the publisher validates all
expected revisions, then inserts content, heads and events. Payload insertion and
engine IO still occur under the writer: this is not an external blob allocator.

Two barrier-synchronized threads prepare the same expected revision. Exactly one
must commit, with no records from the rejected writer. A second check places a
conflict late in a multi-object batch and verifies that the entire batch is rejected.
These checks prevent lost updates for explicitly checked objects; they do not
provide general read-set, range/predicate or cross-service isolation semantics.

The normal mode retains immutable content and events. A separate retirement mode
deletes the previous content/event in the same publication. That mode is used only
for a single object with unique payloads: it is not shared-content GC, durable
subscriber retention, or a production history policy.

## Competing workloads

[contention.rs](contention.rs) starts two preparing producers, one publisher and
a concurrent reader over a real host file backend. The request queue holds at most
eight messages; each producer can additionally own one request blocked on admission.
The publisher can own one active request. The bounded request sizes make this
experiment finite, but a production queue also needs an explicit byte budget.

- The interactive producer sends 64 successive 4 KiB replacements of one object.
- The bulk producer imports 64 distinct 64 KiB objects: 4 MiB total.
- One mode groups all bulk objects into a single transaction; the other uses 64
  transactions. They have the same final contents and different atomicity.
- The reader repeatedly verifies its original pinned version and fresh snapshots
  while writes run, checking content hashes, matching events and monotonic revisions.
- Reopen verifies the final interactive object and every imported object.

The single publisher deliberately puts contention in its bounded queue. It does
not demonstrate parallel engine writers or fairness guarantees. The separate
conflict check exercises simultaneous writer acquisition. Reader checks sleep
one millisecond between iterations; they are correctness probes, not read-throughput
measurements. Old application records remain retained in both contention modes.

`contention.csv` records each request:

| Metric | Meaning |
| --- | --- |
| `preparation_us` | Fixture construction, owned buffers and hashing before enqueue |
| `queue_us` | From attempting to enqueue until receipt, including blocked admission |
| `writer_wait_us` | Time acquiring the engine writer; expected to be small with one publisher |
| `writer_held_us` | Transaction configuration, validation, insertion and durable commit |
| `latency_us` | Preparation start through durable commit, including queue delay |

`contention-summary.csv` uses nearest-rank empirical percentiles, includes sample
counts, maximum writer occupancy, reader checks and file extents. With 64 requests,
p99 is the maximum sample; with one bulk request, every percentile is that one
sample. These are diagnostic samples, not estimates of production tail latency.

Recorded Windows x86-64/Rust 1.94.0 run `20261016T183811622594Z`:

| Bulk transaction size | Small-write p50 | Small-write p95 | Small-write p99 | Maximum bulk writer occupancy |
| --- | ---: | ---: | ---: | ---: |
| 64 objects / 4 MiB | 7.770 ms | 25.372 ms | 29.466 ms | 17.648 ms |
| 1 object / 64 KiB | 10.602 ms | 18.131 ms | 18.712 ms | 7.538 ms |

The sample exposes a transaction-size/latency tradeoff. Smaller publications did
not improve every metric and require more commits. The experiment changes commit
granularity explicitly; it does not silently split a requested atomic transaction.
Host scheduling, filesystem cache and flush behavior affect every timing. There is
no throughput ranking, guest/QEMU measurement or latency acceptance threshold.

## Simulated failures

[device.rs](device.rs) implements separate volatile and durable byte images. Reads
see volatile data. A successful sync makes all preceding writes/length changes
durable. A power cut freezes further writes and syncs before engine destructors
run. Reopening gets only a copy of the durable image in a new device instance.
Independent model tests check barrier preservation, partial persistence, capacity
refusal and loss of unflushed data.

[faults.rs](faults.rs) records each mutating backend call during a successful
publication, then repeats from the same baseline with a cut at every recorded
call. It tests 4 KiB and 2 MiB replacements, with old records retained or retired.
For every write, resize and sync boundary it exercises:

- `Before`: fail before that operation; discard earlier unflushed changes.
- `Partial`: persist the first half of the current write, or the first half of
  the volatile file for a sync, then fail. A resize has no partial persistence.
- `After`: persist that operation and then report failure. An individual write
  may persist without earlier unflushed writes; a completed sync persists them all.

After a successful barrier, later cuts never undo that barrier's effects. File
growth exposes zero-filled bytes. The capacity check applies before growth/write;
limits do not model host memory exhaustion. This is a deterministic byte model,
not a disk cache, sector-atomicity or controller simulator.

The recorded run passed **150 cuts** (138 old states, 12 new states), plus four
successful commits followed immediately by simulated loss of volatile state.
Every reopened full table must equal the complete baseline or complete new table.
Recovered heads must point to present, correctly hashed payloads and matching
events. Every acknowledged publication must recover new. Failure need not mean
rollback: a failed operation can have reached durable storage.

Coverage is bounded: two payload sizes, one update per fresh baseline, three cut
patterns per observed mutation boundary. It does not enumerate arbitrary sector
subsets, all reorderings, bit rot, failures during recovery, hardware flush lies,
or all allocator states. Engine object destruction is prevented from changing the
captured image, but this is an in-process simulated crash, not an actual power cut.

## Reader pressure and reclamation

[reclamation.rs](reclamation.rs) compares 17 revisions of a 256 KiB object with
and without an old reader pinned. Each publication retires the previous records;
the reader must nevertheless retain its old bytes through engine snapshot isolation.
The no-reader control reached a 2,109,440-byte extent. The pinned-reader case reached
16,846,848 bytes. File extent includes engine reserve and metadata; the difference
is not a measurement of the exact number of pinned physical pages.

After releasing the reader, the test caps the device at that same 16,846,848 bytes
and completes **289 more replacements** without reopening, raising the cap or
requesting compaction. Their cumulative payload is more than four times the cap,
demonstrating reuse. The engine's logical extent eventually falls to 4,861,952 bytes.
The report records every revision/extent instead of assuming release immediately
shrinks the file.

An oversized replacement must then fail specifically through capacity refusal.
A simulated reboot retains the cap, recovers the exact prior records, and accepts
another normal replacement. Unlike the earlier comparison's capacity test, this
test never lifts the cap. It demonstrates this finite replacement workload can
continue with reclaimed space; it does not prove progress at arbitrary disk fullness,
bound production GC latency or decide when live readers may be denied/expired.

## Design consequences and next evidence

Keep preparation independent of writer acquisition, validate expected versions at
publication, expose batching explicitly, bound admitted bytes as well as request
count, and measure queueing separately from engine work. Keep reader retention
funded and make engine reclamation progress observable. Our current all-in-engine
baseline still copies/inserts bulk payload while holding the writer.

Next evaluate a byte-budgeted publisher with explicit transaction-size admission,
then extend failure/reuse tests to longer histories and recovery failures. Moving
payloads outside the engine may reduce writer occupancy, but creates a new ordering,
orphan-recovery and accounting boundary that must be proven before guest adoption.
The [existing owner questions](../../../../OWNER_QUESTIONS.md#dbfs-history-and-durable-subscriber-limits)
still own retention and general isolation guarantees. These finite experiments
do not select those semantics or change the guest storage provider.
