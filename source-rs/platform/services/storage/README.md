# Object storage provider

A separate user task owns the exclusive boot-granted disk controller and the
[object store](../../libraries/database-objects/README.md). [main.rs](main.rs)
funds the heap, opens the guarded volume and recovers the engine, then
[service.rs](service.rs) accepts connections and dispatches the
[object protocol](../../contracts/storage.rs): one 64-byte packet each way, a
sealed attachment for publications and, through [bulk.rs](bulk.rs), a sealed
attachment for bulk reads. The private control connection answers health and
the supervisor's close; it conveys no object access.

Readiness is the exact `ready` reply to HEALTH after the engine and store open.
The [desktop startup controller](../../../distribution/desktop/session/persistence/startup.rs)
keeps display/input active while waiting, bounds readiness, and collects a failed
task and its control stream before manual retry. Opening never formats a missing
or damaged envelope. The `startup-probes` feature adds
[pre-open failure fixtures](startup_probe.rs) for death, silence and a malformed
reply; ordinary builds contain none of those argument-controlled paths.

Two boot-approved data links bind to private roots 0 and 1 by their
service-local link position. The kernel isolates principals and grants
endpoints; the provider enforces root-local object access, and no message
carries a root selector. Within a root, objects form a tree, and a request's
three scope words name subtrees it must stay inside: before dispatch the
provider checks that every stamped scope is an ancestor of every object the
request would touch, refusing an object outside the scope and one that does
not exist alike, and refusing to replace or delete the scope object itself.
Objects hold up to 256 KiB, so programs fit; a root has a 2 MiB retained-record
allowance. The lab volume occupies 8 MiB, including engine transaction headroom.
The desktop's root matrix holds both links and delegates
subtrees of them to its children; the [objects lab](../../../distribution/lab/objects/README.md)
hands them to two bare clients instead.

Every write is one engine transaction. Before a publication or copy (including
subtree copy), the provider performs **one** collection pass, removing at most
eight records. It does not drain an upgrade backlog inside an application's
save: repeated whole-root scans and durable commits exceeded the storage IPC
deadline on retained desktop disks. Later mutations or explicit COLLECT calls
continue maintenance. A root still at capacity can refuse a write even with
reclaimable history; that refusal publishes no partial record. Background
maintenance and quota-pressure scheduling remain future work. An engine or
corruption failure ends the provider rather than serving uncertain state, and
the distribution replaces it and regrants roots.

The `recovery-lab` feature adds the owner-controlled [fault gate](gate.rs),
which stalls one root's next matching request before execution or after
commit, so proofs can exercise lost acknowledgements and provider replacement.
`trace-storage` emits per-request timing and ATA counters for the
[timing summary](../../../../tools/boot-harness-rs/storage_timing.py). Ordinary
builds contain neither.

The provider's reader pages receive publication attachments; its private pages
hold the heap and the pages behind bulk replies, which stay charged to it until
the last reader releases them. Profiles fund both explicitly: 128 private pages
leave room for a 64-page reply beside the heap, and 64 reader pages take the
largest publication.

The engine cache is 64 KiB within the existing 56-page heap. A larger cache is
not free capacity: the engine also holds writer and allocator state. A 128 KiB
cache exhausted this heap during the desktop template copy. The 64 KiB setting
and ordered chunk-index validation reduce collection I/O; neither changes the
immediate durability barrier or permits reporting a save before commit.

Proofs, from the repository root:

```powershell
python tools/boot-harness-rs/objects_test.py --release
python tools/boot-harness-rs/objects_test.py --retention --release
python tools/boot-harness-rs/desktop_test.py
python tools/boot-harness-rs/matrix_database_test.py --release --recovery
```
