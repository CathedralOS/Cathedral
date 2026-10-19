# Discoverability architecture

Cathedral's source should be explorable from an obvious entrance. At each level,
the reader sees the decisions relevant to that level and can follow an interesting
step to its owner. The code's execution path should supply the navigation.

## Entrances and descent

- An executable's `main.rs` exposes startup or leads directly to its named
  coordinator. A library exposes its public operation and the coordinator that
  implements it. Re-exports may preserve an API; they do not substitute for an
  understandable execution path.
- A coordinator shows actual sequencing, branching or dispatch. Its callees name
  meaningful work: validate composition, admit tasks, check providers, save state.
  Keep safety ordering and rollback obligations visible at the boundary.
- A subsystem's named entrance lives inside its folder: `admission/admission.rs`,
  with the next responsibilities beside it. That file shows the decisions at its
  level, with implementation details owned below. Package entrances remain
  `main.rs` or `lib.rs`; separately built host programs can have their own `main.rs`.
- Rust's `mod.rs` is disposable language plumbing: module declarations, exports,
  module attributes and inclusion of the named body. It contains no state,
  algorithms, orchestration or executable wrappers. `include!("admission.rs")`
  keeps the named body in the existing module scope without inventing an extra
  ownership or visibility layer. Start reading in `admission.rs`; the Omega port
  preserves that file and its sibling responsibilities while dropping the plumbing.
- Keep a coherent leaf together; line counts are not architectural boundaries.
  Avoid chains of substantive files that only forward to the next file.
- These are reading-path preferences, not a fixed directory template. A few
  cohesive sibling files are fine when naming and control flow make the starting
  point obvious. Do not manufacture folders or split an operation merely to
  satisfy a file-count rule.
- Folder names identify ownership or purpose. Prefer `kernel_tasks`, `user_tasks`
  and `lab` over ambiguous peers such as `tasks`, `users` and `programs`.
  A generic `runtime`, `helpers` or `common` must not accumulate unrelated work.
- Group supporting packages beneath a real subsystem as that subsystem grows.
  Preserve independently owned libraries, contracts and Cargo boundaries; folder
  grouping is not permission to absorb them into a coordinator.
- Small entrance comments name the mechanism, its important invariant and the
  next owners. Documentation supplements the navigable implementation.

## Review each descent

Choose an entrance from the file tree and read its body: what happens first,
what decision selects the next owner, and where does a result or failure go?
Repeat that check in the selected child. Semantic folders must narrow the job;
a large folder named after the coordinator must not conceal all peer domains.
A module list or README cannot stand in for the operation's actual control flow.
Representation owners may start with their principal state and invariants.

The Rust kernel's normal route starts in `boot/uefi/main.rs`, enters
`boot/uefi/startup.rs`, then `core/user_tasks/user_tasks.rs`. That task entrance owns
preparation, physical reservation, initial admission, execution and collection.
Inside `user_tasks/`, descend into each folder's named entrance: `startup` validates
composition and prepares grants; `admission` constructs images; `execution`
orders deferred work and CPU re-entry; `lifecycle` owns spawn/teardown; `memory`
dispatches page-object work to allocation, sharing and retirement. `address_space`
owns each process's image layout, checked copies and frame ledger, delegating
translation encoding to the selected CPU backend.
`session.rs` holds their shared state, not a second hidden execution pipeline.

The second CPU proof follows the same boot entry into
`arch/isa/aarch64/memory/memory.rs`, `interrupts/interrupts.rs` and
`context/context.rs`. Its saved context returns to the shared core trap dispatcher;
`boot/uefi/lab/arm64/arm64.rs` visibly sequences the headless proof.
`arch/lib.rs` lists the shared machine interface; `arch/isa/isa.rs` selects CPU
mechanisms, while `arch/board/board.rs` selects actual device adapters. Both CPUs
run the shared trusted-kernel-task lab, with register probes beneath that lab.

