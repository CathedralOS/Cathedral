# Rust implementation map

This map connects implemented mechanisms to their integrated consumers, proofs
and remaining restrictions. The [lab guide](README.md) owns detailed ABI/build
instructions. [Owner questions](../OWNER_QUESTIONS.md) own unresolved semantics.

## Userspace register and interrupt grants

The recursive matrix desktop also has bounded atomic presentation transactions:
four distinct retained surfaces, staged sealed readers, all-or-nothing publication
and latest-commit retry scoped to connection/view. Host policy tests and the
release desktop fixture under CPU pressure verify rejection, old pixels before
commit, bitmap forwarding, input and subtree recovery. See the
[matrix charter](platform/services/matrix/CHARTER.md) for the transaction path and
limits, including non-transactional owner placement and no vblank guarantee.

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
texture decoding, GPU import and transactional multi-surface commits remain open.

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
| [Address spaces, admission, faults](kernel/core/user_tasks/user_tasks.rs) | All separately compiled services/apps | `--smoke`: rollback, faults, stale tasks, frame/heap reclamation | One CPU (x86-64 or ARM64 proof); bounded executable catalogue and eight-account arena |
| [Dynamic IPC and startup bundles](kernel/core/ipc/ipc.rs) | Matrix-child, root-display and physical-source connections | `--ipc-test`: moves, rejected bundles, death, quotas | Small copied messages; accepted moves can outlive their creator |
| [Backed IPC reservations](kernel/core/ipc/budget/budget.rs) | Every initial/launch account, including matrix children | Host tests fill all 32 slots; `--ipc-test` exhausts restarted creators while a sibling fills its reservation | Owned idle accounts support conserving subdivision; no sponsorship transfer |
| [CPU turns and measured ceilings](kernel/core/scheduler/README.md) | Every user launch subtree, including nested matrices | Host accounting tests; `--smoke` checks capped spinners, yields, restart debt and idle refill | Boot/owned-idle burst/refill caps; periodic-timer overrun; no deadlines, weights or service-call donation |
| [Physical-memory funding](kernel/core/user_tasks/resources/physical/README.md) | Every admitted image, stack, page table and runtime buffer; recursive matrix pool | Host conservation/bank tests; smoke rollback and retained readers; admission exhaustion and desktop pixel checks | Eager bounded bank; idle-child assignment; no swap, active rebudgeting or sponsorship transfer |
| [Recursive launch loans](kernel/core/user_tasks/lifecycle/supervision/delegation.rs) | Matrices own direct children, lend budgets further, reclaim subtrees | `--ipc-test` plus matrix replacement tests | Static launch loans plus executable-authorized process pools; fixed account arena |
| [Executable-authorized admission](kernel/core/user_tasks/admission/pool/README.md) | Apps and the matrix desktop construct descendants from code authority and a process pool | `--admission-test`: recursive launch, exhaustion, load rollback, retained funding, sibling survival | Boot-supplied catalogue; eight accounts; device grants remain static |
| [Matrix lifecycle](platform/services/matrix/service/children/children.rs) | Same provider above apps and another matrix | `--matrix-test`: replacement preserves an outside sibling | Two direct children; distribution supplies policy through owner-only control |
| [Local viewport, focus and capture](platform/services/matrix/world/world.rs) | Nested and sibling visible apps | Host policy tests; `--matrix-test`; `--matrix-desktop-test` physical focus | Five keys; coalesced wake channels; synchronous upward refresh |
| [Owned surfaces](platform/services/matrix/world/surface/README.md) | App rectangles cross actual matrix boundaries to the existing compositor | Host quota/name/clipping/stacking tests; scanout removal and teardown checks | Sixteen surfaces; bounded immutable RGBX images; no text surfaces or atomic scene commits yet |
| [Retained compositor and hardware input](platform/services/display/compositor/compositor.rs) | Root adapter uses existing scanout/cursor and shared input decoders | `--matrix-desktop-test`; `--session-test --pressure` | Existing richer scenes/hosting remain in the earlier session fixture |
| [Private/shared page lifetime](kernel/core/user_tasks/memory/README.md) | Matrix bitmaps and existing rendering/storage labs | Host transactions; both CPU boot proofs; matrix desktop | Four reader rights per task, sixteen-page budgets; immutable copies only; original sponsor retained |
| [Object storage and recovery](platform/services/storage/service/service.rs) | Default status/counter distribution | Storage host tests; `--storage-test` | Not yet an attenuated matrix namespace; not the custom database filesystem |

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
| Devices | Concrete framebuffer/PS2/ATA grants, no arbitrary device discovery | One reusable userspace interrupt/MMIO boundary, proved with another driver; DMA needs a separate ownership/isolation design |

Matrix input now wakes idle apps; retained local scenes replay after placement
changes. The desktop test checks idle CPU use and moves without app repaint.
`--matrix-desktop-test --pressure` adds a spinner inside a 20% nested CPU ceiling
and reports key-to-scanout latency (roughly 0.5 to 0.9 seconds in recent runs, including tracing overhead). Equal
turns and synchronous parent RPC still need latency work. Owned surface objects now replace fixed
path keys and conserve backing through recursive allowances. Physical admission
funding and shared image objects now reach the integrated desktop. Root-adapter recovery/trusted-path behavior remains an owner-level
policy question. SMP, GPU support and the database filesystem follow these
boundaries rather than becoming prerequisites for exercising them.

Local names are scoped by caller and API namespace, not globally unique bearer
secrets. The same number can designate unrelated rights in two processes. Passing
it as message data installs no authority. Counters survive task-slot reuse so old
names cannot resurrect a replacement's grants; unsuccessful admission may burn
reserved names. This does not conceal timing, resource contention or all local
admission history. No claim of whole-system noninterference follows from it.
