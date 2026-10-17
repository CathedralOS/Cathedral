# Catalog transactions over content storage

[lib.rs](lib.rs) opens two root snapshots and implements the existing catalog
[backend interface](../protocol.rs). Each transaction validates the entire change
set, collects older garbage, publishes one immutable root snapshot, then updates
the provider's cache after durable commit. [snapshot.rs](snapshot.rs) owns its
152-byte canonical payload. It is separate from the legacy two-sector disk format.

One provider owns both the cache and database writer. Root generations equal
content revisions; open verifies that relationship. The catalog still has four
32-byte objects per private root and two changes per atomic transaction. The
protocol keeps per-connection staging and clears it on incarnation change. No new
matrix opcode, global namespace or kernel database mechanism is required.

This mode retains the current revision. Collection runs before the next write,
so the preceding revision can remain as garbage until then. It exposes neither
history pins nor watcher replay. An engine/corruption failure retires the provider;
it does not continue serving cached state after uncertain storage effects.

The [provider entry](../../../services/storage/database/main.rs) selects this mode
with `catalog`; [catalog.rs](../../../services/storage/database/catalog.rs) serves
its IPC. Follow [matrix storage forwarding](../../../services/matrix/service/parent/storage/storage.rs)
across the executable seam. The distribution chooses the replaceable backend in
[matrix-database.json](../../../../../tools/boot-harness-rs/profiles/matrix-database.json).

From the repository root:

```powershell
python tools/boot-harness-rs/matrix_database_test.py --release
python tools/boot-harness-rs/matrix_database_test.py --release --recovery
python tools/boot-harness-rs/run.py --profile tools/boot-harness-rs/profiles/matrix-database.json --release --window
```

The runner creates a fresh bounded database volume, saves each app's color through
physical input and its matrix-local capability, then cold-boots to restore both.
An independent host oracle checks all snapshots, chunks, manifests, events and
quota records on image copies. Guards outside the volume must remain unchanged.
This is the ordinary desktop path, not the legacy provider's injected recovery
protocol; those recovery tests retain their existing composition.

The ordinary two-boot release run `20261017T025617017076Z` and the recovery run
`20261017T032755977723Z` each passed with 16 exact private records on both boots.
Artifacts live under `build/boot-harness-rs/matrix-database/`. QMP quit
leaves the guest database open but does not model loss of physical device caches.
No existing desktop disk is migrated or reformatted.

The desktop still requires `--release` under TCG. The original dev timeout
(`20261017T023355118118Z`) exposed an app bug: CREATE unwrapped a storage error.
[App state](../../../../distribution/lab/matrix/desktop/app/state.rs) now retains
confirmed state, reports failure visibly, and reads before any subsequent mutation.
Enter on a red error mark reloads the current grant and stored value; it never
blindly repeats a toggle. A failed CREATE may have committed, so readback settles
it. Only the explicitly diskless sibling probe uses volatile state.

The [recovery exercise](../../../../distribution/lab/matrix/desktop/database_recovery.rs)
lets a committed CREATE exceed the actual matrix deadline, checks the app survives
and shows its error, then lets the owner detect and replace the live stalled provider. The distribution's ordinary
[supervisor](../../../../distribution/lab/matrix/desktop/persistence/persistence.rs) replaces
the dead provider and explicitly re-grants both roots through the existing matrix
tree. Readback finds revision 1 without duplicating CREATE. Further cuts before
REPLACE and after its commit preserve the correct old/new values and exact
revisions. A sibling in the same nested matrix continues pointer/key handling and
painting while saves are blocked. Physical input then saves both apps, and a cold
boot plus independent host oracle verify all 16 records (root revisions 4 and 2).

The ordinary supervisor checks once per 100 guest ticks. It replaces dead providers,
providers whose control health probe expires (300 ticks, probes spaced by 500 ticks),
and providers whose data stream the root matrix has already retired. The root's
owner-only `STORAGE_STATUS` prevents a resumed provider's healthy control reply from
hiding an unusable old stream. Children cannot query that status or reconnect it.
Cancellation and collection precede revocation, reopen and explicit grants.

Replacement authority is capped at three attempts per desktop session; healthy
replies never reset that allowance. Exhaustion or replacement failure leaves storage
offline with grants revoked while the matrix/apps remain alive. Startup/open failure
still fails closed. These are bounded lab policy choices, not production restart
policy or automatic media repair. Loss of the app's own uncorrelated transport still
requires app replacement. Apps keep confirmed state and reconcile fresh grants on
Enter; no uncertain mutation is replayed automatically.

VS Code's **Cathedral: Database Desktop in QEMU** entry retains
`build/boot-harness-rs/matrix-database-storage.raw`. The host creates this guarded
volume only when absent; an existing file is never reformatted. The guest validates
its envelope/engine on every open. The older catalog desktop remains a separate
launch option and disk. No migration is performed.

`--recovery` selects test-only provider gates and `trace-storage`. Artifacts include
`timing-N.csv` and `ata-N.csv` alongside serial, scanout and independent host checks.
Add `--trace-storage` to an ordinary runner command for diagnostics without fault
injection. [The timing summarizer](../../../../../tools/boot-harness-rs/storage_timing.py)
records app round trips, matrix dispatch/service, provider queueing/execution,
collection and commit. Scope numbers are producer-local. The provider-queue join
uses this fixture's fixed slot-to-root mapping and has no cross-process CPU delta.
Matrix dispatch starts when the event is received; prior mailbox/backpressure time
is included in outer round trips, not that span. ATA samples count read/write/flush
commands, phase-wait ticks and polls, not physical-media work. Ticks are nominally
10 ms, logging perturbs execution, and the recovery trace includes deliberate waits;
these are diagnostics, not latency guarantees. Missing ends after process death
are not converted into successful timings.

Inside `source-rs/`, run `cargo test --locked -p cathedral-database-catalog --features host`
for atomic changes, stale revisions, staging abandonment, root isolation, 300
replacements and reopen. The engine remains an experiment, not an accepted DBFS ABI.

The [notes scratchpad](../../../../distribution/applications/notes/README.md) adds
physical text editing, atomic two-record saves, draft-preserving recovery and a
measured ordinary save workload over these same private catalog capabilities.
