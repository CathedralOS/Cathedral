# Appendix: Open Questions Register

Cross-cutting unknowns that do not belong to a single chapter. Each chapter has its own `## Open Questions` section for local issues; this register tracks the big questions whose answers ripple across many chapters. When one is resolved, it should collapse into a decision recorded in the relevant chapters and be struck from here.

## Scope

- **How much legacy compatibility, if any.** A real answer to [[compatibility_and_legacy]] changes the risk profile of nearly every contract. Native-only is cleanest; any Linux/POSIX surface risks letting a legacy contract become Cathedral's contract.

## Architectural

- **Single address space vs. hardware isolation.** Theseus-style language-level isolation buys zero-copy IPC and clean hot swap, but interacts with untrusted code, the driver model, and the kernel's trusted base. Recurs across [[kernel_architecture]], [[memory_and_persistence]], [[ipc_and_service_invocation]], [[driver_model]].
- **What is the runtime representation of a held capability** — and its serialized, at-rest, and over-the-wire forms — such that it stays unforgeable and revocable across IPC, reboot, and migration? Drives [[capability_lifecycle]], [[ipc_and_service_invocation]], [[distributed_boundary]], [[updates_and_hot_swap]].
- **Eager vs. lazy revocation**, and whether revoking a capability revokes its delegated sub-tree by default. Cost model unknown ([[capability_model]], [[capability_lifecycle]]).
- **What is the unit?** The component family ([[component_model]]) deliberately splits isolation / restart / swap / authority / scheduling / persistence / upgrade. Which of these genuinely need distinct units, and which collapse?

## Proof & Language

- **Omega OS-foundation carriers.** The primitive split is settled, but Omega still must settle the public `Extent`/placed-view API, carry-contract spelling, and executable-publication evidence. Cathedral's vertical slices are the deciding customers ([[hardware_foundation_profile]]); the matching owner questions live in Omega's `OWNER_QUESTIONS.md`.
- **Static authority flow vs. live held grants.** The compiler describes *possible* power; the runtime graph holds *actual* grants. How tightly are they reconciled, and who flags drift? ([[capability_model]], [[observability_and_introspection]].)
- **How much hot-swap safety is statically provable** vs. load-time/runtime checked? ([[versioned_state_and_migration]], [[updates_and_hot_swap]].)

## Hardware Profile

- **Privileged broker split.** How are address-space mapping, placed-view minting, executable publication, interrupt installation, and IOMMU control divided without duplicating admission or inflating one universal broker? ([[hardware_foundation_profile]], [[kernel_architecture]].)
- **Interrupt stack/nesting policy.** Pick Cathedral's initial x86 stack classes, masking/preemption graph, and WCSU composition, then derive the concrete IST fields from that one normalized policy ([[hardware_foundation_profile]]).
- **Executable publication target.** Which AP/I-cache/W^X guarantees are mandatory on the first real platform, which are honestly reported as unavailable or convention-only, and how does the provider distinguish dormant future executors from cores that may already be running the range? ([[hardware_foundation_profile]], [[boot_and_trust_chain]].)
- **Device reset domains.** Where FLR is unavailable or bus-wide, how much sibling disruption is acceptable before Cathedral marks a device/domain dead? ([[driver_model]], [[updates_and_hot_swap]].)

## Trust & Governance

- **The trusted computing base is the boundary-provider set.** Enumerating, minimizing, and auditing it is a standing task across [[kernel_architecture]], [[driver_model]], [[boot_and_trust_chain]].
- **Is system/OS-level code itself bound by the capability model**, or does it sit outside it? Whether observation and data collection are subject to the same authority machinery as everything else is a structural question, not a policy one ([[telemetry_and_feedback]]).

## How to use this register

- Add an entry only when the question spans multiple chapters.
- Link the chapters it touches.
- When answered, record the decision in those chapters and remove it here.
