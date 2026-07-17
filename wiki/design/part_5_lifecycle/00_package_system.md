# Chapter 00: Package & Component System

> A package is not a tarball plus scripts — it is a proof-carrying object whose install is a declarative, checked state transition, never code execution.

## The Legacy Model

Linux packaging is the clearest example of the problem: a package is an archive plus a set of imperative hooks — `preinst`, `postinst`, `%post`, `pkg_postinst` — that run as root with full ambient authority. **Install *is* arbitrary code execution.** Around that core sit dependency solvers, dynamic-linker search paths, signature schemes, trust databases, and per-distro policy, none of which can constrain the one thing that matters: what the postinstall script actually does. The system cannot answer "what authority does this package require, what state does it own, is this upgrade compatible, and can I revoke it?" — because none of that was ever declared. It was buried in a shell script.

## The Cathedral Model

Installation is a **declarative state transition**, not a program. A package is an Omega artifact that *declares* everything the system needs to admit it, and the compiler/loader *checks* the declaration before anything is written. There is no install-script escape hatch — the manifest is the only way in.

A package declares exports/imports; capabilities required, stored, and
delegated; protocol schema/codec identities; persistent named state schemas and
migration functions; resource budgets; provider-compatibility facts; security
invariants; and the test/proof artifacts backing them. Build provenance is
signed and reproducible; dependencies are hermetic. Installation verifies the
whole declaration and atomically commits or refuses.

This is the core divergence: every other OS treats the package as an opaque payload whose effects are discovered only by running it; Cathedral treats it as a checkable contract whose blast radius is computed in advance.

## The decided mechanics

The thesis above (declarative, checkable, no scripts) is the framing; the mechanics below are how install/update actually run. Most of the *mechanics* are prior art — delta updates, A/B slots, atomic snapshots, content-addressed generations all ship in macOS (the sealed System Volume), Android/ChromeOS (A/B), and NixOS/Silverblue (generations). Cathedral's contribution is not those mechanics but their **composition** with four distinguishing pieces: content-addressed **chunked** closures, **proof-carrying** components re-checked on update, **capability manifests** (declarative install, no scripts), and **versioned interfaces** (so partial updates stay valid).

### Fetch is a chunk-level delta

A package is a content-addressed closure, but content-addressing is at the **chunk** level, not whole-object: files are split by **content-defined chunking** (a rolling hash picks boundaries, so an insertion re-syncs after the edit instead of shifting every later chunk), each chunk is addressed, and a file is a Merkle manifest of chunk hashes. So a 10 KB patch to a 2 GB file fetches only the touched chunk(s) — tens to hundreds of KB, not the file — and chunks dedupe globally across components, versions, and apps (casync/borg-style). A new OS version pulls only the closures, and within them only the chunks, whose hashes differ.

### The update pipeline and its gates

An update **stages additively** — new closures written beside the old, nothing overwritten, the machine still running the old — then passes checked gates before any commit, each of which *refuses cleanly* rather than half-applying:

