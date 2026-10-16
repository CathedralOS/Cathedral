# Rust implementation map

This map connects implemented mechanisms to their integrated consumers, proofs
and remaining restrictions. The [lab guide](README.md) owns detailed ABI/build
instructions. [Owner questions](../OWNER_QUESTIONS.md) own unresolved semantics.

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
bare-metal targets. No guest runtime adapter exists and no engine is selected.
Arbitrary IO faults, active-reader pressure, metadata queries, physical allocation
and sustained low-space progress remain next evidence. The guest private catalog
is unchanged; see the engine lab's measured table, portability assessment and limits.

## Resumable storage and provider recovery

Child storage requests now return to the matrix event loop while awaiting replies.
The root's separate provider links let physical input and sibling rendering continue
through a blocked save. Nested matrices in the visible desktop now receive a separate catalog pair in
[their named startup bundle](platform/libraries/user-runtime/matrix.rs). A gated
save leaves a sibling inside that same matrix able to consume pointer/keyboard
input and repaint. The legacy headless fixtures retain their shared-stream mode.
Owner grant/revoke and teardown aborts remain synchronous bounded operations.
Follow [storage forwarding](platform/services/matrix/service/parent/storage/storage.rs),
[pending replies](platform/services/matrix/service/parent/storage/pending.rs) and
[deferred stream policy](platform/services/matrix/service/deferred.rs).

The [desktop stall proof](distribution/lab/matrix/desktop/stalls.rs) gates writes
before execution and after durable commit. Its [nested peer](distribution/lab/matrix/desktop/stalls_peer.rs)
checks input and repaint while blocked, plus survival across catalog revocation,
provider replacement and writer death. It also checks
normal completion, timeout followed by a late reply, refusal to recycle an accepted
live link, and provider replacement with explicit fresh grants. Unacknowledged
writes are reconciled by readback, never automatically retried. Root storage
transport loss revokes the affected binding without killing display/input.
Separate parent catalog transport loss retires that connection and its grants;
legacy shared-stream failures still retire the nested matrix.

[Provider recovery](distribution/lab/matrix/desktop/providers.rs) kills the physical
input decoder and display provider. Input loss cancels focus while apps survive;
display loss retires the root. Distribution policy explicitly tears down/rebuilds
adapters and the tree, re-funds the root from reclaimed reservations, then restores
committed app colors. This is a lab restart policy, not a trusted recovery UI.

The arena now supports twelve accounts; the pressure desktop uses nine so storage,
the capped spinner and all recovery checks run together. IPC backing stays at 32
channels and existing four-channel defaults remain: larger compositions must fit
explicit reservations. Account liveness, clock, pool, cancellation and region masks
use u16. The launch-loan packet is now 152 bytes with 256-byte-aligned runtime
storage. Five startup endpoints, a twenty-channel per-account ceiling and twenty
dynamic handles support the split. The global channel arena remains 32; startup
explicitly funds the root/inner connections. Pausing catalog request streams
while storage is pending preserves the eight-handle wait bound. Host tests cover
high-account isolation, full startup bundles, exhaustion and reclamation.
The traced debug matrix grew to 67 loaded pages: the image bound is now 80 pages
and its frame ledger 160 entries, preserving the 16 KiB stack, 1 MiB image window,
u128 writable-backing mask and physical funding checks. ELF tests cover the exact
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
Follow [desktop recovery](distribution/lab/matrix/desktop/recovery.rs) and
[app state](distribution/lab/matrix/desktop/app/state.rs).

[Publication](platform/services/matrix/service/parent/presentation.rs) now resumes
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
[matrix charter](platform/services/matrix/CHARTER.md) for the transaction path and
limits, including non-transactional owner placement and no vblank guarantee.

Debug integration also checks the fixed userspace stack/image bounds. Persistent
matrix batches and compositor staging live in process data storage; constant
input-queue initialization avoids large nested debug constructor temporaries.
The debug desktop and matrix-storage fixtures pass without enlarging kernel limits.

