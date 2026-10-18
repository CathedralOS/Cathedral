# Experimental volume bootstrap

This host lab creates and reopens a Cathedral-owned volume inside an explicitly
assigned range of a raw image. The envelope owns identity, geometry, formatting
state and engine logical length; redb owns its internal format and recovery.
Neither layer is an accepted Cathedral disk ABI. This does not create a GPT,
EFI partition, bootable OS image or installer.

Start at [volume.rs](volume.rs): dispatch commands, validate blank space, initialize
the engine while marked formatting, close/sync it, then publish ready. Opening
validates the envelope and engine identity without a formatting fallback. Descend
into the shared [header codec](../../../../source-rs/platform/libraries/storage-volume/header.rs)
for encoding/selection, [region.rs](region.rs) for checked
range translation, [backend.rs](backend.rs) for file-length adaptation and
[image.rs](image.rs) for exclusive host-image ownership. [proof.rs](proof.rs) sequences
native-image checks, refusal/bounds cases and simulated interruption. `mod.rs` is
wiring only. The proof reuses the existing [fault device](../stress/device.rs).

`compact IMAGE OFFSET BYTES` enters [compact.rs](compact.rs). Before installing
changed programs, the boot harness uses it to reclaim physical engine slack
without removing logical records. It locks the image, saves and syncs a complete
content-named backup, compacts, cold-opens and compares every table's records.
Identity, assigned extent and image size remain fixed. The host lab supports
byte-key/byte-value tables here; other schemas are refused. This is offline
maintenance, not guest collection, volume expansion or reformatting.

Stock program refresh in [labels.rs](labels.rs) also handles logical quota
pressure. It identifies all recognized copies before retiring template history,
stages their replacement heads, then invokes the shared database-object collector
inside the same transaction if retained bytes exceed the root quota. Current
heads and durable pins protect their content, including user data and customized
programs. Unreachable revisions may be reclaimed just as in the running provider.
If live/pinned data still cannot fit, replacements and collection roll back
together. The pre-upgrade backup remains available. Seeding and the later
hierarchy migration remain separate transactions; this is not an atomic installer.

The desktop's explicit stock migration enters [hierarchy.rs](hierarchy.rs):
plan names, media types, indexes and matrix startup configuration, back up the
image, then commit without renumbering objects or replacing saved application
data. `upgrade-hierarchy IMAGE OFFSET BYTES` repeats safely; `inspect-hierarchy`
prints IDs, parents and content manifests for migration audits. The latter opens
the engine and may recover it, so use an image copy when inspecting evidence.
`failure-fixture IMAGE OFFSET BYTES login|notes|launcher|user CONTENT_FILE` enters
[failure.rs](failure.rs). It replaces root Login or the named User 1 program's
code on a disposable one-user fixture, preserving identity, metadata and saved
subtree. `startup-path-fixture IMAGE OFFSET BYTES login|users` consistently
renames that root entry to Unavailable. These are QEMU failure fixtures, never
interactive installer operations.
`verify-unchanged BEFORE AFTER OFFSET BYTES` cold-opens both disposable images,
compares every table/record, volume identity and outside guard bytes. Engine
bookkeeping may differ after boot; any logical record difference is refused.
Like other host engine opens, this can recover/modify engine bookkeeping, so
use evidence copies rather than a live or original user disk.
`configure-fixture IMAGE OFFSET BYTES renamed|missing` is test preparation only,
implemented in [configuration.rs](configuration.rs). It edits a disposable
one-user fixture for the guest path test; it is not an interactive settings tool.

## Commands

From the repository root:

```powershell
python tools/storage-lab/engines/run.py --volume
```

The runner creates a timestamped directory under `build/storage-volume/`, retaining
raw images, the cut matrix, checks and compiler/host/revision/source-hash context.
It also exercises the command interface in separate processes. Existing experiment
directories and images are never overwritten. As with other engine modes, avoid
concurrent builds of different features into the shared executable path.

From `tools/storage-lab/`, individual operations look like:

```powershell
cargo run --locked --release --features engines -- engines volume format new-volume.raw 25165824 0123456789abcdef0123456789abcdef
cargo run --locked --release --features engines -- engines volume inspect new-volume.raw 0 25165824
cargo run --locked --release --features engines -- engines volume open new-volume.raw 0 25165824
```