- **Integrity** — a chunk whose hash mismatches is rejected by construction.
- **Attestation + proof re-check** — the closure signature verifies and the local checker **re-verifies the proof certs**; you never trust the publisher's "it's proven," so a forged proof dies here (Thompson-resistance).
- **Manifest delta** — a **widened** ceiling (authority the old version didn't declare) requires fresh consent; it cannot land silently.
- **Migration totality** — a state-schema change ships a migration proven **total**; absent that, the swap is refused and the old state is left intact.

Refuse at any gate → you stay on the working old version, nothing half-installed.

### Live-patch versus reboot is proof-gated, with reboot as the safe fallback

Whether a component (or the core) can be updated **in place** is not guessed — you **prove live-patchability, or you reboot.** Three checkable gates:

1. **Quiescence** — the checker proves the component reaches a safepoint in bounded time (the preemptibility proof, [[scheduler_and_resources]]).
2. **Total migration** — the new version's state migration is proven total over the old committed state.
3. **Containment** — the change touches only the component's own code + `data`, not a **foundational representation every live thing embodies.** A normal component *can't* change the substrate (it is built on it), so its updates are contained by construction; the reboot class is specifically a change to a core foundational primitive whose **in-memory layout** changed — the memory model (MMU/SAS/CHERI), the capability-arena format, the task/continuation representation, the IPC region layout, the trap-vector model. Detected by whether a foundational layout changed vs. only code/policy.

All three pass → **live hot-swap.** Any fail or absent → **staged reboot** — always correct, so an unsure classification safely defaults to reboot. (Changing the memory model provably fails gate 3 by inspection: its purpose is to re-lay-out what all live state embodies — [[kernel_architecture]].)

### A/B applies on the next natural reboot — never forced

Most updates are **services and drivers → live hot-swap, zero reboot** ([[updates_and_hot_swap]]). A small core fix (a single-function security patch) can be **live-patched** into the running core at a safepoint (Linux-`livepatch`-style, cleaner here because the core *is* the hot-swap machinery). Only a **wholesale core replacement** (a foundational-layout change) needs the A/B path — and even then it **stages into the inactive slot and applies on the next reboot the user takes anyway**, with automatic fallback to the old slot if the new one fails to boot-measure-good ([[boot_and_trust_chain]]). No forced reboot; at most a *nudge* for a critical security fix. Reboots are therefore rare (only deep core rewrites) and never imposed.

### Many components update independently — no OS-wide two-phase commit

Each component's swap is an **atomic copy-on-write root flip**. Across
components, updates flip independently because pinned interface contracts and
compatible protocol schemas make mixed-provider operation valid. A coupled
breaking change temporarily serves both explicit contract identities or uses a
coordinated flip scoped to the coupled set—never an OS-wide 2PC. A feature that
must appear atomic deploys incrementally and is enabled by a final config flag.

### The setup-transition: a confined computation, not a script

The one place imperative code wants back in — first-run setup (seed a database, build an initial index) — is a **confined setup machine**, not a `postinstall` script. It is a computation `{ fresh realm } + { explicitly-granted input capabilities } → initial-state value`, run with *no ambient authority and no side effects*: it holds a write-cap to its own fresh blank realm plus whatever inputs it was **granted by gesture or manifest** (read a prior install's data to import, query hardware to detect devices), and it **produces the initial state as a value committed atomically.** Side-effect setup — registering a service, a file association — is **declarative** (the manifest declares it, the activator registers it, [[service_activation]]), never code. So the postinstall hole closes: setup can only *compute a value from granted inputs*, never *act on the system*.

### Migration with extra inputs is the same shape — Omega's `capture` + pure `upgrade`

A migration needing external state captures it into an owned typed context
before commit, then prepares the successor from explicit old/context inputs.
This is an ordinary Cathedral framework protocol, optionally organized by an
`Upgradable<Old, New, Context>` library trait—not privileged Omega syntax. See
Omega [Evolution, Migration, And Replacement](../../../../Omega/wiki/language_guide/chapter_22_versioned_data.md).

## Concerns & Design Space

- **No install scripts, ever.** Any per-install side effect must be expressed as a declared state transition the loader executes under a bounded capability set, not as code the package supplies. This is the load-bearing principle of the chapter.
- **Capability manifest.** What the package `accepts`, `uses`, `stores`, `derives`, and `acquires` (from [[capability_model]]) is the heart of the manifest. No ambient filesystem, network, or device access during install.
- **State schemas & migration.** A package owns explicit persistent shapes and
  ordinary migration machines (see [[versioned_state_and_migration]]).
- **Atomic install / uninstall.** Commit or refuse as one transaction ([[transactions_and_consistency]]); no half-installed state, no orphaned hooks.
- **Reproducible, hermetic builds + signed provenance.** The same source yields the same artifact; the build's inputs and signer are recorded ([[audit_compliance_provenance]]).
- **Static, hermetic linking.** A component ships with its dependencies resolved and pinned at build time, so there is no runtime symbol resolution and no dynamic-library search path. Identical code used by many instances or components is deduplicated in physical memory by content address ([[filesystem_as_database]]), so a component carrying its own dependencies does not cost extra RAM per instance.
- **Machine-checkable compatibility.** ABI/protocol/schema compatibility with the installed world is a checked fact, not a version-string heuristic. The baseline it is checked against exists by construction: the distribution plane records every published version's declared schemas ([[store_and_economic_control]]), so admission can refuse a package whose wire schemas break what came before, instead of relying on a developer to supply the old schema by hand. Storage formats are the strict case, since on-disk data can outlive every version that could write it ([[versioned_state_and_migration]]).
- **Revocation & staged rollout.** Package-level revocation, staged/canary rollout, and rollback are first-class, gated by the store/control plane ([[store_and_economic_control]]).
- **Zero value.** A zero package is the empty manifest (valid-empty): it declares no exports and an empty capability manifest, so it requires zero authority and installs as a no-op, which is both the least-privilege admission case and ZII-coherent ([[omega_substrate]]).

## Key Questions

- **Manifest content, build-time vs install-time — resolved:** the manifest is the authority-flow report (accepts/uses/stores/derives/acquires) + state schemas + protocols + upgrade-compatibility facts + proof certs; the **proofs are proven at build and *re-checked* at install**, and **compatibility** (schema/protocol/manifest-delta) is checked at install against the live system.
- **Declarative install expressing legitimate side effects — resolved:** side-effect setup (register a service, an association) is **declarative** (the activator registers it, no code); *computational* setup (seed/build initial state) is a **confined setup machine** that maps a fresh realm + granted inputs → an initial-state value with no ambient authority and no side effects.
- **Dependency model — resolved:** content-addressed, **chunked**, hermetic and pinned; versions coexist by hash (no solver, no dependency hell); fetch is a chunk-level delta.
- **ABI stability — resolved:** the store records every published schema and
  codec-plan identity, so admission refuses an unaccounted breaking update;
  durable formats are the strict case because data outlives its writers.

## Omega Leverage

- **Authority-flow inference** produces the capability manifest directly — the accepts/uses/stores/derives/acquires report *is* what the package declares.
- **`effects` ceilings** bound what an install-time transition may touch; an install with no `filesystem_io` outside its allotted folder is a checkable fact.
- **Versioned `data` + migration machines** carry the persistent-state schema and its upgrade path as typed code with obligations.
- **Protocol schema/codec identities** ([[ipc_and_service_invocation]]) declare
  the protocols spoken, making compatibility part of the manifest.
- Omega does **not yet** define a package-manifest format or a "declarative install transition" primitive — that loader contract is an extension Cathedral pushes onto the runtime.

## Open Questions

- **Setup machine — resolved:** a confined setup machine *is* used (not everything is pure-declarative), and it is kept from becoming the postinstall hole by tight confinement — only a fresh blank realm + explicitly-granted input capabilities, no ambient authority, no side effects (side effects are declarative). It can only compute a value, never act on the system.
- **Foreign/native components — resolved-direction:** a legacy/foreign package is a **walled closure with an attested-but-unproven manifest**, contained-not-trusted ([[compatibility_and_legacy]]); it cannot prove quiescence, so its updates are **restart/reboot-class, not live hot-swap** — legacy-language drivers are disadvantaged on update, an accepted cost.

## Related
- [[capability_model]] — the capability manifest is authority-flow made durable.
- [[transactions_and_consistency]] — atomic install/uninstall.
- [[updates_and_hot_swap]] — the operational act the package enables.
- [[audit_compliance_provenance]] — signed, reproducible build provenance.
- [[store_and_economic_control]] — distribution, revocation, staged rollout.
- [[governance_and_extension_boundaries]] — what a package may and may not be.
