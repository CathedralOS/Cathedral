# Owner questions

Only unresolved owner-level Cathedral semantic, compatibility, architecture,
or trust decisions belong here. Settled decisions live in the
[specification](wiki/spec/README.md); implementation work and deliberately
deferred research do not.

Before adding a question:

1. Name the concrete product or implementation requirement that is blocked.
2. Cite the specification owners and machine-readable contracts checked and
   found silent.
3. Separate the owner choice from ordinary engineering or extraction work.
4. State the viable choices, their invariant and compatibility consequences,
   and a recommended answer.

A missing specification page is not by itself an owner question. Neither is an
implementation difficulty. Record design conflicts and oddities as they are
encountered when they expose an owner choice affecting an active requirement
or architecture boundary. Ordinary documentation corrections can be fixed
directly; recommendations here remain proposals until accepted.

Questions are a mutable decision queue, not stable contract identities. Code,
tests, and settled documentation must cite the specification clause produced by
the answer rather than an owner-question number. Resolving a question updates
the specification and affected source contracts in the same change, then
removes the question. Git retains the discussion history.

Existing `Key Questions` and `Open Questions` in design chapters are design
inventory. They are not automatically owner questions under this narrower
standard.

## Open questions

### Replaceable distribution versus permanent OS chrome

**Requirement and conflict.** A fork must be able to replace the single
`distribution/`, including its desktop, as accepted in
[ADR 0002](wiki/decisions/0002-kernel-platform-distribution.md). The
[windowing design](wiki/design/part_6_human_surface/00_windowing_and_compositor.md)
still calls the entire stock OS chrome permanent, always resident and never
replaceable, including the panel, launcher, file browser and wallpaper. Its
new ownership note does not settle what must remain available when a
replacement shell fails. This blocks defining the shell/recovery contract,
not the already accepted folder split.

**Contracts checked.** The
[specification index](wiki/spec/README.md) assigns compositor and seat semantics
to the unwritten `spec/human_surface/compositor_and_seat.md`. Neither the
[Omega distribution profile](source/distribution/profile.json) nor the
[Rust profile](source-rs/distribution/profile.json) defines a shell or recovery
interface; both currently select only early boot.

**Owner choice.** Must the platform retain a complete fallback desktop, or only
a minimal trusted interaction and recovery surface? A complete fallback would
require an independently maintained shell outside the replaceable distribution.
A minimal surface permits replacement of the entire everyday desktop, but must
still provide trusted prompts, observation indicators and operator escape/revoke
behavior when the distribution is absent or broken. The eventual contract must
define what replacement shells may draw over, capture or intercept.

**Recommendation (unaccepted).** Require the minimal platform-owned trusted
surface and recovery path; keep the everyday desktop entirely in
`distribution/`. Specify those guarantees before implementing shell integration,
then reconcile the older permanent-chrome prose with the accepted contract.

### Kernel mechanisms versus platform orchestration

**Requirement and tension.** The kernel/platform split needs a defensible trust
boundary as the boot lab grows. The
[kernel charter](source/kernel/core/CHARTER.md) and
[TCB inventory](wiki/architecture/tcb.md) assign hot-swap, transaction
coordination, trusted loading and attestation to a kernel described as small
enough to audit in full. Those assignments do not distinguish enforcement
primitives from service orchestration. This is an unresolved scope decision,
not evidence that those features are impossible or that a particular line
count is required. The
[kernel architecture design](wiki/design/part_5_lifecycle/04_kernel_architecture.md)
also describes a narrower privileged substrate and restartable services, so the
charter and inventory need an explicit reconciliation with that direction.

**Contracts checked.** ADR 0002 establishes placement and dependency direction,
but does not decide this split. The specification index lists component lifecycle
and admission as unwritten subjects; its current boot and root-extent contracts
do not define these service boundaries. The
[contracts charter](source/contracts/CHARTER.md) names future interfaces without
settling their enforcement/orchestration split.

**Owner choice.** Keep the listed subsystems wholly in the trusted kernel, or
place orchestration in platform services over narrowly defined kernel
mechanisms. The former expands the kernel audit surface. The latter requires
explicit authority, atomicity and failure/recovery contracts across the boundary;
moving code alone neither isolates it nor removes it from the TCB.

**Recommendation (unaccepted).** Default to platform orchestration, retaining
kernel mechanisms wherever required to enforce system invariants. Decide each
subsystem's boundary before implementing it, and update its specification,
contracts, charter and TCB entry together. Track kernel size and total trusted
code separately; do not adopt a 12,000-line target as an architectural constraint.

### Default isolation boundary for proved OS components

**Requirement and tension.** The Rust lab now exercises hardware-contained
tasks as a prerequisite to services outside the kernel. The
[kernel architecture design](wiki/design/part_5_lifecycle/04_kernel_architecture.md)
already places unproved apps and drivers behind hardware boundaries; this
milestone is consistent with that direction. For OS components, however, the
chapter describes proof-based isolation in one address space as the model,
then favors MMU isolation by default with proof-based sharing as an aspirational
option. The [component design](wiki/design/part_2_components/00_component_model.md)
leaves the per-component boundary open. This affects future platform-service
placement and failure containment, not whether the current lab can run ring-3
payloads.