`format` creates a **new regular image file**, with an explicit size and nonzero
128-bit volume ID in hexadecimal. The example ID is illustrative; the lab does not
generate or enforce globally unique IDs. The internal formatter can initialize an
assigned subregion, but first scans the entire range and refuses any nonzero byte.
It will not reformat an existing volume or an interrupted, nonblank formatting attempt.

`inspect` validates and prints envelope metadata without modifying it. `open`
requires ready state, opens the existing engine and verifies its matching identity
record. It can perform ordinary engine recovery/housekeeping. Missing files are not
created; zero-length or invalid engines are never initialized during open. The redb
custom-backend entry can create at length zero, so the ready-envelope path explicitly
forbids zero length before calling it. The lab table's identity must already exist.

Host images acquire a nonblocking exclusive file lock. A second cooperating owner
is refused. The underlying adapter cannot resize an opened image or write beyond
its extent. Real block devices, partition discovery and guest capability binding
are outside this tool.

## Envelope and bounds

The assigned offset and length must be 4 KiB aligned and fit the device without
overflow. Minimum volume size is 8 KiB of envelope plus 2 MiB of engine capacity;
this admits an empty lab database, not arbitrary payloads. The proof uses a 24 MiB
assigned volume between distinct 4 KiB prefix/suffix sentinels.

| Volume-relative bytes | Owner |
| --- | --- |
| 0..4096 | Envelope copy 0 |
| 4096..8192 | Envelope copy 1 |
| 8192..volume length | Fixed-capacity engine backing |

Each copy is a 4 KiB record with explicit little-endian integers:

| Offset | Field |
| ---: | --- |
| 0 | 16-byte lab magic |
| 16 | Envelope version, u64 (1) |
| 24 | Formatting/ready state, u64 (1/2) |
| 32 | Volume ID, 16 opaque bytes |
| 48 | Generation, u64 |
| 56 | Assigned volume length, u64 |
| 64 | Engine offset, u64 (8192) |
| 72 | Engine capacity, u64 |
| 80 | Engine logical length, u64 |
| 88 | Engine discriminator, u64 (1 = experimental redb adapter) |
| 96..4064 | Reserved zero bytes |
| 4064..4096 | BLAKE3 digest of a domain prefix and preceding bytes |

The digest detects damage; it is not authentication, encryption or protection
against a malicious writer. Geometry, supported fields and bounds are checked
after checksum verification. The two valid copies must agree on identity/geometry;
equal generations must agree completely. Otherwise the newer valid generation wins.
One damaged copy can be ignored. An intact unsupported version/engine is refused
even beside an older supported copy. Both damaged copies require refusal.

An update writes one copy, syncs, writes the other and syncs again. Both copies
contain the same new generation before success is returned. This permits recovery
through an interrupted update and later damage to one completed copy under the
tested fault assumptions. It does not tolerate arbitrary loss of both copies.

## Formatting and engine length

Formatting first durably writes a formatting envelope. It then initializes redb
through the bounded backend and commits an identity record with immediate durability.
The engine is closed while still marked formatting. Only after another successful
sync does the formatter publish ready. Interrupted format may therefore leave a
refused incomplete volume or a valid ready volume, including a lost acknowledgement.
There is no automatic resume, erase or repair command.

redb expects a growable file; the assigned disk region has fixed capacity. Its
logical length is therefore envelope metadata, not the raw image's file length.
On growth, the backend checks capacity, zeros the newly exposed range, syncs and
durably publishes the new length before returning to the engine. On shrink it
syncs prior engine writes and publishes the shorter length without truncating the
physical image. Subsequent regrowth zeroes that range again. Reads/writes must fit
the current logical length, and checked translation keeps them inside the assigned
range. A ready engine cannot be resized to zero.

Length updates add three syncs and two header writes, beyond redb's own durability
work. Whole-range blank checks and growth zeroing are deliberately conservative.
This is a correctness baseline, not a tuned allocation or installation strategy.
Capacity must account for engine reserve and transient allocation: the initial
8 MiB fixture could not admit the 2 MiB payload workload, while the 24 MiB fixture
does. Payload size alone does not predict required engine capacity.

