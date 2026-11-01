# Cathedral work index

Start here when taking over work. These are implementation queues, not the
specification or a claim that every design ambition is implemented.

**Owner direction, 2026-10-26: prioritize Omega-centered implementation.** The
Rust reference has established enough of the OS design to support moving runnable
behavior into `source/`. Use Rust for the host mechanisms, regression fixes and
tests needed by that transition; completing every Rust experiment is not a
prerequisite. Prefer replacing existing desktop behavior with Omega over adding
another Rust-only feature or isolated proof. Keep F5 usable throughout.

| Work | Start here | Source |
| --- | --- | --- |
| Runnable Rust reference OS | [Rust tasks and handoff](TASKS_RUST.md) | `source-rs/` |
| Shared-memory service transport and capability-controlled parking | [IPC and notification outcome](TASKS_RUST.md#shared-memory-ipc-and-notification-outcome) | Kernel mechanisms, platform protocols and ordinary desktop consumers |
| Launch-established Matrix connection and service acquisition | [Matrix connection outcome](TASKS_RUST.md#launch-established-matrix-connection-and-service-acquisition) | Funded bootstrap backing, syscall retrieval and accepted service-resource grants |
| Hierarchical memory budgets and optional allocator interception | [Memory outcome](TASKS_RUST.md#memory-budgets-and-allocator-delegation) | Kernel grant/accounting enforcement and replaceable userspace allocation policy |
| Drawing commands, images and resource availability | [Rendering outcome](TASKS_RUST.md#drawing-images-and-resource-availability) | Platform renderer, caller-funded backing and remote-host imports |
| Shared-buffer audio playback | [Audio outcome](TASKS_RUST.md#shared-buffer-audio-playback) | Platform mixer, funded sample backing and audio driver |
| Scoped cursor assets without origin disclosure | [Cursor outcome](TASKS_RUST.md#cursor-presentation-without-upstream-disclosure) | Supplied input/cursor interfaces and authorized final rendering |
| Object-rooted watches and durable capture | [Watch outcome](TASKS_RUST.md#object-rooted-watches-and-durable-capture) | Platform storage service, caller-funded delivery and transactional history |
| Explicit atomic storage batches | [Batch outcome](TASKS_RUST.md#explicit-atomic-storage-batches) | Platform storage service, revision checks and recoverable commit outcomes |
| Reference deletion and scoped erasure | [Deletion outcome](TASKS_RUST.md#reference-deletion-and-scoped-erasure) | Storage retention policy, independent copies and explicit guarantee refusal |
| Independent installation and component-owned activation | [App update outcome](TASKS_RUST.md#independent-installation-and-component-owned-activation) | Platform updater, scoped lifecycle owners and immutable executable backing |
| Moving distribution behavior to Omega | [Current migration and next matrix step](TASKS_RUST.md) | `source/distribution/applications/`, `source/distribution/matrix/`, [independent Omega matrix](source/distribution/lab/matrix-composition/README.md) |
| Stock matrix in Omega | [F5 desktop test](TASKS_RUST.md#stock-matrix-migration-returned-reservations-and-the-f5-desktop-test), [App entries and selection](TASKS_RUST.md#stock-matrix-migration-the-chromes-app-entries-and-selection), [Funding and parity](TASKS_RUST.md#stock-matrix-migration-funding-and-the-f5-parity-measure), [Startup](TASKS_RUST.md#stock-matrix-migration-startup-from-the-realm), [Nested](TASKS_RUST.md#stock-matrix-migration-nested-operation-under-a-parent), [Child launch](TASKS_RUST.md#stock-matrix-migration-a-childs-launch), [Realm launches](TASKS_RUST.md#stock-matrix-migration-launches-from-the-realms-objects), [Storage and presentation](TASKS_RUST.md#stock-matrix-migration-storage-root-and-presentation), [Children](TASKS_RUST.md#stock-matrix-migration-children-at-the-display-root), [Display root](TASKS_RUST.md#stock-matrix-migration-the-display-root-in-omega) | [Standalone stock matrix process](source/distribution/matrix/process/README.md) run by the F5 shell as the root and as a user's matrix, which configures its workspace from its realm under the stock startup policy: owner control and routed owner words, display or parent, source or snapshots, children run from the catalogue, from realm objects or by the launcher's launch under the stock admission, import and resource policy, their surfaces, images and batches, their storage relayed under scopes and launch receipts; funding under the stock policy, the workspace's app entries and selection; the chrome's remaining words, surveys, recovery and input capacity still in the Rust host |
| Omega World | [Delegation](TASKS_RUST.md#omega-world-layout-control-management-and-launch-observations), [Input model](TASKS_RUST.md#omega-world-the-input-model), [Surfaces and demand](TASKS_RUST.md#omega-world-surfaces-and-frame-demand), [World outcome](TASKS_RUST.md#omega-world-and-the-standalone-independent-matrix) | [Grants, placement, input, surfaces, demand, layout control, management and launch observations in Omega](source/platform/libraries/matrix/world/README.md); the independent matrix runs standalone on it; presentation batches and storage scopes remain in Rust |
| Independent Omega service clients | [Service runtime outcome](TASKS_RUST.md#omega-service-runtime-independent-clients-and-retained-requests) | Shared acceptance, sealed request custody, reply tickets and parking; real multi-client fault tests |
| Standalone Omega userspace | [Process runtime](source/platform/libraries/user-runtime/README.md) | Installed chooser, Background, Colors and Notes run as standalone Omega processes, including shared IPC, owner control, storage and images; the matrix retains its Rust host |
| Omega child admission and service offers | [Admission outcome](TASKS_RUST.md#omega-child-admission-and-service-offers) | Lent launches, brokered admission with rollback, endpoint and wait-right replies, reclamation; proven by the standalone supervisor witness |
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