## Recursive storage delegation

[Matrix storage scopes](platform/services/matrix/world/storage/README.md) delegate
whole private catalog roots through the ordinary matrix executable. Every hop
checks connection identity and a local binding revision; siblings have separate
root data and generations. Revocation/death aborts staging and regranting an
ancestor does not restore descendants automatically. The headless
`--matrix-storage-test --release` fixture checks these boundaries and committed
data after restarting the matrix tree and storage provider. Dynamic namespaces,
read-only rights and storage quotas remain future work; the visible desktop now consumes these grants.

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
debug/release proofs, the matrix
desktop's physical input/recovery checks, and ordinary `--input-test` with seven
acknowledged storage updates. The recovery and power-cut profiles also pass in
release after correcting the stale lab expectations described below.

The ARM kernel-task bring-up caught excessive debug stack use in the lab's heap
exhaustion cleanup. Borrowing its bookkeeping array avoids by-value iterator
copies; both backends retain guarded 64-KiB task stacks. Debug ARM now completes
the same lifecycle/rollback suite before its user-process proof.

## Current integrated path

Start at [distribution desktop policy](distribution/lab/matrix/desktop/desktop.rs).
It admits display/input adapters and gives a root matrix code grants and a funded
process pool with physical-memory funding.
The root matrix admits an inner matrix and an outside app; the inner admits its
own app. Both run the same [matrix service](platform/services/matrix/service/service.rs).
Only the root adapts physical input and compositor output.

```powershell
python tools/boot-harness-rs/run.py --matrix-desktop-test --release --timeout 120
python tools/boot-harness-rs/run.py --profile tools/boot-harness-rs/profiles/matrix-desktop.json --release --window
```

Each app has a base surface, an independent light accent and a retained checkerboard bitmap. Click either app,
then Enter to change its base color. The left app is nested;
the right app is its outer sibling. Startup exercises inner-subtree replacement
and checks that old pixels disappear while the sibling survives. The automated
host test adds real mouse/keyboard injection and framebuffer checks.

The earlier `session.json` desktop remains a richer compositor/input regression
fixture. The default distribution profile still starts the storage/status demo.
Neither is presented as using the new matrix-owned lifecycle. These fixtures
remain until their extra behavior has moved into the integrated path with tests.

## Mechanisms, consumers and evidence

