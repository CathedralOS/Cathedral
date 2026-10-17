# Private notes scratchpad

An ordinary application of the matrix/catalog APIs. Press **F5**, click **Notes**
in Programs, then click the workspace to focus it. The equivalent default boot is:

```powershell
python tools/boot-harness-rs/run.py --release --window
```

The [notes fixture](../../../../tools/boot-harness-rs/profiles/notes.json) also runs
two instances with separate private catalogs, one inside a nested matrix, for
isolation and recovery tests. Neither knows its position, siblings, disk provider
or the root owner's diagnostics.

Click a note to focus it. Type letters, digits, spaces or Enter for a newline;
Left/Right move the insertion cursor and Backspace deletes. F1 saves. Green means
the current text matches confirmed storage, amber means an unsaved draft, and red
means storage needs reconciliation. On red, F1 reads current storage while keeping
the draft; a subsequent F1 deliberately saves any remaining difference. F2 reloads
and discards the draft only after a successful read. A failed read never erases it.

This is a **60-character ASCII scratchpad**, with lowercase stored text and the
existing uppercase-shaped diagnostic font. Shift, layout selection, selection,
clipboard, IME and general documents are not implemented. The limit follows the
existing two-change/32-byte catalog protocol; it is not a proposed DBFS file limit.
Larger notes should use the bulk-content protocol through explicit matrix grants,
not expand this application encoding into a second filesystem.

## Follow execution

[main.rs](main.rs) enters [session/session.rs](session/session.rs): load supplied
connections, read confirmed state, process local input, save/reconcile, present.
[document.rs](document.rs) owns bounded edits and the `NTS1` schema.
[storage.rs](session/storage.rs) splits that schema across objects 1 and 2, stages
both, and publishes one catalog transaction. It validates a common generation on
read and never silently retries an uncertain commit. A lost app transport remains
retired; app replacement is required because replies lack correlation IDs.

Storage IPC enters the parent's
[storage coordinator](../../../platform/services/matrix/service/parent/storage/storage.rs),
then the [database provider](../../../platform/services/storage/database/catalog.rs).
The [distribution supervisor](../../desktop/session/persistence/persistence.rs)
owns provider replacement and explicit re-grants. The app owns only its local
binding and optimistic revision. Another app's replacement grant does not revive
this app's cached binding: the first refused save preserves its draft, then F1
reconciles the new binding.

[view.rs](session/view.rs) publishes a sealed 180x64 bitmap using the existing
surface chain. Each app has 24 private pages for the retained 12-page image plus
its replacement. Matrix/display reader budgets are explicitly funded by the
profile. These allocations are bounded; repeated typing exercises their reuse.
No kernel, device or storage policy is added for this app.

## Proof and measurements

```powershell
python tools/boot-harness-rs/matrix_database_test.py --release --notes
python tools/boot-harness-rs/matrix_database_test.py --release --notes-recovery
```

Both use disposable guarded images and two fresh QEMU processes. Physical key
events edit/save notes at several lengths, cross the record boundary, hit capacity,
discard an unsaved draft, and verify focus isolation. Every text pixel and status
color is compared after cold boot. An independent host oracle checks all 16 retained
records, including prior/current snapshots, content, events and exact quota charges.
The recovery run `20261017T032825500398Z` passed with the same exact record checks.
The recovery profile stalls after the first commit but before its reply. Ordinary
supervision restores grants; F1 reconciles without an extra revision and the sibling
keeps its own unsaved draft through binding replacement.

Artifacts include serial logs, screenshots, source hashes, timing and ATA CSVs.
The ordinary run `20261017T032932558231Z` under
`build/boot-harness-rs/matrix-database/` measured six saves:

| Stage | Minimum / median / maximum nominal ticks |
| --- | --- |
| Whole app save transaction | 7 / 13.5 / 26 |
| Database commit | 4 / 9.5 / 11 |
| Pre-publication collection | 0 / 2 / 12 |

One tick is nominally 10 ms. Each save's provider request issued 41Ã¢â‚¬â€œ130 single-sector
writes (20.5Ã¢â‚¬â€œ65 KiB) and one or two flushes, including collection when needed.
The app publishes a 152-byte catalog snapshot regardless of note length; these
numbers do not characterize bulk files. TCG, diagnostic logging, coarse clocks and
the tiny six-save sample prevent latency guarantees. One startup provider-queue
sample was 97 ticks; it is separate from the six save spans. The evidence points
first to commit/collection and startup scheduling, not to text rendering SIMD.

The default desktop disk is `build/boot-harness-rs/cathedral-storage.raw`, created
only when absent and never auto-reformatted. The isolated notes fixture retains
its earlier `notes-storage.raw`; existing lab images are not migrated or deleted.
QEMU termination does not emulate physical device-cache loss.
