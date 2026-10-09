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
count is required.

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
