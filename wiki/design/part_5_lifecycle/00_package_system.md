# Chapter 00: Package & Component System

> A package is a proof-carrying object, not a tarball plus scripts. Installing it is a declarative, checked state transition, never code execution.

## The Legacy Model

Linux packaging is the clearest example of the problem. A package is an archive plus a set of imperative hooks (`preinst`, `postinst`, `%post`, `pkg_postinst`) that run as root with full ambient authority. Install is arbitrary code execution. Around that core sit dependency solvers, dynamic-linker search paths, signature schemes, trust databases, and per-distro policy, and none of them can constrain the one thing that matters: what the postinstall script actually does. The system cannot answer "what authority does this package require, what state does it own, is this upgrade compatible, and can I revoke it?" because none of that was ever declared. It was buried in a shell script.

## The Cathedral Model

Installation is a **declarative state transition**, not a program. A package is an Omega artifact that declares everything the system needs to admit it, and the compiler and loader check the declaration before anything is written. There is no install-script escape hatch. The manifest is the only way in.

A package declares exports and imports; capabilities required, stored, and delegated; protocols spoken; persistent state schemas and migration functions; candidate resource demands and optional fixed policy budgets; upgrade-compatibility facts; security invariants; and the test and proof artifacts that back them. Runtime opaque values additionally publish their normalized carrier class, introduction contracts, backing and provider requirements, and reachable authority. Build provenance is signed and carries an input-reproducibility classification. Dependencies are pinned by content identity. Because all of this is structured, installation reduces to verifying provenance, checking policy, validating state migration and component representations, provisioning the selected realization, and committing or refusing. No ambient install script runs.

This is the core divergence. Every other OS treats the package as an opaque payload whose effects are discovered only by running it. Cathedral treats it as a checkable contract whose blast radius is computed in advance.

## The decided mechanics

The thesis above is the framing: declarative, checkable, no scripts. The mechanics below are how install and update run. Most of the mechanics are prior art. Delta updates, A/B slots, atomic snapshots, and content-addressed generations all ship in macOS (the sealed System Volume), Android and ChromeOS (A/B), and NixOS and Silverblue (generations). Cathedral's contribution is their composition with four distinguishing pieces: content-addressed chunked closures, proof-carrying components re-checked on update, capability manifests (declarative install, no scripts), and versioned interfaces (so partial updates stay valid).

### Fetch is a chunk-level delta

A package is a content-addressed closure, and content-addressing is at the chunk level, not the whole object. Files are split by **content-defined chunking**: a rolling hash picks boundaries, so an insertion re-syncs after the edit instead of shifting every later chunk. Each chunk is addressed, and a file is a Merkle manifest of chunk hashes. A 10 KB patch to a 2 GB file fetches only the touched chunks, tens to hundreds of KB rather than the file. Chunks dedupe globally across components, versions, and apps, in the casync and borg style. A new OS version pulls only the closures, and within them only the chunks, whose hashes differ.

Dependencies are content-addressed, chunked, and pinned. Versions coexist by hash, so there is no solver and no dependency hell. Whether the complete source graph is hermetic and reproducible follows the recorded build-input classes, not content addressing alone.

### What is checked at build and what at install

The manifest is the authority-flow report (accepts, uses, stores, derives, acquires) plus state schemas, protocols, upgrade-compatibility facts, and proof certs. Proofs are proven at build and re-checked at install. Compatibility, meaning schema, protocol, and manifest delta, is checked at install against the live system.

Admission of authority evidence is split between the two sides. Omega validates public data shape, checked claim transformations, and boundary-domain evidence. Cathedral admits the final artifact's transitive trust and authority expansion. Policy selects who may receive authority and never substitutes for the contract constraining its later use.

The store records every published version's schemas, so admission refuses a schema-breaking update. Durable and wire-selected formats are the strict case, because data outlives its writers.

### The update pipeline and its gates

An update stages additively and then passes checked gates before any commit. New closures are written beside the old, nothing is overwritten, and the machine keeps running the old version. Each gate refuses cleanly rather than half-applying:

- **Integrity.** A chunk whose hash mismatches is rejected by construction.
- **Attestation and proof re-check.** The closure signature verifies and the local checker re-verifies the proof certs. You never trust the publisher's "it's proven", so a forged proof dies here (Thompson-resistance).
- **Manifest delta.** A widened ceiling, meaning authority the old version did not declare, requires fresh consent. It cannot land silently.
- **Opaque-representation delta.** Inline/handle changes, new introduction contracts, mutable carrier access, admitted backing, generation/revocation, and their transitive authority consequences are compared from normalized manifests. New privileged reach requires policy approval.
- **Migration totality.** A state-schema change ships a migration proven total. Absent that, the swap is refused and the old state is left intact.