| Mechanism / owner | Integrated consumer | Proof | Remaining restriction |
| --- | --- | --- | --- |
| [Address spaces, admission, faults](kernel/core/user_tasks/user_tasks.rs) | All separately compiled services/apps | `--smoke`: rollback, faults, stale tasks, frame/heap reclamation | One CPU (x86-64 or ARM64 proof); bounded executable catalogue and twelve-account arena |
| [Dynamic IPC and startup bundles](kernel/core/ipc/ipc.rs) | Matrix-child, root-display and physical-source connections | `--ipc-test`: moves, rejected bundles, death, quotas | Small copied messages; accepted moves can outlive their creator |
| [Backed IPC reservations](kernel/core/ipc/budget/budget.rs) | Every initial/launch account, including matrix children | Host tests fill all 32 slots; `--ipc-test` exhausts restarted creators while a sibling fills its reservation | Owned idle accounts support conserving subdivision; no sponsorship transfer |
| [CPU turns and measured ceilings](kernel/core/scheduler/README.md) | Every user launch subtree, including nested matrices | Host accounting tests; `--smoke` checks capped spinners, yields, restart debt and idle refill | Boot/owned-idle burst/refill caps; periodic-timer overrun; no deadlines, weights or service-call donation |
| [Physical-memory funding](kernel/core/user_tasks/resources/physical/README.md) | Every admitted image, stack, page table and runtime buffer; recursive matrix pool | Host conservation/bank tests; smoke rollback and retained readers; admission exhaustion and desktop pixel checks | Eager bounded bank; idle-child assignment; no swap, active rebudgeting or sponsorship transfer |
| [Recursive launch loans](kernel/core/user_tasks/lifecycle/supervision/delegation.rs) | Matrices own direct children, lend budgets further, reclaim subtrees | `--ipc-test` plus matrix replacement tests | Static launch loans plus executable-authorized process pools; fixed account arena |
| [Executable-authorized admission](kernel/core/user_tasks/admission/pool/README.md) | Apps and the matrix desktop construct descendants from code authority and a process pool | `--admission-test`: recursive launch, exhaustion, load rollback, retained funding, sibling survival | Boot-supplied catalogue; twelve accounts; device grants remain static |
| [Matrix lifecycle](platform/services/matrix/service/children/children.rs) | Same provider above apps and another matrix | `--matrix-test`: replacement preserves an outside sibling | Two direct children; distribution supplies policy through owner-only control |
| [Local viewport, focus and capture](platform/services/matrix/world/world.rs) | Nested and sibling visible apps | Host policy tests; `--matrix-test`; `--matrix-desktop-test` physical focus | Five keys; coalesced wake channels; synchronous upward refresh |
| [Owned surfaces](platform/services/matrix/world/surface/README.md) | App rectangles cross actual matrix boundaries to the existing compositor | Host quota/name/clipping/stacking tests; scanout removal and teardown checks | Sixteen surfaces; bounded immutable RGBX images; no text surfaces or vblank guarantees |
| [Retained compositor and hardware input](platform/services/display/compositor/compositor.rs) | Root adapter uses existing scanout/cursor and shared input decoders | `--matrix-desktop-test`; `--session-test --pressure` | Existing richer scenes/hosting remain in the earlier session fixture |
| [Private/shared page lifetime](kernel/core/user_tasks/memory/README.md) | Matrix bitmaps and existing rendering/storage labs | Host transactions; both CPU boot proofs; matrix desktop | Four reader rights per task, sixteen-page budgets; immutable copies only; original sponsor retained |
| [Object storage and recovery](platform/services/storage/service/service.rs) | Status/counter and private matrix app colors | Storage host tests; `--storage-test`; two-boot `--matrix-desktop-test` | Whole-root delegation only; not the custom database filesystem |

## Kernel boundary audit and next steps

The discoverability pass reviewed all 27 Rust package trees and preserves the
[entry routes](../wiki/architecture/discoverability.md): boot composition, task
lifetime, CPU transitions, service event ordering, scene publication and lab
scenario ownership. Contracts, small drivers, init/application workflows and the
matrix/input service boundaries remain cohesive without extra forwarding layers.

Subsystem entrances live in named files inside their folders (`memory/memory.rs`),
alongside the responsibilities they introduce. `mod.rs` contains only disposable
Rust plumbing; the named reading path is intended to survive the Omega port.
The next reading level owns checked IPC delivery,
page sharing/retirement, scene geometry, pointer queues and forwarded matrix
control requests. Cohesive leaves remain sibling files; the guide treats this as
a reading-path preference rather than a file-count or line-count rule.

**Recovery verification restored:** the status app's
[authority probe](distribution/applications/status/application/lab/lab.rs) and
[buffer probe](distribution/applications/status/application/lab/buffers.rs) now
expect `BAD_HANDLE` for cancellation of handle zero and an unoffered private page.
These names confer no authority in the receiving namespace. Both probes had
retained the older `DENIED` expectation; kernel behavior is unchanged.

`--recovery-test --release --timeout 120` passes provider crash/spin/block
recovery, application replacement, sibling preservation, checked disk transport,
private storage authority, exact scanout and lost-reply reconciliation.
`--storage-test --release --timeout 120` passes fourteen boots covering reboot
durability, all four power-cut phases, continued writes, lost acknowledgements
and legacy-format migration followed by another reboot.

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
