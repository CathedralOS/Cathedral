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
- When a Rust subsystem needs a folder, prefer its entrance inside that folder:
  `admission/mod.rs`, with its next responsibilities beside it. Package entrances
  remain `main.rs` or `lib.rs`; separately built host programs can have their own
  `main.rs`. This keeps the entrance and the implementation it introduces together.
- `mod.rs` shows the decisions belonging to that level, not every implementation
  detail below it. It is neither a dumping ground nor a module list forwarding to
  a hidden coordinator. Keep a coherent leaf together; line counts are not
  architectural boundaries. Avoid chains of files that only forward to the next.
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
`boot/uefi/startup.rs`, then `core/user_tasks/mod.rs`. That task entrance owns
preparation, physical reservation, initial admission, execution and collection.
Inside `user_tasks/`, descend into each folder's `mod.rs`: `startup` validates
composition and prepares grants; `admission` constructs images; `execution`
orders deferred work and CPU re-entry; `lifecycle` owns spawn/teardown; `memory`
dispatches page-object work to allocation, sharing and retirement.
`session.rs` holds their shared state, not a second hidden execution pipeline.

Trap requests descend from `execution/traps.rs` into `syscalls/mod.rs`, then a
resource family. For example, `syscalls/ipc/mod.rs` selects copied delivery,
capability transfer, links or readiness, while `syscalls/devices/mod.rs` selects a
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
| CPU turn sharing | [Task scheduling](../../source-rs/kernel/core/scheduler/mod.rs) | [Hierarchical selection](../../source-rs/kernel/core/scheduler/shares/mod.rs), [contract and proof route](../../source-rs/kernel/core/scheduler/README.md), [loan reattachment](../../source-rs/kernel/core/user_tasks/lifecycle/delegation.rs) |
| CPU time ceilings | [Persistent account charging](../../source-rs/kernel/core/scheduler/budget/mod.rs) | [Architecture clock](../../source-rs/kernel/arch/x86_64/cpu_clock.rs), [trap samples](../../source-rs/kernel/core/user_tasks/execution/traps.rs), [measured QEMU proof](../../source-rs/kernel/boot/uefi/lab/supervision/cpu/budget.rs) |
| Firmware and kernel startup | [boot main](../../source-rs/kernel/boot/uefi/main.rs) | Firmware handoff, memory, interrupts, supplied-program startup |
| Host-supplied boot composition | [build script](../../source-rs/kernel/boot/uefi/build/main.rs) | [Grant/image/link generation](../../source-rs/kernel/boot/uefi/build/composition/mod.rs), [reservation validation](../../source-rs/kernel/boot/uefi/build/composition/reservations.rs), [ELF bundling](../../source-rs/kernel/boot/uefi/build/artifacts.rs); consumed by [startup](../../source-rs/kernel/boot/uefi/startup.rs) |
| CPU interrupt installation | [interrupt entrance](../../source-rs/kernel/arch/x86_64/interrupts/mod.rs) | [Privilege and vector tables](../../source-rs/kernel/arch/x86_64/interrupts/tables.rs), [assembly bindings](../../source-rs/kernel/arch/x86_64/interrupts/entries.rs) |
| Isolated address spaces | [admission and rollback](../../source-rs/kernel/arch/x86_64/user/memory/mod.rs) | [Image validation](../../source-rs/kernel/arch/x86_64/user/memory/image.rs), [mapping construction](../../source-rs/kernel/arch/x86_64/user/memory/creation.rs), [checked buffer access](../../source-rs/kernel/arch/x86_64/user/memory/access.rs) |
| Isolated tasks | [user-task session](../../source-rs/kernel/core/user_tasks/mod.rs) | Composition, preparation, admission, execution, outcomes |
| CPU re-entry | [session traps](../../source-rs/kernel/core/user_tasks/execution/traps.rs) | Syscall dispatch, wakeups, scheduling and root selection |
| General IPC | [IPC coordinator](../../source-rs/kernel/core/ipc/mod.rs) | [Process-local grants](../../source-rs/kernel/core/ipc/grants.rs), [creation and transfer](../../source-rs/kernel/core/ipc/channels.rs), [teardown](../../source-rs/kernel/core/ipc/lifecycle.rs); [ordinary-worker proof](../../source-rs/distribution/lab/channels/scenario.rs) |
| IPC reservations | [Account limits and charges](../../source-rs/kernel/core/ipc/budget/mod.rs) | [Admission validation](../../source-rs/kernel/core/ipc/bootstrap.rs), [final closure](../../source-rs/kernel/core/ipc/lifecycle.rs), [restart and sibling proof](../../source-rs/distribution/lab/channels/sponsorship/mod.rs) |
| Child startup environment | [Launch staging](../../source-rs/kernel/core/user_tasks/syscalls/tasks/launch.rs) | [Admission and commit ordering](../../source-rs/kernel/core/user_tasks/lifecycle/mod.rs), [bundle validation and installation](../../source-rs/kernel/core/ipc/initial/mod.rs), [userspace Environment](../../source-rs/platform/libraries/user-runtime/task/environment.rs) |
| Resource subdivision | [Resource transaction](../../source-rs/kernel/core/user_tasks/resources/mod.rs) | [Authority/copy boundary](../../source-rs/kernel/core/user_tasks/syscalls/resources.rs), [contract and proofs](../../source-rs/kernel/core/user_tasks/resources/README.md) |
| Physical-memory funding | [Funding and invariants](../../source-rs/kernel/core/user_tasks/resources/physical/mod.rs) | [Physical bank](../../source-rs/kernel/core/user_tasks/memory/frames.rs), [admission commit/rollback](../../source-rs/kernel/core/user_tasks/admission/mod.rs), [contract and proofs](../../source-rs/kernel/core/user_tasks/resources/physical/README.md) |
| Executable-authorized admission | [Admission syscall coordinator](../../source-rs/kernel/core/user_tasks/syscalls/admission/mod.rs) | [Reservation transaction](../../source-rs/kernel/core/user_tasks/syscalls/admission/reserve.rs), [process pool](../../source-rs/kernel/core/user_tasks/admission/pool/mod.rs), [contract and proofs](../../source-rs/kernel/core/user_tasks/admission/pool/README.md) |
| Recursive admission | [Launch syscall staging](../../source-rs/kernel/core/user_tasks/syscalls/tasks/launch.rs) | [Loan validation and commit](../../source-rs/kernel/core/user_tasks/lifecycle/delegation.rs), [loan lifetime](../../source-rs/kernel/core/user_tasks/lifecycle/supervision/delegation.rs), [same-code recursive proof](../../source-rs/distribution/lab/channels/tree/node.rs) |
| Caller-local kernel names | [Name allocation](../../source-rs/kernel/core/names.rs) | [Admission reservations](../../source-rs/kernel/core/user_tasks/lifecycle/grants.rs), [loan return](../../source-rs/kernel/core/user_tasks/lifecycle/supervision/delegation.rs), [local region mappings and offers](../../source-rs/kernel/core/user_tasks/memory/regions/mod.rs) |
| Kernel syscall enforcement | [syscall dispatcher](../../source-rs/kernel/core/user_tasks/syscalls/mod.rs) | IPC, tasks, readiness and specific device grants |
| Trusted kernel tasks | [task orchestration and API](../../source-rs/kernel/core/kernel_tasks/mod.rs) | [Admission and reaping](../../source-rs/kernel/core/kernel_tasks/lifecycle.rs), [execution](../../source-rs/kernel/core/kernel_tasks/execution/mod.rs), [trap selection](../../source-rs/kernel/core/kernel_tasks/execution/traps.rs), stack ownership |
| Distribution startup | [init session](../../source-rs/distribution/init/session/mod.rs) | Provider readiness, application launch and supervision |
| Application entry and termination | [entry macros](../../source-rs/platform/libraries/user-runtime/entry.rs) | Installed environment, supplied Rust main, [task exit](../../source-rs/platform/libraries/user-runtime/task/mod.rs), [bounded diagnostics](../../source-rs/platform/libraries/user-runtime/diagnostics.rs) |
| Shared service transport | [server event loop](../../source-rs/platform/libraries/user-runtime/server/mod.rs) | [Connection admission and retirement](../../source-rs/platform/libraries/user-runtime/server/connections.rs), [fair polling and readiness waits](../../source-rs/platform/libraries/user-runtime/server/polling.rs) |
| Status application | [application workflow](../../source-rs/distribution/applications/status/application/mod.rs) | Connections, input, state, persistence, view and test probes |
| Display protocol selection | [display main](../../source-rs/platform/services/display/main.rs) | [Immediate drawing](../../source-rs/platform/services/display/direct/mod.rs) or [retained compositor](../../source-rs/platform/services/display/compositor/mod.rs); both use the framebuffer surface |
| Retained rendering | [composition state](../../source-rs/platform/services/display/composition/mod.rs) | Row composition order, [scene snapshots and hosting](../../source-rs/platform/services/display/composition/scene/mod.rs), [input state and focus](../../source-rs/platform/services/display/composition/input/mod.rs) |
| Scene publication | [client submission](../../source-rs/platform/services/display/composition/scene/mod.rs) | [Snapshot validation](../../source-rs/platform/services/display/composition/scene/snapshot.rs), swap on success, [damage and rasterization](../../source-rs/platform/services/display/composition/scene/raster.rs) |
| Physical input | [input main](../../source-rs/platform/services/input/main.rs) | [Keyboard service](../../source-rs/platform/services/input/keyboard/mod.rs) or [compositor source](../../source-rs/platform/services/input/pointer.rs), shared controller and pure decoders |
| Storage service | [storage main](../../source-rs/platform/services/storage/main.rs) | [Disk/catalog startup and request dispatch](../../source-rs/platform/services/storage/service/mod.rs), with recovery hooks beneath the service |
| Compositor input | [compositor loop](../../source-rs/platform/services/display/compositor/mod.rs) | [Connection authorization](../../source-rs/platform/services/display/compositor/input.rs), [pointer lifetime](../../source-rs/platform/services/display/composition/input/mod.rs), routing and cursor damage |
| Focus and hosted scenes | [session dispatch](../../source-rs/platform/services/display/compositor/session.rs) | [Focus attenuation](../../source-rs/platform/services/display/composition/input/focus/mod.rs), [hosting and clips](../../source-rs/platform/services/display/composition/scene/hosting.rs), deferred event delivery |
| Integrated matrix desktop | [Distribution policy](../../source-rs/distribution/lab/matrix/desktop/mod.rs) | [Matrix service loop](../../source-rs/platform/services/matrix/service/mod.rs), [child admission](../../source-rs/platform/services/matrix/service/children/admission.rs), [local surface clipping](../../source-rs/platform/services/matrix/world/surface/mod.rs), [root compositor adapter](../../source-rs/platform/services/matrix/service/parent/rendering.rs) |
| Owned matrix surfaces | [Request/commit coordinator](../../source-rs/platform/services/matrix/service/requests/surfaces.rs) | [Names, quotas and clipping](../../source-rs/platform/services/matrix/world/surface/mod.rs), [contract and proof route](../../source-rs/platform/services/matrix/world/surface/README.md), [root adapter](../../source-rs/platform/services/matrix/service/parent/rendering.rs) |
| Recursive matrix boundary | [matrix service](../../source-rs/platform/services/matrix/service/mod.rs) | [Parent adaptation](../../source-rs/platform/services/matrix/service/parent/mod.rs), [local world](../../source-rs/platform/services/matrix/world/mod.rs), child grants and input; [IPC fixture](../../source-rs/distribution/lab/matrix/scenario/mod.rs) |
| Interactive recovery lab | [desktop session](../../source-rs/distribution/lab/compositor/supervisor/desktop/mod.rs) | Admission, grants, idle probes and recovery; [client session](../../source-rs/distribution/lab/compositor/client/session/mod.rs) owns desired state and presentation |
| Private storage | [catalog engine](../../source-rs/platform/libraries/object-store/lib.rs) | Root-local validation, [format selection](../../source-rs/platform/libraries/object-store/format/mod.rs), commit and recovery; legacy decoding lives beneath format, protocol staging is a separate entrance |
| Second private client | [counter session](../../source-rs/distribution/applications/counter/session/mod.rs) | Paired record schema, health verification and optional failure probes |
| Kernel exercises | [boot lab](../../source-rs/kernel/boot/uefi/lab/mod.rs) | Mechanism suites own their subordinate lifecycle/watchdog fixtures; guest counterparts live in `distribution/lab` |
| Channel exercises | [suite order](../../source-rs/distribution/lab/channels/scenario.rs) | [Delivery and worker](../../source-rs/distribution/lab/channels/delivery/mod.rs), startup, recursive loans, funded admission; each owns its hostile-input probes |

The ownership rules remain in [repository layout](repository_layout.md).
