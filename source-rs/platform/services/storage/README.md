# Object storage provider

A separate user task owns the exclusive boot-granted disk controller and the
[object store](../../libraries/database-objects/README.md). [main.rs](main.rs)
funds the heap, opens the guarded volume and recovers the engine, then
[service.rs](service.rs) accepts connections and dispatches the
[object protocol](../../contracts/storage.rs): one 64-byte packet each way, a
sealed attachment for publications and, through [bulk.rs](bulk.rs), a sealed
attachment for bulk reads. A bulk read with no length fetches the whole object
at its current revision, so a launcher reads a program in one request without
a HEAD first, and ENTRIES ([listing.rs](listing.rs)) answers a parent's direct
children with their numbers, revisions and metadata in one sealed page, so a
realm is named in one request rather than one LOOKUP or METADATA per child.
The private control connection answers health and the supervisor's close; it
conveys no object access.

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
two scope words name subtrees it must stay inside: before dispatch the
provider checks that every stamped scope is an ancestor of every object the
request would touch, refusing an object outside the scope and one that does
not exist alike, and refusing to replace or delete the scope object itself.
Objects hold up to 256 KiB, so programs fit; a root has a 2 MiB retained-record
allowance. The lab volume occupies 8 MiB, including engine transaction headroom.
The desktop's root matrix holds both links and delegates
subtrees of them to its children; the [objects lab](../../../distribution/lab/objects/README.md)
hands them to two bare clients instead.

Every write is one engine transaction. Before a publication, metadata update or
copy (including subtree copy), the provider checks the durable retained-byte
ledger. Below half the 2 MiB quota it skips collection. At or above that
watermark it performs **one** collection pass, removing at most eight records.
This maintenance trigger stays independent of the maximum object size. Every
mutation still checks exact quota, including publication/index overhead.
This avoids scanning all installed programs on every tiny save or seed copy.
Old unpinned revisions remain eligible for collection, with no retention promise.
Explicit COLLECT always performs a pass, independent of the watermark. Later
mutations under pressure continue bounded maintenance; no request drains the
backlog. A root still at capacity can refuse a write even with reclaimable
history; that refusal publishes no partial record. Physical engine headroom
remains independent of logical quota. Background maintenance is future work.
A known extent allocation refusal before commit drops and reopens the engine through
[database.rs](database.rs), retaining authenticated service connections and
returning ordinary capacity refusal. Opening the existing object table is read-only.
Commit errors remain uncertain, including capacity errors during commit. Other engine or
corruption failures end the provider rather than serving uncertain state, and
the distribution replaces it and regrants roots.

The `recovery-lab` feature adds the owner-controlled [fault gate](gate.rs),
which stalls one root's next matching request before execution or after
commit, so proofs can exercise lost acknowledgements and provider replacement.
`trace-storage` emits per-request timing and ATA counters for the
[timing summary](../../../../tools/boot-harness-rs/storage_timing.py).
`collection-pressure` makes the store collect before every write inside a
2 KiB workspace, so the desktop soak can watch windowed collection run and
split on real records. Ordinary builds contain none of these.

The provider's reader pages receive publication attachments; its private pages
hold the heap and the pages behind bulk replies, which stay charged to it until
the last reader releases them. Profiles fund both explicitly. [main.rs](main.rs)
splits funded private pages between the engine/cache and outstanding replies,
after eight pages of headroom. The heap stays between its tested 56-page minimum
and two private objects, leaving object slots available for replies. The lab's
64-page grant keeps the smallest heap; the desktop's 328 pages provide a 160-page
(640 KiB) heap and enough remaining backing for the reusable UI program. A maximum
protocol payload still requires sufficient actual provider funding. A reply
whose pages cannot be allocated is recorded as `bulk-reply-refused` in every
build, with the page count, the error and the provider's page and frame budgets;
the kernel's per-task mapped-backing ceiling covers every private slot plus two
reader mappings, so the heap's objects never crowd a reply out.

[database.rs](database.rs) sizes the engine page cache from the heap: 256 KiB
is reserved for engine state, the writer and the collection workspace, and the
rest is cache, never less than 32 KiB. The smallest heap therefore keeps its
32 KiB cache, which holds only eight pages, so after one bulk program read every
metadata lookup went back to the disk and scope-ancestry checks grew with depth.
The desktop's 640 KiB heap gives the engine 384 KiB of cache, 96 pages, which
holds the metadata working set beside the stock programs every nested level
reads. A content chunk fills one engine page, so a program read moves its own
bytes and no more through that cache; with 4 KiB chunks every chunk was a
two-page leaf, a matrix program read moved a megabyte, and the 256 KiB cache
of the previous heap lost its metadata pages to each launch.

Collection runs inside a fixed workspace, independent of the root's record
count: the [store](../../libraries/database-objects/README.md) marks one record
kind at a time inside one identity window, splitting a window that would exceed
24 KiB of marks. A census that cannot fit even at the finest split is reported
as `Error::Workspace`; this provider then skips that maintenance turn, records
`collect-workspace-exhausted` in every build, and lets exact quota admit or
refuse the write. Nothing is removed on that path, and pins are untouched.
An explicit COLLECT answers it with NO_MEMORY. `trace-storage` reports heap use
before collection, before hierarchy writes and after them, including the
allocator's lifetime peak, failed allocation count and capacity, beside the
private pages and frames the kernel charges the provider, where a reply page
its reader never released would show, and samples
each automatic pass as `collect-pass` with its removals, ledger, windows and
largest marked set.

Proofs, from the repository root:

```powershell
python tools/boot-harness-rs/objects_test.py --release
python tools/boot-harness-rs/objects_test.py --retention --release
python tools/boot-harness-rs/desktop_test.py
python tools/boot-harness-rs/matrix_database_test.py --release --recovery
```
