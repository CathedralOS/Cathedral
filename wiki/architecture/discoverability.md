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
| Firmware and kernel startup | [boot main](../../source-rs/kernel/boot/uefi/main.rs) | Firmware handoff, memory, interrupts, supplied-program startup |
| Isolated tasks | [user-task session](../../source-rs/kernel/core/user_tasks/session.rs) | Composition, preparation, admission, execution, outcomes |
| CPU re-entry | [session traps](../../source-rs/kernel/core/user_tasks/session/traps.rs) | Syscall dispatch, wakeups, scheduling and root selection |
| Kernel syscall enforcement | [syscall dispatcher](../../source-rs/kernel/core/user_tasks/session/syscalls.rs) | IPC, tasks, readiness and specific device grants |
| Trusted kernel tasks | [kernel-task API](../../source-rs/kernel/core/kernel_tasks.rs) | Runtime scheduling and stack ownership |
| Distribution startup | [init session](../../source-rs/distribution/init/session.rs) | Provider readiness, application launch and supervision |
| Status application | [application workflow](../../source-rs/distribution/applications/status/application.rs) | Connections, input, state, persistence, view and test probes |
| Kernel exercises | [boot lab](../../source-rs/kernel/boot/uefi/lab.rs) | Individual fixtures; guest counterparts live in `distribution/lab` |

The ownership rules remain in [repository layout](repository_layout.md).