## Validation and limits

The native-image proof formats a guarded subregion, publishes a 2 MiB payload,
reopens it, refuses an oversized write and reopens the prior contents. Prefix,
suffix and total image length remain unchanged. Other checks exercise nonblank
format refusal, creation over an existing path, duplicate ownership, single-copy
recovery, both-copy corruption, unsupported versions, conflicting IDs, bad geometry,
zero logical length, incomplete formatting, bad engine magic, truncation, offset
overflow, out-of-range engine IO and zeroing after shrink/regrowth. Validation
refusal cases assert that the image was not modified.

The simulated device keeps volatile and durable images separately. The cut matrix
replays every mutating device call in three finite traces: format, growing publication,
and replacing a large payload followed by compaction. Each call is cut before,
partway through, and after persistence using the existing deterministic model.
Successful barriers are preserved; device mutations stop before cleanup. Reopening
must refuse unfinished formatting or recover a valid engine. Existing-volume
publications recover complete old-or-new records with matching payload/head/event
digests. Once publication returned success, later interruption during compaction
must preserve the new contents. Every cut checks the neighbouring sentinels.

The 2026-10-16 run (`build/storage-volume/20261016T193832141921Z`) passed 939 cuts:

| Trace | Cuts | Logical length before → after, bytes |
| --- | ---: | ---: |
| Format | 162 | 0 → 1,056,768 |
| Growing publication | 435 | 1,056,768 → 9,445,376 |
| Small replacement and compaction | 342 | 9,445,376 → 28,672 |

These lengths describe the uninterrupted reference traces, recorded in
`volume-layout.csv`. Each uses the same 25,157,632-byte engine capacity; the physical
image remains fixed. Compaction must actually shrink the logical extent for its
trace to pass. The runner also passed separate-process format/inspect/open checks.

These tests cover fixed traces and selected partial-persistence patterns, not all
sector reorderings, failures during recovery, controller behavior or hardware power
loss. They do not establish space/latency guarantees or corruption repair for DBFS.
There is no encryption, secure erase, migration, external blob allocation, installer
partitioning. The host lock does not replace a future
kernel-enforced device capability and single-owner lifecycle.

The [guest block boundary](../../../../source-rs/platform/libraries/storage-volume/CHARTER.md)
now shares this envelope codec and exercises bounded sector IO, explicit flush,
cold reboot and corruption refusal on a host-formatted volume. Run it with
`python tools/boot-harness-rs/volume_test.py --release`. Host redb still owns engine
initialization and final engine reopen; the guest inspects metadata and transports
opaque bytes. The separate [guest engine proof](../../../../source-rs/distribution/lab/database/README.md)
now supplies a funded allocator and performs actual guest commits and unclean recovery.
Whether this envelope or engine becomes permanent remains an engineering/design
decision; current owner questions about identity, retention and isolation still apply.

The private-content runner uses `verify-objects` or `verify-retention` to compare
the guest's entire private record set against the host representation model.
`verify-desktop`, `verify-colors` and `verify-notes` independently build the
desktop apps' objects and check their chunks, manifests, heads, history and
quota, naming any record that differs. These commands take a copied image, byte
offset and volume length; the object proofs add the fixture revision and the
color and notes oracles add each root's final revision.
See [objects.rs](objects.rs), [desktop.rs](desktop.rs), [colors.rs](colors.rs)
and [notes.rs](notes.rs). Host verification never
opens the original image before the next cold guest, so host cleanup cannot hide
guest recovery problems. These are fixed fixture oracles, not general fsck tools.

The stock desktop invokes `upgrade-app-close IMAGE OFFSET BYTES LEGACY_LIST`
after seeding. [labels.rs](labels.rs) matches old copied stock programs against
retained template content or the distribution's explicit legacy digests,
and refreshes their launcher and Notes/Colors. The earlier `upgrade-labels`
mode only refreshes the launcher and appends labels to unchanged app code.
It backs up the locked image and publishes all user-copy changes atomically;
it does not rewrite saved notes/colors, tree edges or custom programs. Start at
the [distribution migration](../../../../source-rs/distribution/migrations/README.md)
for policy, provenance and compatibility replay instructions.
