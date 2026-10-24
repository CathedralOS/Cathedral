# Rust implementation map

Notes and Colors now acquire interaction/storage through a
[scoped startup broker](platform/contracts/services/README.md). Sealed imports
select the broker format; existing programs retain legacy startup. Both the stock
and independent matrix use platform acquisition/custody, without a kernel locator.
Required imports still gate launch, optional ones may be absent, and ordinary
operations go directly over the acquired connections. Provider startup remains
eager; arbitrary discovery and dynamic grant additions are not implemented.

This map connects implemented mechanisms to their integrated consumers, proofs
and remaining restrictions. The [lab guide](README.md) owns detailed ABI/build
instructions. [Owner questions](../OWNER_QUESTIONS.md) own unresolved semantics.

The [matrix input entrance](platform/libraries/matrix/world/input/input.rs) now
separates four bounded channels from their focus groups. F5 supplies keyboard
and pointer channels in one group; nested matrices preserve group association
and event order. Pointer capture and app button edges remain channel-local.
The owner control connection can register and retire channels/groups; table slots
are reclaimed after explicit retirement events drain. Local and inherited names
are separate. Explicit child producer grants now reserve channel quotas, isolate
group control and retire on disconnect. Nested stock hosts now negotiate incoming
channel/group ceilings before reserving additional local producers; retirement
restores upstream capacity. Explicit manifest/owner offers now permit separate
producer endpoint acquisition, with interrupted handoff and disconnect cleanup.
Ordinary app offers remain unchanged. The installed [Split matrix](distribution/applications/split/README.md)
now explicitly admits a producer through AUTO, with independent focus and a scoped
cursor. STOP or view invalidation ends the run; restarting is explicit.
Disposable receiver/producer variants verify held-input retirement, producer loss,
per-channel overflow isolation, fresh input afterward and replacement identities
through that installed IPC path, including cold persisted edits.

