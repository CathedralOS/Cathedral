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
- An implementation file gains a same-named directory when it has distinct
  responsibilities to descend into: `session.rs` and `session/`, or `view.rs`
  and `view/`. Keep a coherent leaf together; line counts are not architectural
  boundaries. Avoid chains of files that only forward to the next file.
- Folder names identify ownership or purpose. Prefer `kernel_tasks`, `user_tasks`
  and `lab` over ambiguous peers such as `tasks`, `users` and `programs`.
  A generic `runtime`, `helpers` or `common` must not accumulate unrelated work.
- Group supporting packages beneath a real subsystem as that subsystem grows.
  Preserve independently owned libraries, contracts and Cargo boundaries; folder
  grouping is not permission to absorb them into a coordinator.
- Small entrance comments name the mechanism, its important invariant and the
  next owners. Documentation supplements the navigable implementation.

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
| CPU turn sharing | [Task scheduling](../../source-rs/kernel/core/scheduler.rs) | [Hierarchical selection](../../source-rs/kernel/core/scheduler/shares.rs), [contract and proof route](../../source-rs/kernel/core/scheduler/README.md), [loan reattachment](../../source-rs/kernel/core/user_tasks/session/lifecycle/delegation.rs) |
| CPU time ceilings | [Persistent account charging](../../source-rs/kernel/core/scheduler/budget.rs) | [Architecture clock](../../source-rs/kernel/arch/x86_64/cpu_clock.rs), [trap samples](../../source-rs/kernel/core/user_tasks/session/traps.rs), [measured QEMU proof](../../source-rs/kernel/boot/uefi/lab/supervision/cpu/budget.rs) |
| Firmware and kernel startup | [boot main](../../source-rs/kernel/boot/uefi/main.rs) | Firmware handoff, memory, interrupts, supplied-program startup |
| Isolated tasks | [user-task session](../../source-rs/kernel/core/user_tasks/session.rs) | Composition, preparation, admission, execution, outcomes |
| CPU re-entry | [session traps](../../source-rs/kernel/core/user_tasks/session/traps.rs) | Syscall dispatch, wakeups, scheduling and root selection |
| General IPC | [IPC coordinator](../../source-rs/kernel/core/ipc.rs) | [Process-local grants](../../source-rs/kernel/core/ipc/grants.rs), [creation and transfer](../../source-rs/kernel/core/ipc/channels.rs), [teardown](../../source-rs/kernel/core/ipc/lifecycle.rs); [ordinary-worker proof](../../source-rs/distribution/lab/channels/scenario.rs) |
| IPC reservations | [Account limits and charges](../../source-rs/kernel/core/ipc/budget.rs) | [Admission validation](../../source-rs/kernel/core/ipc/bootstrap.rs), [final closure](../../source-rs/kernel/core/ipc/lifecycle.rs), [restart and sibling proof](../../source-rs/distribution/lab/channels/sponsorship.rs) |
| Child startup environment | [Launch staging](../../source-rs/kernel/core/user_tasks/session/syscalls/tasks/launch.rs) | [Admission and commit ordering](../../source-rs/kernel/core/user_tasks/session/lifecycle.rs), [bundle validation and installation](../../source-rs/kernel/core/ipc/initial.rs), [userspace Environment](../../source-rs/platform/libraries/user-runtime/task/environment.rs) |
| Recursive admission | [Launch syscall staging](../../source-rs/kernel/core/user_tasks/session/syscalls/tasks/launch.rs) | [Loan validation and commit](../../source-rs/kernel/core/user_tasks/session/lifecycle/delegation.rs), [loan lifetime](../../source-rs/kernel/core/supervision/delegation.rs), [same-code recursive proof](../../source-rs/distribution/lab/channels/tree/node.rs) |
| Caller-local kernel names | [Name allocation](../../source-rs/kernel/core/names.rs) | [Admission reservations](../../source-rs/kernel/core/user_tasks/session/lifecycle/grants.rs), [loan return](../../source-rs/kernel/core/supervision/delegation.rs), [local region mappings and offers](../../source-rs/kernel/core/regions.rs) |
| Kernel syscall enforcement | [syscall dispatcher](../../source-rs/kernel/core/user_tasks/session/syscalls.rs) | IPC, tasks, readiness and specific device grants |
| Trusted kernel tasks | [kernel-task API](../../source-rs/kernel/core/kernel_tasks.rs) | Runtime scheduling and stack ownership |
| Distribution startup | [init session](../../source-rs/distribution/init/session.rs) | Provider readiness, application launch and supervision |
| Status application | [application workflow](../../source-rs/distribution/applications/status/application.rs) | Connections, input, state, persistence, view and test probes |
| Display protocol selection | [display main](../../source-rs/platform/services/display/main.rs) | [Immediate drawing](../../source-rs/platform/services/display/direct.rs) or [retained compositor](../../source-rs/platform/services/display/compositor.rs); both use the framebuffer surface |
| Retained rendering | [composition state](../../source-rs/platform/services/display/composition.rs) | Row composition order, [scene snapshots and hosting](../../source-rs/platform/services/display/composition/scene.rs), [input state and focus](../../source-rs/platform/services/display/composition/input.rs) |
| Physical input | [input main](../../source-rs/platform/services/input/main.rs) | [Keyboard service](../../source-rs/platform/services/input/keyboard.rs) or [compositor source](../../source-rs/platform/services/input/pointer.rs), shared controller and pure decoders |
| Storage service | [storage main](../../source-rs/platform/services/storage/main.rs) | [Disk/catalog startup and request dispatch](../../source-rs/platform/services/storage/service.rs), with recovery hooks beneath the service |
| Compositor input | [compositor loop](../../source-rs/platform/services/display/compositor.rs) | [Connection authorization](../../source-rs/platform/services/display/compositor/input.rs), [pointer lifetime](../../source-rs/platform/services/display/composition/input.rs), routing and cursor damage |
| Focus and hosted scenes | [session dispatch](../../source-rs/platform/services/display/compositor/session.rs) | [Focus attenuation](../../source-rs/platform/services/display/composition/input/focus.rs), [hosting and clips](../../source-rs/platform/services/display/composition/scene/hosting.rs), deferred event delivery |
| Integrated matrix desktop | [Distribution policy](../../source-rs/distribution/lab/matrix/desktop.rs) | [Matrix service loop](../../source-rs/platform/services/matrix/service.rs), [child admission](../../source-rs/platform/services/matrix/service/children/admission.rs), [local surface clipping](../../source-rs/platform/services/matrix/world/surface.rs), [root compositor adapter](../../source-rs/platform/services/matrix/service/parent/rendering.rs) |
| Recursive matrix boundary | [matrix service](../../source-rs/platform/services/matrix/service.rs) | [Parent adaptation](../../source-rs/platform/services/matrix/service/parent.rs), [local world](../../source-rs/platform/services/matrix/world.rs), child grants and input; [IPC fixture](../../source-rs/distribution/lab/matrix/scenario.rs) |
| Interactive recovery lab | [desktop session](../../source-rs/distribution/lab/compositor/supervisor/desktop.rs) | Admission, grants, idle probes and recovery; [client session](../../source-rs/distribution/lab/compositor/client/session.rs) owns desired state and presentation |
| Private storage | [catalog engine](../../source-rs/platform/libraries/object-store/lib.rs) | Root-local validation, encoding, commit and recovery; protocol staging is a separate entrance |
| Second private client | [counter session](../../source-rs/distribution/applications/counter/session.rs) | Paired record schema, health verification and optional failure probes |
| Kernel exercises | [boot lab](../../source-rs/kernel/boot/uefi/lab.rs) | Individual fixtures; guest counterparts live in `distribution/lab` |

The ownership rules remain in [repository layout](repository_layout.md).
