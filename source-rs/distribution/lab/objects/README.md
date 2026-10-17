# Private content publication proof

[main.rs](main.rs) selects the supervisor or ordinary client. The
[supervisor](supervisor.rs) launches the separately compiled
[platform provider](../../../platform/services/storage/database/main.rs), then
two [clients](client.rs). Each client has one explicit link and four private
pages, with no disk, clock or sibling connection. The provider receives the disk,
64 private pages for its heap and four reader pages for sealed attachments.
Profile link ordering binds provider slots to persistent roots 0 and 1; reordering
those links on an existing image is unsupported. These are private fixture roots,
not a proposal for global object identity.

Run from the repository root:

```powershell
python tools/boot-harness-rs/objects_test.py --release
python tools/boot-harness-rs/objects_test.py
```

The runner formats a new 4 MiB volume, surrounds it with guard bytes and boots
the separate `objects.json` composition. Each root creates an 8 KiB object with
distinct deterministic contents. Clients reconstruct it through revision-pinned
range reads and check stale-write, oversized-request, short-attachment and
retained-quota refusals. They also reject supervisor-only commands, extra root
selectors, overflowing ranges and stale reads. Refusals leave the current head
and bytes unchanged. The next valid write after refusal must still succeed.

After both clients have received success, the runner kills QEMU with the database
still open. The second cold guest verifies the previous contents, replaces one
byte, verifies revision 2, and closes the provider. Its heap allocations and
reader-page charges must return to zero. The independent host layout model checks
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

One early release run hit the existing ATA readiness deadline (`status=c0`, busy)
during database initialization. A subsequent complete two-boot run passed without
changing the driver. The failed artifact is retained alongside successful runs;
the cause remains undiagnosed (`20261017T005941877071Z`). The driver's 50-tick deadline and intermittent host
IO stalls need investigation; a passing rerun does not establish a fix.

This is a bounded, synchronous lab: 16 KiB maximum contents, 4 KiB chunks,
inline contents through 128 bytes, one object per connection root, and a 24 KiB
retained-record quota per root. The 40-byte read response exercises ordinary
copied IPC; it is not a proposed bulk-read transport. There is no public history
subscription, GC, cross-root transaction, encryption, installer or migration from
the ordinary desktop catalog. Quota refusal is distinct from physical full-disk
progress. QEMU termination does not emulate host-cache loss or physical power loss.

The shared publication policy tests run inside `source-rs/`:

```powershell
cargo test --locked -p cathedral-database-objects --features host
```

They check rollback of every record, writer revalidation after competing
preparation, exact quota accounting, corruption detection, range boundaries,
inline/chunk transitions and root isolation. Existing engine/volume fault labs
remain the evidence for simulated write/sync failures; this guest trace does not
exhaust those fault histories.

The dev profile size-optimizes the engine, content libraries and provider, and
uses one redb codegen unit. `debug_assert!` is disabled in redb, database-objects
and the provider; explicit validation/assertions remain enabled. The provider fits
235 image pages in the checked dev build; the kernel's 240-page limit is unchanged.

Verified artifacts: debug `20261017T010736771525Z`, release
`20261017T010906650068Z`. Both cold-boot traces pass with 12 records at revision 1
and 18 at revision 2 across the two roots. Cold recovery's cumulative peak is
81,751 requested bytes; the full second boot peaks at 95,676 bytes. Largest single
allocation is 9,432 bytes, allocation failures are zero, and final live bytes are
zero. Host workspace tests, all 24 storage-layout tests, four publication policy
tests, x86 userspace lint, ARM64 library lint and source-layout checks also pass.