A refusal at any gate leaves you on the working old version with nothing half-installed.

### Live-patch versus reboot is proof-gated, with reboot as the safe fallback

Whether a component, or the core, can be updated in place is proven, not guessed. You prove live-patchability or you reboot. There are three checkable gates:

1. **Quiescence.** No entrant can reach the retiring era, and every existing activation is drained, validly cancelled, migrated, or retained. A promise of bounded drain requires a declared semantic safe point plus bounded work and finite wait ceilings on every path. Architectural timer preemption alone does not establish quiescence ([[scheduler_and_resources]], [[updates_and_hot_swap]]).
2. **Total migration.** The new version's state migration is proven total over the old committed state.
3. **Containment.** The change touches only the component's own code and `data`, not a foundational representation every live thing embodies. A normal component cannot change the substrate it is built on, so its updates are contained by construction. The reboot class is a change to a core foundational primitive whose in-memory layout changed: the memory model (MMU/SAS/CHERI), the capability-arena format, the task/stack representation, the IPC region layout, the trap-vector model. The gate detects whether a foundational layout changed, as opposed to only code or policy.

If all three pass, the update is a live hot-swap. If any fails or is absent, the update is a staged reboot. A staged reboot is always correct, so an unsure classification defaults to reboot safely. Changing the memory model fails gate 3 by inspection, since its purpose is to re-lay-out what all live state embodies ([[kernel_architecture]]).

### A/B applies on the next natural reboot, never forced

Most updates are services and drivers, which live hot-swap with zero reboot ([[updates_and_hot_swap]]). A small core fix can publish a new binding era once entrants are redirected and every member of the old population has a disposition. It is not authorized merely because the timer can stop execution. Only a wholesale core replacement, meaning a foundational-layout change, needs the A/B path. Even then it stages into the inactive slot and applies on the next reboot the user takes anyway, with automatic fallback to the old slot if the new one fails to boot-measure-good ([[boot_and_trust_chain]]). There is no forced reboot, at most a nudge for a critical security fix. Reboots are rare, only for deep core rewrites, and never imposed.

### Many components update independently, with no OS-wide two-phase commit

Each replacement publishes a new requirement-binding era and then drains or retains the old era. Publication and reclamation are separate completion states. Across components, updates proceed independently when their versioned interfaces permit a mixed-version state. A coupled breaking change is made compatible across one step, or uses a coordinated commit scoped to the coupled set. There is never an OS-wide two-phase commit. A multi-component feature that must appear atomically is deployed incrementally and enabled by a final configuration transition.

### The setup-transition: a confined computation, not a script

First-run setup is the one place imperative code wants back in: seed a database, build an initial index. Not everything is purely declarative, so Cathedral gives this a **confined setup machine**, not a `postinstall` script. It is a computation `{ fresh realm } + { explicitly-granted input capabilities } → initial-state value`, run with no ambient authority and no side effects. It holds a write-cap to its own fresh blank realm plus whatever inputs it was granted by gesture or manifest, such as reading a prior install's data to import or querying hardware to detect devices. It produces the initial state as a value committed atomically. Side-effect setup, such as registering a service or a file association, is declarative: the manifest declares it and the activator registers it ([[service_activation]]). It is never code. The postinstall hole closes because setup can only compute a value from granted inputs, never act on the system.

### Migration with extra inputs is the same shape: Omega's `capture` plus pure `upgrade`

