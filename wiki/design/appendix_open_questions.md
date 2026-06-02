# Appendix: Open Questions Register

Cross-cutting unknowns that do not belong to a single chapter. Each chapter has
its own `## Open Questions` section for local issues; this register tracks the
big questions whose answers ripple across many chapters. When one is resolved, it
should collapse into a decision recorded in the relevant chapters and be struck
from here.

## Strategic

- **The first wedge is unchosen.** TV box, appliance, kiosk, router, thin client,
  dev board, or a single controlled laptop SKU? The choice sharpens
  [[24_driver_model]], [[25_boot_and_trust_chain]], [[26_kernel_architecture]],
  [[29_media_and_graphics]], [[31_multi_user_and_org_control]], and
  [[36_store_and_economic_control]] dramatically. Until chosen, several chapters
  hedge.
- **How much legacy compatibility, if any.** A real answer to
  [[41_compatibility_and_legacy]] changes the risk profile of nearly every
  contract. Native-only is cleanest; any Linux/POSIX surface risks colonization.

## Architectural

- **Single address space vs. hardware isolation.** Theseus-style language-level
  isolation buys zero-copy IPC and clean hot swap, but interacts with untrusted
  code, the driver model, and the kernel's trusted base. Recurs across
  [[26_kernel_architecture]], [[11_memory_and_persistence]],
  [[15_ipc_and_service_invocation]], [[24_driver_model]].
- **What is the runtime representation of a held capability** — and its
  serialized, at-rest, and over-the-wire forms — such that it stays unforgeable
  and revocable across IPC, reboot, and migration? Drives [[04_capability_lifecycle]],
  [[15_ipc_and_service_invocation]], [[17_distributed_boundary]],
  [[23_updates_and_hot_swap]].
- **Eager vs. lazy revocation**, and whether revoking a capability revokes its
  delegated sub-tree by default. Cost model unknown ([[03_capability_model]],
  [[04_capability_lifecycle]]).
- **What is the unit?** The component family ([[09_component_model]]) deliberately
  splits isolation / restart / swap / authority / scheduling / persistence /
  upgrade. Which of these genuinely need distinct units, and which collapse?

## Proof & Language

- **What must Omega grow before Cathedral can boot at all?** Tracked in
  [[01_omega_substrate]]. Candidates: serialized capabilities, quiescence proofs
  under interrupts/async/hardware, purpose-tagged authority, operation-capabilities
  for secrets.
- **Static authority flow vs. live held grants.** The compiler describes
  *possible* power; the runtime graph holds *actual* grants. How tightly are they
  reconciled, and who flags drift? ([[03_capability_model]],
  [[33_observability_and_introspection]].)
- **How much hot-swap safety is statically provable** vs. load-time/runtime
  checked? ([[21_versioned_state_and_migration]], [[23_updates_and_hot_swap]].)

## Trust & Governance

- **The trusted computing base is the boundary-provider set.** Enumerating,
  minimizing, and auditing it is a standing task across
  [[26_kernel_architecture]], [[24_driver_model]], [[25_boot_and_trust_chain]].
- **Does the OS vendor obey the capability model in practice**, not just in
  principle? The credibility of [[35_telemetry_and_feedback]] depends on it.

## How to use this register

- Add an entry only when the question spans multiple chapters.
- Link the chapters it touches.
- When answered, record the decision in those chapters and remove it here.
