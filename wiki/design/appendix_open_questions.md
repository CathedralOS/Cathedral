# Appendix: Open Questions Register

Cross-cutting unknowns that do not belong to a single chapter. Each chapter has its own `## Open Questions` section for local issues; this register tracks the big questions whose answers ripple across many chapters. When one is resolved, it should collapse into a decision recorded in the relevant chapters and be struck from here.

## Scope

- **How much legacy compatibility, if any.** A real answer to [[compatibility_and_legacy]] changes the risk profile of nearly every contract. Native-only is cleanest; any Linux/POSIX surface risks letting a legacy contract become Cathedral's contract.

## Architectural

- **Single address space vs. hardware isolation.** Theseus-style language-level isolation buys zero-copy IPC and clean hot swap, but interacts with untrusted code, the driver model, and the kernel's trusted base. Recurs across [[kernel_architecture]], [[memory_and_persistence]], [[ipc_and_service_invocation]], [[driver_model]].
- **What is the runtime representation of a held capability** — and its serialized, at-rest, and over-the-wire forms — such that it stays unforgeable and revocable across IPC, reboot, and migration? Drives [[capability_lifecycle]], [[ipc_and_service_invocation]], [[distributed_boundary]], [[updates_and_hot_swap]].
- **Eager vs. lazy revocation**, and whether revoking a capability revokes its delegated sub-tree by default. Cost model unknown ([[capability_model]], [[capability_lifecycle]]).
- **Replaceable-realization representation.** The semantic unit is settled: a
  selected provider realization plus its owned code/state/resource closure,
  distinct from package, boundary, task, and principal. Cathedral still must
  choose its binding-era algorithm, live-era bound, replacement/disposition
  receipts, mapping-cohort manifest, stack-provision strategy, and migration/
  cancellation interfaces ([[component_model]], [[updates_and_hot_swap]]).

## Proof & Language

- **Remaining Omega OS-foundation contracts.** `Extent` is linear ordinary data
  carrying base and `u64` length; abstract domain evidence carries grant,
  address-space, rights, and provenance facts. Cathedral's platform provider
  now originates its first `Granted` extent under an admitted provider-plan
  receipt and carries it linearly into owned idle. The settled qualification
  also requires proof that embedded `base + length` fits the target address
  space, and its owner-unique content projection uses proof-level natural
  interval bounds. Physical-space and rights qualification plus checked
  transformations that conserve the claim remain.
  Bodyless/boundary-domain spelling, exact boundary-result authorization,
  transparent declared-domain aliases, and state-local constrained-parameter
  obligations are implemented; package-owner coherence,
  compiler-owned per-claim carry atoms, and resource-frontier inference still
  need implementation. Accepted resource claims begin with strict carry
  and gain positive per-claim permissions through their provider contracts. The
  placed-view public model, demand-driven carry admission, fixed-stack task
  model, and admitted executable loader ladder are otherwise specified.
  Checked-Omega return integrity is settled as a consequence of memory safety
  plus non-addressable live stack/control state. Local `dyn Trait` dispatch is
  settled as an artifact-local selected-conformance descriptor and cannot cross
  a replaceable boundary. External callback entry is settled as a named static
  machine satisfying a `Calling<C>` boundary requirement, with plan-driven
  thunk lowering and linear durable registration. The Windows/platform adapter
  canary and exact target stack policies remain implementation work
  ([[hardware_foundation_profile]]); component calls use boundary bindings and
  the era protocol in [[updates_and_hot_swap]].
- **Fuel, fixed work, and foreign-entry completion.** WCSU answers stack
  space, not work or response latency. Cathedral depends on Omega's canonical
  IR fuel meter, restricted fixed-work checking for hard roots and safe-point
  segments, and attributed unbounded-edge reporting. Same-stack firmware calls
  use admitted ceilings. A hosted blocking executor, where needed, is an
  ordinary package with separately provisioned worker stacks, bounded queues,
  and attributable exhaustion ([[hardware_foundation_profile]],
  [[scheduler_and_resources]]).
- **Formal atomics and modular protocols.** Cathedral's mailbox, scheduler, and
  reclamation proofs depend on Omega defining its complete atomic-event model,
  proving the x86-64/AArch64 mappings, and settling separately compiled
  environment premises in Omega owner question #1. Existing instruction
  selection is not yet that proof.
- **Static authority flow vs. live held grants.** The compiler describes *possible* power; the runtime graph holds *actual* grants. How tightly are they reconciled, and who flags drift? ([[capability_model]], [[observability_and_introspection]].)
- **Hot-swap evidence split.** Omega statically supplies closure, contract,
  representation, resource-demand, and liveness facts. Cathedral still must
  define which era, device, health, drain, transfer, and reclamation facts are
  provider receipts versus runtime-ledger observations
  ([[versioned_state_and_migration]], [[updates_and_hot_swap]]).

## Hardware Profile

- **Privileged broker split.** How are address-space mapping, placed-view minting, admitted-artifact installation, interrupt installation, and IOMMU control divided without duplicating admission or inflating one universal broker? ([[hardware_foundation_profile]], [[kernel_architecture]].)
- **Executable installation target.** Which AP/I-cache/W^X guarantees are mandatory on the first real platform, which are honestly reported as unavailable or convention-only, and how does the provider distinguish dormant/local installation, a future remote fetcher needing visibility, and cores that may already be running the range and therefore require replacement? ([[hardware_foundation_profile]], [[boot_and_trust_chain]].)
- **Device reset domains.** Where FLR is unavailable or bus-wide, how much sibling disruption is acceptable before Cathedral marks a device/domain dead? ([[driver_model]], [[updates_and_hot_swap]].)

## Trust & Governance

- **The trusted computing base is the boundary-provider set.** Enumerating, minimizing, and auditing it is a standing task across [[kernel_architecture]], [[driver_model]], [[boot_and_trust_chain]].
- **Is system/OS-level code itself bound by the capability model**, or does it sit outside it? Whether observation and data collection are subject to the same authority machinery as everything else is a structural question, not a policy one ([[telemetry_and_feedback]]).

## How to use this register

- Add an entry only when the question spans multiple chapters.
- Link the chapters it touches.
- When answered, record the decision in those chapters and remove it here.