**Contracts checked.** The [specification index](wiki/spec/README.md) assigns
component, capability, IPC and admission semantics to unwritten subjects.
The [Rust boot contract](source-rs/contracts/boot.rs) covers the firmware-neutral
memory inventory, not admission into a shared protection domain. ADR 0002
explicitly makes no userspace-isolation claim for its source grouping.

**Owner choice.** Should proved OS services share a protection domain by default,
use hardware isolation by default, or select a domain through explicit admission
policy? Sharing requires a defined proof/checker assurance threshold and fault
model; MMU separation requires service-call and shared-memory boundaries; a
mixed policy must identify which guarantees each admitted placement provides.
None can infer containment merely from a package name or the language used.

**Recommendation (unaccepted).** Use hardware separation as the initial service
baseline; require an explicit admission contract before sharing a protection
domain. Keep the current Rust syscall ABI experimental. Settle the production
policy before treating the lab's task model as Cathedral's component model.

### Endpoint transport and revocation of pending IPC

**Requirement and tension.** Isolated services need a transport and a defined
outcome when their authority is revoked during a wait. The
[IPC design](wiki/design/part_3_communication/00_ipc_and_service_invocation.md)
names shared regions as the one IPC primitive, with queues and RPC in libraries,
but its endpoint section also describes kernel creation and delivery of opaque
messages across hardware boundaries. It does not settle whether those are two
transports or descriptions of one shared-region protocol. The
[capability lifecycle](wiki/design/part_1_authority/01_capability_lifecycle.md)
says already-redeemed operations run to completion and revocation is discovered
on next use. An empty blocking receive may have no completion unless the contract
defines when redemption commits it and how cancellation or peer death intervenes.

**Contracts checked.** The [specification index](wiki/spec/README.md) assigns
authority and IPC semantics to the unwritten `spec/authority/capabilities.md`
and `spec/communication/ipc.md`. The new
[Rust syscall constants](source-rs/contracts/user.rs) and
[transport model](source-rs/kernel/core/ipc.rs) explicitly define only lab
behavior. No accepted contract settles transport selection or pending-operation
revocation. This blocks promoting the experiment to a stable service interface,
not continued laboratory work.

**Owner choice.** Either require shared-region data movement with kernel endpoints
limited to authority and notification, or retain copied-message delivery as an
additional supported transport with explicit limits and equivalent protocol
semantics. Separately, define whether a parked receive has already redeemed
authority: allowing it to finish preserves the current lifecycle wording but
permits post-revocation delivery; rechecking at delivery cancels the wait but
requires distinguishing admission to a wait from committed message delivery.
Queued messages also need an explicit ownership/commit point so revocation
cannot silently change their promised outcome.

**Recommendation (unaccepted).** Treat copied queues as lab scaffolding until
the transport boundary is specified. Define separate wait, delivery and
completion points; permit cancellation before delivery commits, without
retroactively undoing completed delivery. Reconcile that rule with the lifecycle
chapter before freezing an API. The current Rust experiment rechecks pending
receives, returns `REVOKED`, and discards undelivered queued bytes on revocation;
those are explicit experimental choices, not an accepted production policy.

### Child lifetime when a supervisor fails

**Requirement and tension.** Userspace supervision needs to determine whether a
child survives the loss of its supervisor and who may subsequently control it.
The [component design](wiki/design/part_2_components/00_component_model.md)
requires structured tasks that cannot outlive their owning scope, while leaving
the relationship between component and child restart open. The
[activation design](wiki/design/part_2_components/06_service_activation.md)
anticipates surviving instances and orphans being reattached after an activator
crashes. The [capability lifecycle](wiki/design/part_1_authority/01_capability_lifecycle.md)
revokes delegated authority on parent death and requires explicit transfer to
remove that lifetime coupling. These can coexist, but the surviving child's
owner and authority graph are unspecified; component supervision and task-scope
ownership cannot simply be assumed identical.

**Contracts checked.** The [specification index](wiki/spec/README.md) leaves
component lifecycle/admission and capability contracts unwritten. The
[Rust launch model](source-rs/kernel/core/supervision.rs) and
[userspace wrappers](source-rs/platform/libraries/user-runtime/task.rs) are lab
contracts only. This blocks promising independent service survival or reattachment,
not implementing bounded restart experiments.

**Owner choice.** Bind every supervised child to the supervisor's lifetime,
or distinguish a lifetime owner from the component currently supervising it.
The former guarantees teardown but makes supervisor failure a subtree outage.
The latter permits continuity, but requires an explicit persistent owner,
authority transfer and a checked adoption protocol; rediscovering a PID or
saved handle must not silently grant control or resurrect revoked authority.

**Recommendation (unaccepted).** Keep task scopes structured. Require explicit
lifetime ownership and transfer before an independently owned service can
survive and be adopted by a replacement supervisor. Specify component ownership
separately from task scheduling. The Rust lab currently cancels its owned
children on supervisor exit/fault, without running user destructors; it implements
neither surviving orphans nor production cooperative cancellation.
