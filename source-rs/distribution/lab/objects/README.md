# Private content publication proof

[main.rs](main.rs) selects the supervisor or ordinary client. The
[supervisor](supervisor.rs) launches the separately compiled
[platform provider](../../../platform/services/storage/main.rs), then
two [clients](client.rs). Each client upgrades its one explicit link to a retained
shared-memory connection. It has five private pages (four for content, one for
its mailbox), one mailbox reader and one notification allowance. It receives a
clock for connection setup, but no disk or sibling connection. Retention clients
get four additional reader pages for bulk replies.

The supervisor retains a separate shared owner-control connection from HEALTH
through final CLOSE. The provider receives the disk, 83 private pages, seven
reader pages and three notification allowances. Its 56-page heap, eight output
pages and three mailbox writers leave 16 pages of additional funding: boot derives
physical reservations from the private-page grant, and the current image plus
translation tables exceeds its fixed 288-frame startup allowance. Four provider
readers remain available for sealed input attachments beside its three mailboxes.
These are explicit fixture grants, not increased kernel limits.
Profile link ordering binds provider slots to persistent roots 0 and 1; reordering
those links on an existing image is unsupported. These are private fixture roots,
not a proposal for global object identity.

Run from the repository root:

```powershell
python tools/boot-harness-rs/objects_test.py --release
python tools/boot-harness-rs/objects_test.py
python tools/boot-harness-rs/objects_test.py --retention --release
python tools/boot-harness-rs/objects_test.py --retention --timeout 240
python tools/boot-harness-rs/objects_test.py --release --ata-trace
```

The runner formats a new 8 MiB volume, surrounds it with guard bytes and boots
the separate `objects.json` composition. Each root creates an 8 KiB object with
distinct deterministic contents. Clients reconstruct it through revision-pinned
range reads and check stale-write, oversized-request, short-attachment and
retained-quota refusals. They also reject supervisor-only commands, extra root
selectors, overflowing ranges and stale reads. Refusals leave the current head
and bytes unchanged. The next valid write after refusal must still succeed.

After both clients have received success, the runner kills QEMU with the database
still open. The second cold guest verifies the previous contents, replaces one
byte, verifies revision 2, and closes the provider. Its heap allocations and
reader-page charges must return to zero. After collecting the provider and retiring
the control connection, the supervisor also checks zero writer/reader/channel
charges and notification objects/rights. The independent host layout model checks
the entire retained record set on **copies** of both images: chunks, manifests,
heads, events, quota ledgers, and absence of records from refused transactions.
Opening those copies cannot clean the original before guest recovery.

Artifacts under `build/boot-harness-rs/objects/<timestamp>/` include both serial
logs, original and recovered images, host verification results, heap samples,
source hashes, EFI hash and tool versions. `heap-*.txt` samples requested live
bytes, cumulative peak, largest allocation and allocation failures after engine
recovery, content preparation and commit, plus final close. Peak is cumulative,
not a separate maximum for each phase. Allocator metadata/fragmentation, stack,
code, kernel storage and sealed reader pages are outside those numbers.

The retention profile selects [retention.rs](retention.rs). Boot one pins revision
1 and replaces through revision 9, collecting between writes. Boot two recovers
both heads, refuses a stale release, releases the pin, then replaces through 25.
Boot three verifies current contents and the absence of the collected revision.
The host oracle checks exactly 34 records after boot one and 24 after each later
boot. Sealed bulk replies exercise full reads, a cross-chunk partial read, zero
page padding, reader-page exhaustion returning NO_MEMORY, and reuse after dropping
read objects. Two retained replies fill four data-reader pages; accounting must
show five readers including the mailbox, then one after both replies are dropped.
These backings are charged to the provider until their last reader
releases them; there is no general per-client fairness or sponsorship contract yet.

One early release run hit the ATA readiness deadline (`status=c0`, busy) during
database initialization (`20261017T005941877071Z`). The cause remains undiagnosed.
The driver now reports opcode, phase, LBA, status, elapsed ticks and poll count on
errors/timeouts and slow successful waits. `--ata-trace` also records QEMU ATA and
block events per boot. The instrumented two-boot release run
`20261017T020820017190Z` passed without reproducing the stall. The 50-tick deadline
is unchanged; no write retries hide an uncertain device outcome.

This is a bounded, synchronous lab: 16 KiB maximum contents, 4 KiB chunks,
inline contents through 128 bytes, and a 24 KiB retained-record quota per root.
Each client uses object number one of its root; numbering, copies and deletion
are covered by the store's host tests and the matrix storage fixture. The 40-byte
inline read and sealed bulk replies both use shared-memory transport; typed
attachment grants still cross the kernel capability boundary. Two durable
root-owned pins and eight-record collection
passes are experimental; there is no public watcher subscription, cross-root
transaction, encryption, installer or accepted production retention policy.
Quota refusal is distinct from physical full-disk progress. QEMU termination does
not emulate host-cache loss or physical power loss.

The shared publication policy tests run inside `source-rs/`:

```powershell
cargo test --locked -p cathedral-database-objects --features host
```

They check rollback of every record, writer revalidation after competing
preparation, exact quota accounting, corruption detection, range boundaries,
inline/chunk transitions and root isolation. New tests cold-open after every
collection pass, verify remaining event references can still be pinned, and
perform 600 8 KiB replacements alongside 100 unrelated 4 KiB records within a
fixed 1,056,768-byte engine extent. This demonstrates reuse for that trace, not
universal progress on a physically full volume. Existing engine/volume fault labs
remain the evidence for simulated write/sync failures; these collection tests do
not inject failures inside each commit.

The dev profile size-optimizes the engine, content libraries, provider, ATA driver,
contracts, heap allocator and user runtime. redb and database-objects use `z`
optimization. Those two packages, the provider and runtime use one codegen unit.
`debug_assert!` is disabled in redb, database-objects and the provider; explicit
validation/assertions remain enabled. Runtime assertions remain enabled. The dev
provider occupies 238 image pages; the kernel's 240-page limit is unchanged.

The initial publication traces passed in debug (`20261017T010736771525Z`) and
release (`20261017T010906650068Z`). Retention and bulk reads passed the three-boot
release trace `20261017T021823542776Z` and dev trace `20261017T022849159913Z`;
their final closes report zero live heap bytes,
zero allocation failures and a largest allocation of 9,432 bytes. Consult each
run's `heap-*.txt` for per-boot cumulative peaks rather than treating a sample as
an all-workload memory bound.

The matrix desktop and Notes use this same provider through delegated roots;
the [object store guide](../../../platform/libraries/database-objects/README.md)
owns the representation.

The shared-connection migration passed both cold-boot traces on 2026-10-26:

| Trace | Release artifact | Dev artifact |
| --- | --- | --- |
| Publication, refusal and recovery (two boots) | `20261026T163302147852Z` | `20261026T162448552403Z` |
| Pins, collection and reader pressure (three boots) | `20261026T163323136823Z` | `20261026T162537287143Z` |

All four runs check the transport markers, exact host records, neighbor guards,
zero heap allocation failures and final owner-connection reclamation. These
remain bounded persistence traces, not power-failure or performance guarantees.
