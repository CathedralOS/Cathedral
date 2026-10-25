# Cathedral work index

Start here when taking over work. These are implementation queues, not the
specification or a claim that every design ambition is implemented.

| Work | Start here | Source |
| --- | --- | --- |
| Runnable Rust reference OS | [Rust tasks and handoff](TASKS_RUST.md) | `source-rs/` |
| Shared-memory service transport and capability-controlled parking | [IPC and notification outcome](TASKS_RUST.md#shared-memory-ipc-and-notification-outcome) | Kernel mechanisms, platform protocols and ordinary desktop consumers |
| Object-rooted watches and durable capture | [Watch outcome](TASKS_RUST.md#object-rooted-watches-and-durable-capture) | Platform storage service, caller-funded delivery and transactional history |
| Explicit atomic storage batches | [Batch outcome](TASKS_RUST.md#explicit-atomic-storage-batches) | Platform storage service, revision checks and recoverable commit outcomes |
| Independent installation and component-owned activation | [App update outcome](TASKS_RUST.md#independent-installation-and-component-owned-activation) | Platform updater, scoped lifecycle owners and immutable executable backing |
| Moving distribution behavior to Omega | [Current migration and next matrix step](TASKS_RUST.md) | `source/distribution/applications/`, `source/distribution/matrix/`, [independent matrix proof](source/distribution/lab/matrix-composition/README.md) |
| Omega kernel boot | [Current boot frontier](source/kernel/boot/uefi/README.md) | `source/kernel/` |
| Translating upstream Rust OS libraries into Omega | [Omega porting queue](TASKS_OMEGA_PORTS.md) | `source/`, verification under `tools/ports/` |
| Unresolved owner-level semantics or trust choices | [Owner questions](OWNER_QUESTIONS.md) | Resolve into the owning specification/contract |

The Rust implementation lets us exercise OS mechanisms and a usable distribution
while Omega develops. The porting queue preserves upstream hardware knowledge,
algorithms and contracts in Omega. Finishing a port slice does not imply that it
runs in either booted OS. Follow each slice's recorded verification scope.

For the recent desktop work, continue from **TASKS_RUST.md**. F5 must remain the
normal route to one OS desktop where the user chooses programs. Focused fault and
mechanism tests remain useful, but a new lab alone does not make functionality
available in that desktop.

Read [AGENTS.md](AGENTS.md), the [ownership rules](wiki/architecture/repository_layout.md)
and [discoverability guide](wiki/architecture/discoverability.md) before changing
composition. Keep task queues current when completing work; put detailed contracts
and subsystem explanations with their existing owners rather than duplicating them
here. Proposed priorities are not settled owner decisions.