A migration needing external information captures an owned typed context first, while the old state remains recoverable. An ordinary checked migration machine then transforms `(old, context)` into new state under deterministic contracts. A package may expose an `Upgradable<Old, New, Context>` convenience trait, but Omega does not bless that trait or a migration DSL. The semantic requirement is the capture-before-point-of-no-return discipline described in [Omega Versioned Data](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md#versioned-data).

### Foreign and native components

The package manager rejects opaque native execution by default. A legacy package admitted by policy is a walled closure with an attested-but-unproven manifest, contained-not-trusted ([[compatibility_and_legacy]]). It cannot prove quiescence, so its updates are restart/reboot-class, not live hot-swap. An uncontained opaque in-process provider makes its address-space manifest incomplete and is unavailable to Cathedral safety profiles.

## Concerns & Design Space

- **No install scripts, ever.** Any per-install side effect must be expressed as a declared state transition the loader executes under a bounded capability set, not as code the package supplies. This is the load-bearing principle of the chapter.
- **Capability manifest.** What the package `accepts`, `uses`, `stores`, `derives`, and `acquires` (from [[capability_model]]) is the heart of the manifest. No ambient filesystem, network, or device access during install.
- **Final-artifact admission.** Policy compares the transitive reachable-authority set after exact dependency resolution, not only direct package declarations. A dependency path identifies who introduced every new privileged reach.
- **Executable TCB manifest.** Provider selection independently contributes exact known executable identities, static versus Cathedral-mediated runtime origin, implementation evidence, execution scope, and separately evidenced memory/termination/fault/resource containment. The manifest states whether its known entry set is complete for that scope and attributes every cause of incompleteness. Platform baselines are policy allowlists, not hidden exceptions.
- **Severity-ranked review.** The complete normalized manifest remains machine-readable. Human diffs collapse checked local inline tokens and make system authority, admitted foreign providers, boundary-domain evidence, provider-owned backing, and revocation machinery loud or blocking.
- **State schemas & migration.** A package owns immutable historical state shapes; upgrades add envelope cases and checked conversion/replacement machines (see [[versioned_state_and_migration]]).
- **Atomic install / uninstall.** Commit or refuse as one transaction ([[transactions_and_consistency]]). No half-installed state, no orphaned hooks.
- **Receipted builds + signed provenance.** Hermetic evaluation is deterministic over source and target. `build.omg` may separately use scoped host providers; each reached operation records a receipt classified as hermetic, content-replayable, or volatile. Release policy checks the statically reachable class before execution and records the realized class and receipts. Graph-wide reproducibility from source additionally requires every dependency artifact to have been produced without volatile inputs ([[audit_compliance_provenance]]).
- **Hermetic realization closure.** A package is not automatically a component. When Cathedral selects a provider realization for independent deployment, the compiler validates its closed code/state/resource graph and its requirement-bound imports. There is no ambient dynamic-library search path. Identical immutable cohorts may still deduplicate by content address ([[filesystem_as_database]]).
- **Machine-checkable compatibility.** Each installed channel or store publishes its directional compatibility demand together with selected schema and codec identities. The distribution plane retains those demands and every published historical shape, so admission checks a candidate against the actual rolling, relay, or persistence horizon it must satisfy ([[store_and_economic_control]]). Storage formats commonly demand the longest readable history because on-disk data can outlive every implementation that wrote it ([[versioned_state_and_migration]]).
- **Revocation & staged rollout.** Package-level revocation, staged/canary rollout, and rollback are built in and gated by the store/control plane ([[store_and_economic_control]]).
- **Zero value.** A zero package is the empty manifest, valid-empty. It declares no exports and an empty capability manifest, so it requires zero authority and installs as a no-op. That is both the least-privilege admission case and ZII-coherent ([[omega_substrate]]).

## Key Questions

- What does the manifest contain, and what is checked at build versus at install? Proofs are proven at build and re-checked at install; compatibility is checked at install against the live system.
- Who admits authority evidence? Omega validates the evidence shape and transformations; Cathedral admits the final artifact's transitive authority expansion.
- How does a declarative install express legitimate side effects? Registrations are declarative and the activator performs them; computational setup is a confined setup machine that maps a fresh realm plus granted inputs to an initial-state value.
- What is the dependency model? Content-addressed, chunked, and pinned, with versions coexisting by hash and fetch as a chunk-level delta.
- How is ABI stability enforced? The store records every published schema, and admission refuses a schema-breaking update.

## Omega Leverage

- **Authority-flow inference** produces the capability manifest directly. The accepts/uses/stores/derives/acquires report is what the package declares.
- **Authority-evidence manifests** expose public authority-value shape, boundary-domain permission, checked transformations, provider/backing requirements, and transitive authority while keeping proof evidence behind the evidence firewall, as described in Omega's authority-value brief.
- **`reaches` ceilings** bound which services an install-time transition may reach. Excluding ambient `Storage` reach outside the supplied setup capability is a checkable fact.
- **Immutable historical `data` shapes + checked conversion machines** carry persistent format lineages and upgrade paths as ordinary typed code with obligations.
- **Ordinary numbered schemas plus selected wire codecs** ([[ipc_and_service_invocation]]) declare the protocols spoken, making protocol compatibility part of the manifest.
- Omega does not yet define a package-manifest format or a "declarative install transition" primitive. That loader contract is an extension Cathedral pushes onto the runtime.

## Open Questions

- What is the concrete package-manifest format and the loader's declarative install-transition contract? Omega defines neither, so Cathedral must specify both as a runtime extension.
- Under what policy is a legacy package admitted as a walled closure, given that its manifest is attested but unproven and its updates can only be restart-class?

## Related
- [[capability_model]] — the capability manifest is authority-flow made durable.
- [[transactions_and_consistency]] — atomic install/uninstall.
- [[updates_and_hot_swap]] — the operational act the package enables.
- [[audit_compliance_provenance]] — signed, receipted build provenance and reproducibility policy.
- [[store_and_economic_control]] — distribution, revocation, staged rollout.
- [[governance_and_extension_boundaries]] — what a package may and may not be.
