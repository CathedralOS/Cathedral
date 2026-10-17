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
```

The runner creates a fresh bounded database volume, saves each app's color through
physical input and its matrix-local capability, then cold-boots to restore both.
An independent host oracle checks all snapshots, chunks, manifests, events and
quota records on image copies. Guards outside the volume must remain unchanged.
This is the ordinary desktop path, not the legacy provider's injected recovery
protocol; those recovery tests retain their existing composition.

The two-boot release run `20261017T022558426100Z` passed with 16 exact private
records. Artifacts live under `build/boot-harness-rs/matrix-database/`. QMP quit
leaves the guest database open but does not model loss of physical device caches.
No existing desktop disk is migrated or reformatted.

This desktop proof requires `--release`. The dev attempt
`20261017T023355118118Z` exceeded a matrix storage-response deadline during app
initialization; the app's CREATE currently unwraps that timeout. A timeout may
hide a committed write, so increasing deadlines or blindly retrying is not a
general fix. App startup reconciliation and storage latency under slower builds
remain engineering work. The separate private-content/retention proof passes in
both dev and release and does not use the desktop's response deadline.

Inside `source-rs/`, run `cargo test --locked -p cathedral-database-catalog --features host`
for atomic changes, stale revisions, staging abandonment, root isolation, 300
replacements and reopen. The engine remains an experiment, not an accepted DBFS ABI.
