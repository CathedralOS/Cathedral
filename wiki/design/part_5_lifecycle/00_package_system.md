# Chapter 00: Package & Component System

> A package is not a tarball plus scripts — it is a proof-carrying object whose install is a declarative, checked state transition, never code execution.

## The Legacy Model

Linux packaging is the clearest example of the problem: a package is an archive plus a set of imperative hooks — `preinst`, `postinst`, `%post`, `pkg_postinst` — that run as root with full ambient authority. **Install *is* arbitrary code execution.** Around that core sit dependency solvers, dynamic-linker search paths, signature schemes, trust databases, and per-distro policy, none of which can constrain the one thing that matters: what the postinstall script actually does. The system cannot answer "what authority does this package require, what state does it own, is this upgrade compatible, and can I revoke it?" — because none of that was ever declared. It was buried in a shell script.

## The Cathedral Model

Installation is a **declarative state transition**, not a program. A package is an Omega artifact that *declares* everything the system needs to admit it, and the compiler/loader *checks* the declaration before anything is written. There is no install-script escape hatch — the manifest is the only way in.

A package declares: exports and imports; capabilities required, stored, and delegated; protocols spoken ([[ipc_and_service_invocation]] `wire data`); persistent state schemas with their versions and migration functions; resource budgets; upgrade-compatibility facts; security invariants; and the test/proof artifacts that back them. Build provenance is signed and reproducible; dependencies are hermetic. Because all of this is structured, "install this package" reduces to: verify provenance, check the capability manifest against policy, prove the state schema migrates, allocate the budget, and atomically commit — or atomically refuse.

This is the core divergence: every other OS treats the package as an opaque payload whose effects are discovered only by running it; Cathedral treats it as a checkable contract whose blast radius is computed in advance.

## Concerns & Design Space

- **No install scripts, ever.** Any per-install side effect must be expressed as a declared state transition the loader executes under a bounded capability set, not as code the package supplies. This is the load-bearing principle of the chapter.
- **Capability manifest.** What the package `accepts`, `uses`, `stores`, `derives`, and `acquires` (from [[capability_model]]) is the heart of the manifest. No ambient filesystem, network, or device access during install.
- **State schemas & migration.** A package owns persistent state shapes; upgrades ship versioned data + migration machines (see [[versioned_state_and_migration]]).
- **Atomic install / uninstall.** Commit or refuse as one transaction ([[transactions_and_consistency]]); no half-installed state, no orphaned hooks.
- **Reproducible, hermetic builds + signed provenance.** The same source yields the same artifact; the build's inputs and signer are recorded ([[audit_compliance_provenance]]).
- **Machine-checkable compatibility.** ABI/protocol/schema compatibility with the installed world is a checked fact, not a version-string heuristic.
- **Revocation & staged rollout.** Package-level revocation, staged/canary rollout, and rollback are first-class, gated by the store/control plane ([[store_and_economic_control]]).

## Key Questions

- What exactly is in the manifest, and which parts are *proven at build time* vs. *checked at install time* against the live system?
- How does a declarative install express the legitimate side effects a postinstall script used to do (seed a database, register a service) without becoming code?
- What is the dependency model — hermetic pinned graph, content-addressed, both?
- How is long-term ABI stability defined and enforced across package generations?

## Omega Leverage

- **Authority-flow inference** produces the capability manifest directly — the accepts/uses/stores/derives/acquires report *is* what the package declares.
- **`effects` ceilings** bound what an install-time transition may touch; an install with no `filesystem_io` outside its allotted folder is a checkable fact.
- **Versioned `data` + migration machines** carry the persistent-state schema and its upgrade path as typed code with obligations.
- **`wire data`** ([[ipc_and_service_invocation]]) declares the protocols spoken, making protocol compatibility part of the manifest.
- Omega does **not yet** define a package-manifest format or a "declarative install transition" primitive — that loader contract is an extension Cathedral pushes onto the runtime.

## Open Questions

- Can *every* legitimate install side effect be modeled declaratively, or is a tiny, capability-bounded "setup machine" unavoidable — and if so, how is it kept from becoming the new postinstall hole?
- How are native/foreign components (a legacy box, see [[compatibility_and_legacy]]) packaged without contaminating the proof-carrying model?

## Related
- [[capability_model]] — the capability manifest is authority-flow made durable.
- [[transactions_and_consistency]] — atomic install/uninstall.
- [[updates_and_hot_swap]] — the operational act the package enables.
- [[audit_compliance_provenance]] — signed, reproducible build provenance.
- [[store_and_economic_control]] — distribution, revocation, staged rollout.
- [[governance_and_extension_boundaries]] — what a package may and may not be.