Trap requests descend from `execution/traps.rs` into `syscalls/syscalls.rs`, then a
resource family. For example, `syscalls/ipc/ipc.rs` selects copied delivery,
capability transfer, links or readiness, while `syscalls/devices/devices.rs` selects a
boot-granted device adapter. Pure policies live with their owner: process pools
under admission, supervision under lifecycle, page regions under memory, and
raw input queues under the keyboard adapter. Shared frame allocation, IPC and
scheduling remain core peers. Small coherent leaves need no extra wrapper.

## Executable and protocol boundaries

An OS has several entrances, including boot, task entry and interrupts. Describe
each real route rather than pretending all work is a single synchronous pipeline.
Kernel boot admits the initial executable selected by host composition. The
distribution launches providers and applications. IPC crosses into separately
compiled service entrances.

At an IPC client, identify the shared protocol and the receiving service entrance.
At a provider, identify its protocol and implementation owner. The distribution's
[composition map](../../source-rs/distribution/README.md) connects the stock
executables. These are navigation links, not reverse code dependencies or kernel
knowledge of a particular distribution.

## Current Rust entrances

| Interest | Start here | Descend into |
| --- | --- | --- |
| CPU turn sharing | [Task scheduling](../../source-rs/kernel/core/scheduler/scheduler.rs) | [Hierarchical selection](../../source-rs/kernel/core/scheduler/shares/shares.rs), [contract and proof route](../../source-rs/kernel/core/scheduler/README.md), [loan reattachment](../../source-rs/kernel/core/user_tasks/lifecycle/delegation.rs) |
| CPU time ceilings | [Persistent account charging](../../source-rs/kernel/core/scheduler/budget/budget.rs) | [Architecture clock](../../source-rs/kernel/arch/isa/x86_64/cpu_clock.rs), [trap samples](../../source-rs/kernel/core/user_tasks/execution/traps.rs), [measured QEMU proof](../../source-rs/kernel/boot/uefi/lab/supervision/cpu/budget.rs) |
| Firmware and kernel startup | [boot main](../../source-rs/kernel/boot/uefi/main.rs) | Firmware handoff, memory, interrupts, supplied-program startup |
| Host-supplied boot composition | [build script](../../source-rs/kernel/boot/uefi/build/main.rs) | [Grant/image/link generation](../../source-rs/kernel/boot/uefi/build/composition/composition.rs), [reservation validation](../../source-rs/kernel/boot/uefi/build/composition/reservations.rs), [ELF bundling](../../source-rs/kernel/boot/uefi/build/artifacts.rs); consumed by [startup](../../source-rs/kernel/boot/uefi/startup.rs) |
| CPU interrupt installation | [interrupt entrance](../../source-rs/kernel/arch/isa/x86_64/interrupts/interrupts.rs) | [Privilege and vector tables](../../source-rs/kernel/arch/isa/x86_64/interrupts/tables.rs), [assembly bindings](../../source-rs/kernel/arch/isa/x86_64/interrupts/entries.rs) |
| Kernel image protection | [UEFI section capture](../../source-rs/kernel/boot/uefi/firmware/firmware.rs) | [Physical permission intent](../../source-rs/kernel/arch/memory/image.rs), [ISA mapping and fault proofs](../../source-rs/kernel/arch/memory/README.md) |
| Isolated address spaces | [ownership and rollback](../../source-rs/kernel/core/user_tasks/address_space/address_space.rs) | [Image validation](../../source-rs/kernel/core/user_tasks/address_space/image.rs), [loading](../../source-rs/kernel/core/user_tasks/address_space/creation.rs), [checked copies](../../source-rs/kernel/core/user_tasks/address_space/access.rs), [CPU mapping boundary](../../source-rs/kernel/arch/README.md) |
| Userspace device lifecycle | [Grant/IRQ policy](../../source-rs/kernel/core/user_tasks/devices/devices.rs) | [Syscall entrance](../../source-rs/kernel/core/user_tasks/syscalls/devices/leased.rs), [board selection](../../source-rs/kernel/arch/board/registers.rs), [userspace UART](../../source-rs/platform/drivers/uart_16550/interrupt.rs), [contract and proof](../../source-rs/kernel/core/user_tasks/devices/README.md) |
| Privileged device access | [core authority and buffers](../../source-rs/kernel/core/user_tasks/syscalls/devices/devices.rs) | [Board selection](../../source-rs/kernel/arch/board/board.rs), [PC command restrictions](../../source-rs/kernel/arch/board/pc/pc.rs), [virt hardware](../../source-rs/kernel/arch/board/virt/virt.rs) |
| Isolated tasks | [user-task session](../../source-rs/kernel/core/user_tasks/user_tasks.rs) | Composition, preparation, admission, execution, outcomes |
| CPU re-entry | [session traps](../../source-rs/kernel/core/user_tasks/execution/traps.rs) | Syscall dispatch, wakeups, scheduling and root selection |
| General IPC | [IPC coordinator](../../source-rs/kernel/core/ipc/ipc.rs) | [Process-local grants](../../source-rs/kernel/core/ipc/grants.rs), [creation and transfer](../../source-rs/kernel/core/ipc/channels.rs), [teardown](../../source-rs/kernel/core/ipc/lifecycle.rs); [ordinary-worker proof](../../source-rs/distribution/lab/channels/scenario.rs) |
| Sealed object transfer | [Memory coordinator](../../source-rs/kernel/core/user_tasks/memory/memory.rs) | [Reader custody](../../source-rs/kernel/core/user_tasks/memory/regions/readers.rs), [checked IPC transaction](../../source-rs/kernel/core/user_tasks/syscalls/ipc/capabilities.rs), [mapping/funding contract and proofs](../../source-rs/kernel/core/user_tasks/memory/README.md) |
| IPC reservations | [Account limits and charges](../../source-rs/kernel/core/ipc/budget/budget.rs) | [Admission validation](../../source-rs/kernel/core/ipc/bootstrap.rs), [final closure](../../source-rs/kernel/core/ipc/lifecycle.rs), [restart and sibling proof](../../source-rs/distribution/lab/channels/sponsorship/sponsorship.rs) |
| Child startup environment | [Launch staging](../../source-rs/kernel/core/user_tasks/syscalls/tasks/launch.rs) | [Admission and commit ordering](../../source-rs/kernel/core/user_tasks/lifecycle/lifecycle.rs), [bundle validation and installation](../../source-rs/kernel/core/ipc/initial/initial.rs), [userspace Environment](../../source-rs/platform/libraries/user-runtime/task/environment.rs) |
| Resource subdivision | [Resource transaction](../../source-rs/kernel/core/user_tasks/resources/resources.rs) | [Authority/copy boundary](../../source-rs/kernel/core/user_tasks/syscalls/resources.rs), [contract and proofs](../../source-rs/kernel/core/user_tasks/resources/README.md) |
| Physical-memory funding | [Funding and invariants](../../source-rs/kernel/core/user_tasks/resources/physical/physical.rs) | [Physical bank](../../source-rs/kernel/core/user_tasks/memory/frames.rs), [admission commit/rollback](../../source-rs/kernel/core/user_tasks/admission/admission.rs), [contract and proofs](../../source-rs/kernel/core/user_tasks/resources/physical/README.md) |
| Executable-authorized admission | [Admission syscall coordinator](../../source-rs/kernel/core/user_tasks/syscalls/admission/admission.rs) | [Reservation transaction](../../source-rs/kernel/core/user_tasks/syscalls/admission/reserve.rs), [process pool](../../source-rs/kernel/core/user_tasks/admission/pool/pool.rs), [contract and proofs](../../source-rs/kernel/core/user_tasks/admission/pool/README.md) |
| Recursive admission | [Launch syscall staging](../../source-rs/kernel/core/user_tasks/syscalls/tasks/launch.rs) | [Loan validation and commit](../../source-rs/kernel/core/user_tasks/lifecycle/delegation.rs), [loan lifetime](../../source-rs/kernel/core/user_tasks/lifecycle/supervision/delegation.rs), [same-code recursive proof](../../source-rs/distribution/lab/channels/tree/node.rs) |
| Caller-local kernel names | [Name allocation](../../source-rs/kernel/core/names.rs) | [Admission reservations](../../source-rs/kernel/core/user_tasks/lifecycle/grants.rs), [loan return](../../source-rs/kernel/core/user_tasks/lifecycle/supervision/delegation.rs), [local region mappings and offers](../../source-rs/kernel/core/user_tasks/memory/regions/regions.rs) |
| Kernel syscall enforcement | [syscall dispatcher](../../source-rs/kernel/core/user_tasks/syscalls/syscalls.rs) | IPC, tasks, readiness and specific device grants |
| Trusted kernel tasks | [task orchestration and API](../../source-rs/kernel/core/kernel_tasks/kernel_tasks.rs) | [Admission and reaping](../../source-rs/kernel/core/kernel_tasks/lifecycle.rs), [execution](../../source-rs/kernel/core/kernel_tasks/execution/execution.rs), [trap selection](../../source-rs/kernel/core/kernel_tasks/execution/traps.rs), stack ownership |
| Distribution startup | [desktop session](../../source-rs/distribution/desktop/session/session.rs) | Provider admission, workspace and menu placement, [choice policy](../../source-rs/distribution/desktop/session/shell/shell.rs), storage supervision |
| Scoped service acquisition | [Bootstrap contract](../../source-rs/platform/contracts/services/README.md) | [Runtime acquisition](../../source-rs/platform/libraries/user-runtime/services/services.rs), [endpoint custody](../../source-rs/platform/libraries/user-runtime/services/offer.rs), [stock caller authentication](../../source-rs/distribution/matrix/service/children/services.rs) |
| Application entry and termination | [entry macros](../../source-rs/platform/libraries/user-runtime/entry.rs) | Installed environment, supplied Rust main, [task exit](../../source-rs/platform/libraries/user-runtime/task/task.rs), [bounded diagnostics](../../source-rs/platform/libraries/user-runtime/diagnostics.rs) |
| Shared service transport | [server event loop](../../source-rs/platform/libraries/user-runtime/server/server.rs) | [Connection admission and retirement](../../source-rs/platform/libraries/user-runtime/server/connections.rs), [fair polling and readiness waits](../../source-rs/platform/libraries/user-runtime/server/polling.rs) |
| Notes application | [notes session](../../source-rs/distribution/applications/notes/session/session.rs) | Local editing, [object storage](../../source-rs/distribution/applications/notes/session/storage.rs), presentation |
| Display protocol selection | [display main](../../source-rs/platform/services/display/main.rs) | [Immediate drawing](../../source-rs/platform/services/display/direct/direct.rs) or [retained compositor](../../source-rs/platform/services/display/compositor/compositor.rs); both use the framebuffer surface |
| Retained rendering | [composition state](../../source-rs/platform/services/display/composition/composition.rs) | Row composition order, [scene snapshots and hosting](../../source-rs/platform/services/display/composition/scene/scene.rs), [input state and focus](../../source-rs/platform/services/display/composition/input/input.rs) |
| Scene publication | [client submission](../../source-rs/platform/services/display/composition/scene/scene.rs) | [Snapshot validation](../../source-rs/platform/services/display/composition/scene/snapshot.rs), swap on success, [damage and rasterization](../../source-rs/platform/services/display/composition/scene/raster.rs) |
| Physical input | [input main](../../source-rs/platform/services/input/main.rs) | [Keyboard service](../../source-rs/platform/services/input/keyboard/keyboard.rs) or [compositor source](../../source-rs/platform/services/input/pointer.rs), shared controller and pure decoders |
| Storage service | [storage main](../../source-rs/platform/services/storage/main.rs) | [Request dispatch](../../source-rs/platform/services/storage/service.rs), [sealed bulk replies](../../source-rs/platform/services/storage/bulk.rs), optional fault gate |
| Compositor input | [compositor loop](../../source-rs/platform/services/display/compositor/compositor.rs) | [Connection authorization](../../source-rs/platform/services/display/compositor/input.rs), [pointer lifetime](../../source-rs/platform/services/display/composition/input/input.rs), routing and cursor damage |
| Focus and hosted scenes | [session dispatch](../../source-rs/platform/services/display/compositor/session.rs) | [Focus attenuation](../../source-rs/platform/services/display/composition/input/focus/focus.rs), [hosting and clips](../../source-rs/platform/services/display/composition/scene/hosting.rs), deferred event delivery |
| Integrated matrix desktop | [Distribution policy](../../source-rs/distribution/desktop/session/session.rs) | [Matrix service loop](../../source-rs/distribution/matrix/service/service.rs), [child admission](../../source-rs/distribution/matrix/service/children/admission.rs), [local surface clipping](../../source-rs/platform/libraries/matrix/world/surface/surface.rs), [root compositor adapter](../../source-rs/distribution/matrix/service/parent/rendering.rs) |
| Owned matrix surfaces | [Request/commit coordinator](../../source-rs/distribution/matrix/service/requests/surfaces.rs) | [Names, quotas and clipping](../../source-rs/platform/libraries/matrix/world/surface/surface.rs), [contract and proof route](../../source-rs/platform/libraries/matrix/world/surface/README.md), [root adapter](../../source-rs/distribution/matrix/service/parent/rendering.rs) |
| Recursive matrix boundary | [matrix service](../../source-rs/distribution/matrix/service/service.rs) | [Parent adaptation](../../source-rs/distribution/matrix/service/parent/parent.rs), [local world](../../source-rs/platform/libraries/matrix/world/world.rs), child grants and input; [IPC fixture](../../source-rs/distribution/desktop/scenario/scenario.rs) |
| Matrix startup connection custody | [Shared admission](../../source-rs/platform/libraries/user-runtime/matrix/admission/admission.rs) | [Channel allocation/rollback](../../source-rs/platform/libraries/user-runtime/matrix/admission/connections.rs), [child startup view](../../source-rs/platform/libraries/user-runtime/matrix/matrix.rs); used by stock admission and the [independent matrix](../../source-rs/distribution/lab/matrix-composition/matrix/session.rs) |
| Interactive recovery lab | [desktop session](../../source-rs/distribution/lab/compositor/supervisor/desktop/desktop.rs) | Admission, grants, idle probes and recovery; [client session](../../source-rs/distribution/lab/compositor/client/session/session.rs) owns desired state and presentation |
| Private storage | [object store](../../source-rs/platform/libraries/database-objects/lib.rs) | [Records](../../source-rs/platform/libraries/database-objects/records.rs), [objects](../../source-rs/platform/libraries/database-objects/objects.rs), [publication](../../source-rs/platform/libraries/database-objects/publication.rs), [reading](../../source-rs/platform/libraries/database-objects/reading.rs), [collection](../../source-rs/platform/libraries/database-objects/collection.rs) |
| Private object proof | [objects lab](../../source-rs/distribution/lab/objects/main.rs) | Supervisor, client and retention clients over explicit links |
| Kernel exercises | [boot lab](../../source-rs/kernel/boot/uefi/lab/lab.rs) | Mechanism suites own their subordinate lifecycle/watchdog fixtures; guest counterparts live in `distribution/lab` |
| Channel exercises | [suite order](../../source-rs/distribution/lab/channels/scenario.rs) | [Delivery and worker](../../source-rs/distribution/lab/channels/delivery/delivery.rs), startup, recursive loans, funded admission; each owns its hostile-input probes |

The ownership rules remain in [repository layout](repository_layout.md).