The [surface path](platform/libraries/matrix/world/surface/README.md#bounded-cursor-presentation)
now supports a bounded transparent arrow through ordinary local surface authority,
recursive clipping, stacking and atomic publication. It needs no full-screen
buffer. Split uses it for its synthetic participant. General cursor preferences,
suppression and authenticated attribution remain unimplemented.

## Default desktop

F5 and `python tools/boot-harness-rs/run.py --release --window` share
[one composition](distribution/profile.json). It boots to a chooser over the
folder of users: NEW copies the template user as a subtree copy, and choosing a
user runs it as a matrix whose chrome, a second chooser over the user's realm,
launches that user's installed apps. User entry revokes the boot
chooser's display/input grant and gives the user the full screen: a bottom
taskbar, active app in the remaining space, no persistent boot help or logout control.
The launch protocol can withhold storage for future launches: Notes receives an
actual optional binding and shows TEMP NO SAVE; Colors refuses missing required
storage. The ordinary launcher has no storage toggle. [Declared imports](platform/contracts/imports/README.md) are checked
before store-based admission. Notes keeps up to eight
notes as private objects on the experimental database disk; a duplicate is the
store's copy of the saved note and shares its content. App-owned X offers
Save / Discard / Cancel for a dirty note; a sibling's ordinary CLOSE requires
a clean loss reply. The launcher's separately granted END APP control can stop
an unresponsive app after a loss warning, using a ticket bound to that exact
task. Default Cancel preserves work; stale or revoked tickets deny. The
[management fixture](../tools/boot-harness-rs/management_test.py) checks this
through real input with a spinning app and a restarted launcher. This is not a
protected global escape from a failed desktop. Colors keeps its color. Every
program is an object in the store, launched from a realm by whoever can see it
there and granted its object's subtree, so its state nests under its code and
a user's state nests under the user. [The distribution map](distribution/README.md)
links admission, choosers, apps and provider boundaries.

An exhausted launcher retry can show the owner's boot-supplied recovery program
in that strip. Try Again requests one attempt through the fallback's current
connection, preserving the app and its arrangement. This fallback has no storage,
layout, management or child capacity. Its own failure does not request another
retry. The [launcher recovery fixture](../tools/boot-harness-rs/launcher_recovery_test.py)
exercises pointer/keyboard retries, repeated failure, live drafts and cold persistence.

Live storage replacement polls readiness without blocking the startup owner;
fresh grants follow accepted readiness. The
[F5 storage fault fixture](../tools/boot-harness-rs/provider_live_test.py) keeps
Notes alive through precommit and lost-reply failures, exhausted restart budget
and replacement readiness timeout. It verifies continued editing, failed-save
close protection, reconciliation and exact cold-open contents. An exhausted
live session still lacks an explicit storage-retry control.

`python tools/boot-harness-rs/desktop_test.py` checks this actual image through
physical input, a copied user, exact saved text/status, duplicate and switch,
repeated app replacement and cold reboot, then has an independent host model
verify the retained records. Older profiles remain focused fault/mechanism
fixtures. This is still one chooser over a folder and up to three app instances per user, not general app installation or a full window manager.

## Interface ownership

The stock matrix executable now belongs to `distribution/matrix/`, including
its saved desktop arrangement, launcher recovery and five-child control router.
Its extension protocol and configuration codec live in its own `contracts/`.
`platform/libraries/matrix/` contains the shared authority state with a caller-
selected child count and owner-supplied rectangles. Three-child visual and
four-child headless tests import no distribution code. The stock IPC loop still
uses a bounded five-child composition; this is not yet a general reusable server runtime.
An [independent three-peer matrix](distribution/lab/matrix-composition/README.md)
now verifies local input, revocation and replacement over real IPC. It uses the
existing runtime server and the same local command dispatch as F5, with no stock
contract or implementation dependency. It owns three child admissions through
shared runtime channel construction/spawn/rollback, and tests failure/replacement
and subtree reclamation. Rendering and upstream forwarding remain outside that
proof; it does not supply a complete alternative desktop or matrix runtime.

The universal Rust contracts crate has been retired. [Kernel contracts](kernel/contracts/README.md)
define the public syscall/page/framebuffer boundary; [platform contracts](platform/contracts/README.md)
define service messages without importing their providers. Display requests and
kernel framebuffer information now have separate owners. The core owns its internal
[boot memory inventory](kernel/core/memory/inventory.rs), while the platform
[block-device library](platform/libraries/block-device/README.md) owns the storage
adapter trait. Distribution-specific session messages remain in distribution.

The layout checker allows userspace to import the public kernel contract package,
rejects kernel implementation imports, prevents contracts from importing providers,
and keeps platform protocols out of kernel dependencies. This changes source/package
ownership, not syscall numbers, packet encodings, grants or on-disk formats.

## DBFS storage-layout investigation

The [foundation proposal](../wiki/proposals/0002_dbfs_storage_foundation.md) now
distinguishes object identity, immutable content and physical placement, with
established database-engine families as candidates. Realm boundaries do not
automatically require two-phase commit; durable watchers do not require CRDTs.
Identity/alias, deduplication/erasure, retention and isolation choices remain in
the owner queue. No production engine or on-disk format is selected.

The separate [host layout lab](../tools/storage-lab/README.md) compares flat
manifests with fanout-32/128 copy-on-write chunk-reference trees at 4/16/64 KiB.
Its 270 cases cover inline tiny contents, range edits/reads, copies, shifted
contents, batched publication and retained versions. The runner checks payload
and final-state equivalence across layouts. Thirteen tests exercise boundaries,
shared paths, batching and snapshot retention. At 4 KiB, an aligned small edit
creates 4 KiB of payload and 81,937 bytes of flat metadata versus 2,212 bytes of
tree-32 metadata. Cold range reads trade metadata bytes against dependent records.

The representation lab measures packed bytes and hashing work. Its separate
[engine comparison](../tools/storage-lab/engines/README.md) now runs 48 cells against
redb and SQLite DELETE/WAL with identical records, ordered scans and verified
reopen. Nine checks cover process death before/after commit and capacity refusal.
It measures logical file extents and redb backend writes/syncs, not physical media
traffic or allocated disk blocks. At 4 KiB, batching 64 adjacent tree-32 edits
reduces redb writes from 3,960,832 to 65,856 bytes and syncs from 64 to one.

redb 4.4.0's experimental `no_std` backend entrance compiles for x86-64 and ARM64
bare-metal targets. A bounded guest adapter now exists; no production engine is selected.
The [redb stress lab](../tools/storage-lab/engines/stress/README.md) now checks
competing expected-revision writers and whole-batch rejection, while preparation
occurs before the writer transaction. Two bounded-queue workloads measure small
write latency and writer occupancy beside a bulk importer and active readers.
The atomic bulk transaction has a different latency/commit tradeoff from separate
publications; this is not an automatic transaction-splitting policy.

All 150 simulated crash cuts recover complete old-or-new records with valid
content/event references; four acknowledged commits survive immediate loss of
volatile state. Releasing an old reader permits 289 further replacements under
an unchanged 16,846,848-byte cap, followed by capacity refusal, reboot and a
successful normal replacement under that same cap. The no-reader/pinned-reader
controls reach 2,109,440/16,846,848-byte logical extents. These are bounded host
experiments, not exhaustive device-failure, physical-allocation or general GC
proofs. The [admission lab](../tools/storage-lab/engines/stress/admission/README.md)
now reserves bytes/counts before preparation and holds them through commit, rejects
oversized atomic transactions, and alternates ready producers. Three trials complete
384 publications with all charge returned; eight tests cover budget boundaries,
dispatch and release on failure. The logical charge peak is 232,192 bytes across
11 requests. This is not a total-memory bound, priority contract or latency guarantee.
Controlled arrival/window experiments, longer fault histories, recovery failures and
metadata queries remain next evidence. The guest private catalog is unchanged.

The [volume bootstrap lab](../tools/storage-lab/engines/volume/README.md) now wraps
redb in an explicit fixed-capacity raw-image region. Two checksummed envelope copies
record volume ID, version, geometry, formatting state and logical engine length.
Formatting requires blank assigned space; ready is published only after durable
engine initialization. Opening refuses unsupported, unfinished or invalid volumes
without creating a replacement database. Native-image checks cover exclusive ownership,
capacity refusal, reopen and preservation of neighbouring ranges; 501 simulated cuts
exercise formatting, publication and compaction with logical-length changes. The
engine owns the whole assigned region from format: a length change publishes the
envelope and nothing else, so no range is zeroed at runtime. This adds no GPT/EFI,
installer or accepted disk ABI.

The [shared volume boundary](platform/libraries/storage-volume/CHARTER.md) now has
an allocator-free byte/sector adapter and the host/guest envelope codec. It validates
complete IO ranges, preserves partial-sector neighbours and stops IO after uncertain
write/flush failures. The separate guest fixture uses the granted ATA controller to
inspect a host-formatted redb volume and flush scratch writes. Four boots check cold
reopen, single-copy recovery and both-copy refusal; host checks compare the entire
image and reopen the intact engine. This transport proof remains separate from the
[new database runtime proof](distribution/lab/database/README.md).

The runtime proof installs a 256 KiB heap from an explicit private-page grant,
checks exhaustion and budget reclamation after leaked allocations and a contained
fault, then commits three related records through no_std redb and userspace ATA.
QEMU is killed after an acknowledged commit with the database still open. A cold
guest recovers it and commits revision 2; independent host opens of image copies
check both revisions without cleaning the guest's image first. Guards outside the
assigned region remain unchanged. This does not simulate loss of host disk caches.

Shared admission now permits 240 image pages inside the same 1 MiB image window,
a guarded 32 KiB user stack, 288 admission frames and 128 private/reader pages.
Fallible per-process/per-object ledgers replace the large fixed arrays and u128
write mask. The stack increase was required by unclean database recovery. The
kernel still has no engine dependency; the ordinary catalog format is unchanged.
The engine and worker are size-optimized with debug_assert! disabled even in dev;
explicit fixture assertions remain enabled. See the proof for commands and scope.

## Private content through ordinary IPC

The [content publication proof](distribution/lab/objects/README.md) now runs a
separate platform database provider and two ordinary clients with disjoint roots.
It persists real BLAKE3 chunks/manifests, current revision, change record and quota
ledger atomically. The host and guest share the experimental content codec; the
host model independently checks the complete retained record set. Clients have
no disk grant and cannot select roots or look up arbitrary content hashes.

Cold guest recovery reconstructs acknowledged content before replacing one byte.
Stale, oversized, malformed and retained-quota refusals leave no partial records.
A 224 KiB provider heap, eight output pages and four input reader pages bound this
fixture; serial samples report recovery, preparation and commit allocations.
Sealed bulk reads avoid reconstructing data through 40-byte copied replies. Reader
exhaustion returns BUSY and released readers permit reuse. Backings remain funded
by the provider until their last reader releases them; general client fairness and
funding transfer are still open engineering work.

Two durable root-owned pins and bounded collection now preserve selected revisions
and reclaim retired content. Each pass validates at most three live closures,
scans the 24 KiB root record budget, removes at most eight records and adjusts its
ledger atomically. Host tests reopen every pass and complete 600 8 KiB replacements
in a fixed roughly 1 MiB engine extent containing unrelated data. Three guest boots
prove pin survival, release/collection, bulk pressure and cold recovery. This is
bounded reuse evidence, not a physical full-volume progress or watcher guarantee.

The [object store](platform/libraries/database-objects/README.md) holds many
numbered objects per root, arranged as a tree. Chunks and manifests are shared
within a root, so a copy writes a head, an event and an edge and nothing else,
whatever the content's size, and a subtree copy does that for every object
beneath the source; numbers are never reused. Automatic collection runs before
mutations under quota pressure; explicit collection is unconditional. It reclaims
what no head or pin reaches, reading only metadata and a key-only chunk index.
Objects hold up to 256 KiB and a root 2 MiB, enough for the
desktop's programs to live in the store as objects. The [provider](platform/services/storage/main.rs)
speaks one [object protocol](platform/contracts/storage.rs): 64-byte packets with
sealed attachments for publications and bulk reads. The legacy sector catalog and
its protocol remain only for the status/counter composition.

The matrix forwards that protocol through every level, attachments included, and
the desktop's apps use it directly: Notes publishes whole documents in its
realm, reads them back as sealed objects and duplicates them through the store's
copy; the color probe keeps its one object. Two cold boots and an independent
host oracle verify saved colors, saved notes and exact records, including one
shared manifest behind a duplicated note. The desktop database path requires release mode under TCG.
Apps preserve confirmed state on failed saves, show an unconfirmed-save mark and
reconcile via readback before another mutation. A real startup publish timeout
and provider cuts before/after commit are exercised with exact revision checks and
a live nested sibling. The distribution replaces dead or stalled storage providers,
including retired root streams, and explicitly restores grants; a grant after
replacement re-establishes the root's provider connection through the link
itself. The budget allows at most three replacements per desktop session;
exhaustion or failed reopen leaves storage offline while the apps remain alive.
Retired app transports require app replacement, and bad media still fails closed.

The database desktop is an ordinary CLI/F5 option with its own persistent disk,
created only if absent. Opt-in storage traces separate app/matrix round trips,
dispatch/provider queueing, collection, commit and ATA operation/wait counters.
Their clocks and fault-injection workloads do not establish production latency
bounds. Slow-build latency and production restart policy remain engineering work.

The [notes app](distribution/applications/notes/README.md) exercises real physical
text input, cursor edits, whole-document publication, draft-preserving
reconciliation, duplicate/switch and exact cold-restored glyphs. It is an
ordinary replaceable app. Its 240-character capacity is the editor's choice, not
a storage bound; the matrix's reader budgets are sized for the sealed bitmaps it
retains at each level, with old and new held during replacement.

One early ATA busy timeout remains undiagnosed. The driver now records opcode,
phase, LBA, status, ticks and polls for slow/failed waits; optional QEMU tracing
records device/block events. Instrumented runs passed without reproducing it.

## Resumable storage and provider recovery

Child storage requests now return to the matrix event loop while awaiting replies.
The root's separate provider links let physical input and sibling rendering continue
through a blocked save. Nested matrices in the visible desktop now receive a separate catalog pair in
[their named startup bundle](platform/libraries/user-runtime/matrix/matrix.rs). A gated
save leaves a sibling inside that same matrix able to consume pointer/keyboard
input and repaint. The legacy headless fixtures retain their shared-stream mode.
Owner grant/revoke and teardown aborts remain synchronous bounded operations.
Follow [storage forwarding](distribution/matrix/service/parent/storage/storage.rs),
[pending replies](distribution/matrix/service/parent/storage/pending.rs) and
[deferred stream policy](distribution/matrix/service/deferred.rs).

The [desktop stall proof](distribution/desktop/lab/stalls.rs) gates writes
before execution and after durable commit. Its [nested peer](distribution/desktop/lab/stalls_peer.rs)
checks input and repaint while blocked, plus survival across catalog revocation,
provider replacement and writer death. It also checks
normal completion, timeout followed by a late reply, refusal to recycle an accepted
live link, and provider replacement with explicit fresh grants. Unacknowledged
writes are reconciled by readback, never automatically retried. Root storage
transport loss revokes the affected binding without killing display/input.
Separate parent catalog transport loss retires that connection and its grants;
legacy shared-stream failures still retire the nested matrix.

[Provider recovery](distribution/desktop/lab/providers.rs) kills the physical
input decoder and display provider. Input loss cancels focus while apps survive;
display loss retires the root. Distribution policy explicitly tears down/rebuilds
adapters and the tree, re-funds the root from reclaimed reservations, then restores
committed app colors. This is a lab restart policy, not a trusted recovery UI.

The current arena has fifteen task accounts, 96 dynamic channels, 92 reservable
channels per account, 48 dynamic handles and a 32-handle wait bound. Account
masks remain u16; explicit owner reservations bound recursive admission.
The desktop uses five local slots (chrome, three apps, background), with resource
ceilings requested by newer program declarations. Kernel heaps are 512 KiB.
Host tests cover high-account isolation, startup bundles, exhaustion and reclamation.
The traced debug matrix grew to 67 loaded pages. Admission limits have since
increased for the database proof above, preserving the 1 MiB image window,
and physical funding checks. The database milestone replaced the u128 write mask
with per-backing permission records. ELF tests cover the exact
rounded-page boundary. These remain experimental lab ABI limits. The debug smoke also caught boot-lab
stack pressure from retaining supervision fixture locals across later suites;
the supervision and IPC entrances now sequence their suites after each fixture
unwinds, keeping the existing kernel stack bound.

Validation: workspace host tests, profile budget tests, x86_64/ARM64 target lint,
x86 debug smoke, ARM64 debug portability, traced debug desktop and release desktop
with storage faults/provider rebuilds/cold reboot, combined nine-process pressure
and the headless storage regression. The pressure run includes all recovery checks;
its destructive startup took about 75 seconds, while ordinary window boots skip
those probes.

## Responsive publication and persistent desktop

The visible matrix session now gives each app a private delegated catalog. Apps
save color before presenting, reload after replacement, and recover both colors
in a fresh QEMU process. Ordinary window boots keep
`build/boot-harness-rs/matrix-desktop-storage.raw`, separate from the status demo.
`--matrix-desktop-test` uses a disposable disk, adds staged rendering/storage death,
revoked/stale writes and focus-loss checks, then cold-boots the same disk. Its first
boot log is retained as `first-boot-serial.log` beside the reboot's `serial.log`.
Follow [desktop recovery](distribution/desktop/lab/recovery.rs) and
[app state](distribution/applications/colors/session/state.rs).

[Publication](distribution/matrix/service/parent/presentation.rs) now resumes
from the service loop after upstream replies. Other children's local reads can
proceed; dependent mutations retain one event per stream with backpressure, keeping
geometry stable until commit. FIFO is preserved within each stream. This does not
yet make owner storage operations, placement, creation or all parent refresh operations asynchronous.
A compact `PRESENT_RECTS` packet carries one or two existing rectangle updates in
one atomic request per matrix hop; images and larger batches keep BEGIN/stage/COMMIT.
Malformed compact updates publish nothing, and successful commits share retry history.

Earlier diskless pressure measurements fell from 710 ms guest key-to-repaint to
130-280 ms after compact presentation; host input-to-scanout was 391-563 ms versus
766 ms. The combined storage/pressure fixture has additional work and is not a
like-for-like comparison (traced runs measured 687-766 ms host / 180-270 ms guest).
These are diagnostic observations, not latency guarantees. Equal-turn scheduling
and provider-funded service work remain; no priority or CPU-donation policy changed.

## Atomic matrix presentation

The recursive matrix desktop also has bounded atomic presentation transactions:
four distinct retained surfaces, staged sealed readers, all-or-nothing publication
and latest-commit retry scoped to connection/view. Host policy tests and the
release desktop fixture under CPU pressure verify rejection, old pixels before
commit, bitmap forwarding, input and subtree recovery. See the
[matrix charter](distribution/matrix/CHARTER.md) for the transaction path and
limits, including non-transactional owner placement and no vblank guarantee.

Debug integration also checks the fixed userspace stack/image bounds. Persistent
matrix batches and compositor staging live in process data storage; constant
input-queue initialization avoids large nested debug constructor temporaries.
The debug desktop and matrix-storage fixtures pass without enlarging kernel limits.

## Recursive storage delegation

[Matrix storage scopes](platform/libraries/matrix/world/storage/README.md) delegate
private object roots, or subtrees of them, through the ordinary matrix
executable, which becomes the storage root by being handed provider links rather
than by an argument. A realm is that attenuated view; a matrix is a program that
attenuates. Every hop checks connection identity and a local binding revision
and stamps the binding's scope on the request; the provider enforces every
stamp, so a child sees its realm and nothing beside it. A full scope packet is
folded only after an upstream containment proof, with the caller revision checked
again before forwarding. Immutable parents and never-reused IDs are required.
One absolute deadline bounds forwarding and proofs; depth does not add time.
Siblings have separate
numbers and revisions. Every operation is atomic at the provider, so revocation
leaves nothing staged, and regranting an ancestor does not restore descendants
automatically. The headless `--matrix-storage-test --release` fixture checks
these boundaries, copies through two hops, attenuated realms, a subtree copy
and committed objects after restarting the matrix tree and storage provider.
Read-only rights and per-subtree quotas remain future work; the visible desktop
consumes these grants. A matrix also runs programs from its realm: `RUN_OBJECT`
reads an object on the matrix's own binding, admits it from the sealed reply,
and grants the child that object's subtree, whose root the provider keeps
read-only to its holder. A child's `LAUNCH` does the same through the child's
own binding into a slot the owner opened with a standing policy, and `CLOSE`
stops that sibling only after it answers `LOSS` with nothing to lose. A matrix
answers by surveying all its direct children recursively, retaining the subtree
when any answer is dirty, busy, missing or failed. The headless matrix proof
closes and reopens whole user matrices and also exercises delayed
replies and exhausted control-probe capacity without destroying the slow child.
LOSS is advisory; it does not freeze future edits across processes.

## Userspace register and interrupt grants

The [device boundary](kernel/core/user_tasks/devices/README.md) now supplies
caller-local byte-register capabilities, masked IRQ notifications and revocation
on release, exit, fault or cancellation. A userspace UART driver on COM2/IRQ3
exercises loopback, register/name rejection and replacement after dirty faults
and blocked waits. COM1 remains privileged diagnostics. The PC adapter filters
stale PIC edges after reset; protocol configuration and byte handling run in
userspace. Every session checks exact heap/frame reclamation.

This first device uses mediated port operations, one exclusive grant per child
and boot-lab composition. It does not add discovery, direct MMIO, DMA/IOMMU,
shared interrupts or device-capability transfer. ARM rejects the PC-only grant;
existing PS/2 and ATA adapters remain deliberately restricted.

The runtime now offers `event::wait` for a bounded mix of IPC receivers and device
IRQ notifications plus a deadline. Release smoke proves real IRQ readiness,
non-consuming observation, full-set validation, timer-only sleep, idle driver
shutdown through IPC and cancellation/restart with exact reclamation. Both UEFI
targets and userspace pass Clippy; the host readiness/device policy tests pass.

Validation: host policies and UART drain tests, both UEFI Clippy targets including
smoke code, x86 debug/release smoke, ARM64 release boot, and the release session
focus/drag/recovery regression pass. The full debug smoke used `--timeout 90`.

## Kernel image protection

Both ISAs now enforce RX kernel code, RO/NX immutable data and RW/NX mutable
storage after activating owned tables. Firmware code mappings lose execution
permission. Physical aliases obey image protection too; heap/stack backing is
NX even through the identity map. The [protection guide](kernel/arch/memory/README.md)
connects the UEFI section adapter, shared permission description, ISA encoding,
fatal probes and remaining bootstrap/privileged exceptions.

Seven separate release fault boots per ISA check text/data writes, heap/stack
execution through both virtual and physical mappings, and the stack guard.
Host tests cover invalid section layouts and duplicate physical aliases inside
large x86 mappings; standalone PE parser tests reject malformed loaded-image
headers. Full x86/ARM debug and release boot suites and the release matrix
desktop regression pass with the protected image. This does not isolate trusted kernel code from page tables
or remove writable kernel backing aliases used to load user programs.

## Transferable sealed memory

[Shared objects](kernel/core/user_tasks/memory/README.md) now travel over ordinary
IPC with caller-local read capabilities and independent optional mappings.
Forwarding copies no backing bytes. Queued rights consume recipient capacity;
retained rights pin their original sponsor until the final release, even after
producer death. Both CPUs exercise a three-process chain, unmapped forwarding,
checked copyout, read-only faults and complete frame/heap reclamation.

The matrix desktop submits a bitmap once, retains its read right through both
matrix levels, and maps only in the compositor. It clips and replays image metadata
using the original stride. Replacement releases ancestor references before
returning child funding. Host policy tests and normal/pressure desktop pixel tests
cover clipping, replay, rejected updates and repeated subtree replacement.
This is bounded immutable RGBX storage; writable streams, sponsorship transfer,
texture decoding and GPU import remain open. Bounded multi-surface commits are implemented.

## ARM64 portability proof

The [ARM64 backend](kernel/arch/isa/aarch64/README.md) boots through the same UEFI
entry on QEMU virt. Debug and release tests reuse core admission, scheduling,
IPC, supervision, private/shared pages and reclamation. They exercise timer
preemption, SIMD preservation, forbidden EL0 accesses, producer failure with a
live reader, and every admission frame-failure boundary.

```text
python tools/boot-harness-rs/arm64/run.py --release
```

This is a headless single-CPU board proof. The graphical distribution, PC drivers,
and full CPU-budget stress suite remain x86-64. Trusted kernel threads now run
the same core lifecycle on both ISAs, including guarded stacks, preemption,
dynamic spawn and frame/heap-failure rollback.
The backend guide records fixed mappings and remaining hardening work. The
ARM `--kernel-only` profile also runs the trusted-task tests without a user ELF.

## Shared core and machine boundary

The [address-space owner](kernel/core/user_tasks/address_space/address_space.rs)
now owns image loading, checked user copies, frame accounting, rollback and
borrowed-region bookkeeping for both CPUs. Architecture backends implement the
translation operations and retain their own saved registers and assembly.
[The machine boundary guide](kernel/arch/README.md) describes their ownership
contract and reading paths.

CPU mechanisms now live under `kernel/arch/isa`, selected by `isa/isa.rs`;
`arch/lib.rs` explicitly lists the shared interface. Register-preservation probes
live in the boot lab, and x86 context state is separate from dispatch.

Privileged controller details live under `kernel/arch/board`: PC ATA/PS2 command
validation and PIC/PIT setup/acknowledgement, plus virt PL011/GICv2 and fixed
physical windows. The board supplies the diagnostic UART location and QEMU exit
mechanism; ARM paging encodes the board-provided windows.
Core still enforces grants, validates buffers and owns blocking. Device absence
is explicit at composition; ARM no longer supplies dummy PC controller methods.
Shared event/frame vocabulary uses static backend selection and direct calls.

This pass passed host tests (including device command restrictions), both UEFI
Clippy targets (including smoke code), x86-64 debug/release smoke, ARM64
debug/release proofs, and the matrix desktop's physical input/recovery checks.

The ARM kernel-task bring-up caught excessive debug stack use in the lab's heap
exhaustion cleanup. Borrowing its bookkeeping array avoids by-value iterator
copies; both backends retain guarded 64-KiB task stacks. Debug ARM now completes
the same lifecycle/rollback suite before its user-process proof.

## Current integrated path

The default [distribution policy](distribution/desktop/session/shell/shell.rs)
starts display/input/storage providers and funds a root matrix. Root slot 1 is the
program menu; root slot 0 is a workspace matrix containing the selected app.
Both matrices run the same [service](distribution/matrix/service/service.rs).
Only the root adapts physical input and compositor output. The menu requests
choices; the distribution owner supplies launch, storage and placement policy.

```powershell
python tools/boot-harness-rs/run.py --release --window
python tools/boot-harness-rs/desktop_test.py
```

F5 runs that first command. Choose Notes or Colors, then click the workspace to
focus it. Notes uses F1 to save/check and F2 to reload; Colors changes on Enter.
One workspace app runs at a time. The ordinary dirty-note guard prevents switching
or closing until explicitly saved or reloaded. The desktop test checks that flow,
repeated replacement and exact saved text after cold reboot.

The older `matrix-desktop.json` remains the two-color-app fixture for recursive
surface/input, replacement and resource probes; `--matrix-desktop-test --release`
selects it. `session.json` retains richer compositor/input recovery cases.
These profiles are engineering proofs, not the default OS boot.
[The Rust handoff](../TASKS_RUST.md) records current priorities and evidence.

## Mechanisms, consumers and evidence

| Mechanism / owner | Integrated consumer | Proof | Remaining restriction |
| --- | --- | --- | --- |
| [Address spaces, admission, faults](kernel/core/user_tasks/user_tasks.rs) | All separately compiled services/apps | `--smoke`: rollback, faults, stale tasks, frame/heap reclamation | One CPU (x86-64 or ARM64 proof); bounded executable catalogue and twelve-account arena |
| [Dynamic IPC and startup bundles](kernel/core/ipc/ipc.rs) | Matrix-child, root-display and physical-source connections | `--ipc-test`: moves, rejected bundles, death, quotas | Small copied messages; accepted moves can outlive their creator |
| [Backed IPC reservations](kernel/core/ipc/budget/budget.rs) | Every initial/launch account, including matrix children | Host tests fill all 32 slots; `--ipc-test` exhausts restarted creators while a sibling fills its reservation | Owned idle accounts support conserving subdivision; no sponsorship transfer |
| [CPU turns and measured ceilings](kernel/core/scheduler/README.md) | Every user launch subtree, including nested matrices | Host accounting tests; `--smoke` checks capped spinners, yields, restart debt and idle refill | Boot/owned-idle burst/refill caps; periodic-timer overrun; no deadlines, weights or service-call donation |
| [Physical-memory funding](kernel/core/user_tasks/resources/physical/README.md) | Every admitted image, stack, page table and runtime buffer; recursive matrix pool | Host conservation/bank tests; smoke rollback and retained readers; admission exhaustion and desktop pixel checks | Eager bounded bank; idle-child assignment; no swap, active rebudgeting or sponsorship transfer |
| [Recursive launch loans](kernel/core/user_tasks/lifecycle/supervision/delegation.rs) | Matrices own direct children, lend budgets further, reclaim subtrees | `--ipc-test` plus matrix replacement tests | Static launch loans plus executable-authorized process pools; fixed account arena |
| [Executable-authorized admission](kernel/core/user_tasks/admission/pool/README.md) | Apps and the matrix desktop construct descendants from code authority and a process pool; the shell admits code from sealed objects read out of the store | `--admission-test`: recursive launch, exhaustion, load rollback, retained funding, sibling survival; `--matrix-storage-test` and `desktop_test.py` admit programs from the realm | Boot catalogue or a sealed object, admitted once per reservation; twelve accounts; device grants remain static |
| [Matrix lifecycle](distribution/matrix/service/children/children.rs) | Same provider above apps and another matrix | `--matrix-test`: replacement preserves an outside sibling | Stock desktop has five direct slots; headless fixture uses two; distribution owns policy |
| [Local viewport, focus and capture](platform/libraries/matrix/world/world.rs) | Nested and sibling visible apps | Host policy tests; `--matrix-test`; `--matrix-desktop-test` physical focus | Four channels/groups, bounded physical keys and per-stream queues; tagged asynchronous upstream polling; owner-only registration; restricted producer service pending |
| [Owned surfaces](platform/libraries/matrix/world/surface/README.md) | App rectangles cross actual matrix boundaries to the existing compositor | Host quota/name/clipping/stacking tests; scanout removal and teardown checks | Sixteen surfaces; bounded immutable RGBX images; no text surfaces or vblank guarantees |
| [Retained compositor and hardware input](platform/services/display/compositor/compositor.rs) | Root adapter uses existing scanout/cursor and shared input decoders | `--matrix-desktop-test`; `--session-test --pressure` | Existing richer scenes/hosting remain in the earlier session fixture |
| [Private/shared page lifetime](kernel/core/user_tasks/memory/README.md) | Matrix bitmaps and existing rendering/storage labs | Host transactions; both CPU boot proofs; matrix desktop | Four shared-region slots per task, grants capped at 64 pages; sealed objects only; original sponsor retained |
| [Object storage and recovery](platform/services/storage/service.rs) | Notes and the probe desktop's colors through delegated roots | Store host tests; `objects_test.py`; `desktop_test.py`; `matrix_database_test.py` with and without faults | Two provider roots, whole-root delegation; not the production filesystem |

## Kernel boundary audit and next steps

The discoverability pass reviewed all 27 Rust package trees and preserves the
[entry routes](../wiki/architecture/discoverability.md): boot composition, task
lifetime, CPU transitions, service event ordering, scene publication and lab
scenario ownership. Contracts, small drivers, application workflows and the
matrix/input service boundaries remain cohesive without extra forwarding layers.

Subsystem entrances live in named files inside their folders (`memory/memory.rs`),
alongside the responsibilities they introduce. `mod.rs` contains only disposable
Rust plumbing; the named reading path is intended to survive the Omega port.
The next reading level owns checked IPC delivery,
page sharing/retirement, scene geometry, pointer queues and forwarded matrix
control requests. Cohesive leaves remain sibling files; the guide treats this as
a reading-path preference rather than a file-count or line-count rule.

The matrix protocol exposes local state only. That is narrower than a claim of
whole-system noninterference. The handle-locality pass now covers task, clock, link and page rights as well as
IPC. Host tests compare names and mapping addresses under unrelated graph/arena
activity. The remaining boundary work is recorded below:

| Surface | Current finding | Next implementation boundary |
| --- | --- | --- |
| IPC handles | [Grant table](kernel/core/ipc/grants.rs) supplies local, non-reused names | Retain this model when generalizing other capabilities |
| Task/launch/peer handles | [Caller-local names](kernel/core/names.rs); [admission reservations](kernel/core/user_tasks/lifecycle/grants.rs) and reserved fresh loan-return names | Keep this boundary when replacing fixed executable/slot admission |
| Clock and static links | [Clock](kernel/core/user_tasks/time.rs) and [link](kernel/core/ipc/link.rs) names refresh on recipient admission; unchanged peers retain their names | Separate timing authority from waiting/deadline mechanics; hardware timing remains visible |
| Page handles | [Region grants and virtual slots](kernel/core/user_tasks/memory/regions/regions.rs) are caller-local; producer-local offers require the receiving link and install a separate reader name | Ordinary IPC now copies unmapped read capabilities; writable sharing remains separate |
| IPC capacity | Backed launch-account reservations; surviving channels stay charged across creator replacement; caller-local budget query | Conserving owner/child subdivision implemented; explicit funding transfer remains; retained endpoints pin their original account |
| CPU | Measured runtime and deferred work charge persistent ancestor accounts; exhausted branches wait for refill | Qualified clocks, precise exhaustion preemption, bounded kernel work, privileged latency policy and service-call sponsorship |
| RAM | Physical reservations cover images, stacks, page tables and buffers; accepted readers retain original sponsor charges | Active pressure policy and explicit sponsorship transfer |
| Admission | Executable capabilities choose code for funded unbound accounts; recursive pools and rollback | General executable transfer/loading and larger arenas |
| Devices | Local register/IRQ grants proved by userspace COM2; restricted framebuffer/PS2/ATA adapters | Prove MMIO/interrupt grants with another device and ISA; DMA needs a separate ownership/isolation design |

Matrix input now wakes idle apps; retained local scenes replay after placement
changes. The desktop test checks idle CPU use and moves without app repaint.
`--matrix-desktop-test --pressure` adds a spinner inside a 20% nested CPU ceiling
with persistent storage and reports key-to-scanout latency. Equal
turns and the remaining synchronous parent RPC paths still need latency work. Owned surface objects now replace fixed
path keys and conserve backing through recursive allowances. Physical admission
funding and shared image objects now reach the integrated desktop. The lab exercises explicit root-adapter recovery; production trusted-path behavior remains an owner-level
policy question. SMP, GPU support and the database filesystem follow these
boundaries rather than becoming prerequisites for exercising them.

Local names are scoped by caller and API namespace, not globally unique bearer
secrets. The same number can designate unrelated rights in two processes. Passing
it as message data installs no authority. Counters survive task-slot reuse so old
names cannot resurrect a replacement's grants; unsuccessful admission may burn
reserved names. This does not conceal timing, resource contention or all local
admission history. No claim of whole-system noninterference follows from it.
