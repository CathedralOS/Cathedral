# Cathedral work index

Start here when taking over work. These are implementation queues, not the
specification or a claim that every design ambition is implemented.

| Work | Start here | Source |
| --- | --- | --- |
| Runnable Rust reference OS | [Rust tasks and handoff](TASKS_RUST.md) | `source-rs/` |
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
