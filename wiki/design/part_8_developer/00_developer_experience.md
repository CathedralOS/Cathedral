# Chapter 00: Developer Experience

> The toolchain Cathedral hands a developer: not an editor plugin bolted onto a Unix, but a very opinionated distributed-systems IDE that makes authority, versions, and liveness *visible* before anything ships.

## The Legacy Model

On Unix the developer experience is a loose federation of independent tools: a compiler, a libc, a set of CLI tools, `gdb`, `strace`, `valgrind`, a package manager, a CI service, and a store, each invented separately and unaware of the others. None of them can answer the questions that actually break production: *what authority does this binary need? where does it stash a credential for later? is this schema migration total? can this upgrade deadlock? which protocol field did I just break?* The information needed to answer them is dissolved across ELF symbols, runtime behavior, and tribal knowledge. The tooling sees bytes, so it shows you bytes.

## The Cathedral Model

A single, opinionated SDK where the *facts the Omega compiler already produces* are first-class developer surfaces. The platform should feel less like a code editor and more like a control room for a distributed system you happen to be authoring. The default verbs are not "edit / build / run" but "inspect authority / diff a protocol / simulate an upgrade / replay a trace."

The organizing promise: **the developer can SEE it before a user feels it.** Your app *requires these capabilities*; this function *stores that authority*; this migration is *incomplete*; this upgrade *can deadlock*; this component *blocks hot swap*; this protocol change is *breaking*. Each of those is a report the developer sees before shipping.

## The decided mechanism

Most of the SDK is *surfacing facts the compiler already produces* (debugger, simulator, replay, package tooling, the certification gate are banked in [[debugging_and_tracing]], [[testing_and_simulation]], [[package_system]], [[store_and_economic_control]]). Four decisions structure the rest.

### Stable-for-tooling = the frozen tier-1 contracts, nothing new

The artifacts durable enough to build tooling on are exactly the ones already frozen for platform-identity reasons ([[governance_and_extension_boundaries]]): the **manifest** (the authority-flow report made durable), **`wire data` schemas**, **proof certs**, **versioned-`data` shapes + migrations**, and the **source state graph**. Churning them would fork the ABI, so their stability is a commitment that already exists. Compiler *internals* (IR, lowering) stay unstable; formats like debug-info are tooling-side builds over exported raw material (the RDI lesson).

### Every report is a typed artifact; the IDE, CI, and an agent are three lenses

Each surface — authority graph, protocol diff, migration totality, quiescence — is a **typed `wire data` artifact**, consumed identically by a human GUI, **CI gates**, and **LLM agents**. Given the LLM-authors-the-proofs bet, the agent is arguably the *primary* consumer, which is why structured typed output — not human-formatted text — is canonical, and the IDE is a renderer over it. "Fused view vs separate lenses" dissolves: one fact base, and a fused view is just a saved query.

### Opinionated about facts and gates, permissive about tools

The OS is opinionated about exactly one thing — the **facts are canonical and the gates re-check them**: the manifest is the enforced ceiling, proofs are re-verified at admission, wire schemas must be compatible, migrations must be total. CI runs these and the store's admission gate re-runs them *regardless of the tool used*, so a bad fact **cannot ship**. Everything else — editor, frontend language, GUI vs CLI, whether you use the reference IDE — is free. So "a strong default becomes a wall devs route around" is defanged: you can only route around **presentation** (harmless), never **facts** (re-checked). N loose tools sharing one canonical typed fact base is not the Unix disease, because the coordination lives in the fact base, not the tools — identity frozen, surface free.

### A tool is a core component + thin frontends — there is no library-vs-CLI distinction

"Runnable" is not a binary-format property; it is a **conventional interface** `main(args) -> ExitCode` the shell knows how to drive. A "library" is a component exposing a domain interface (`search(...)`); a "CLI" is a component exposing `main`. **Same uniform component artifact**, differing only in *which interface is invoked and by whom* — so there is no separate executable vs `.so` format, no `x` bit (which is also why an "executable" is not a blessed kind of file, [[cathedral-no-blessed-apps]]). A tool is therefore a **core component with a typed interface plus thin frontends** (`cli` = a `main` over the core; `gui` = a surface over the core), grouped in one **package** the Cargo `[lib]` + `[[bin]]` way. `import` materializes the core; `run` materializes core + cli; the unused frontends are chunks nobody fetches — so dual-use is the *default*, and `git`/`libgit2` splits are an anti-pattern except across a **provenance** boundary (a third-party frontend is a separate package composing the core). SAS additionally collapses link-vs-call (embedding the core vs invoking it as a walled service is a placement decision, one interface).

Nothing *forbids* a monolith — one component exposing `main` + `surface` + `search` all at once is legal — but it is paid for on two axes, because a component is one dependency-closure unit *and* one trust boundary: every consumer (even a headless one that only calls `search`) materializes the **union of all frontends' dependencies** (a GUI toolkit riding along on a server that never renders) and instantiates a boundary carrying **all the frontends' code + authority** (the lib consumer inherits the GUI's attack surface). The split into separate objects avoids both — smaller closure, smaller boundary, each consumer pulls its subset — which is why it is the default for a heavy frontend. Cathedral does **not ban** the monolith; it makes the cost **legible** — the dependency and capability graphs show exactly what a given entry point drags in — so the choice is made with eyes open rather than hidden as legacy bloat.

### Dependencies are pinned closures with content-dedup, never namespace fishing

A Cathedral program **cannot** "expect grep on the system" and dynamically link it by name — there is no global namespace, no `PATH`, no `LD_LIBRARY_PATH`. Legacy dynamic linking conflated two things; Cathedral keeps the good one and kills the bad: **sharing** (one physical copy, no static bloat) is kept via **content-addressed dedup**; **ambient name resolution** (bind whatever the system provides by name) is killed. So a program either **declares a dependency in its closure by hash** (the common case — `grep@hash` is *in the app's complete, pinned closure*; the app never fishes, "not found" cannot happen, and it never gets the wrong version) or **holds a capability to a provider** (handed by its host / a broker when it wants the environment's live instance). Physically, identical objects are stored once and refcounted — installing fetches only the chunks you lack, an already-present object is `refcount++`, GC reclaims it when the last reference drops — so every app *logically* holds its whole closure while *physically* deduping. Crucially, dedup is **by exact content hash**, so two apps share a file only if it is bit-identical: different pinned versions are different files, no conflict — every good property of shared libraries with **no DLL hell**, because it is dedup-by-content, not resolve-by-name.

### ABI evolution: additive by default, versioned-coexistence for breaks

Tier-1 contracts still evolve. **Additive** changes (new `wire data` fields, stable numbers, reader tolerance) are forward-compatible — most evolution, no break. A genuine **breaking** change is a **new contract version**, and the OS serves both over a **bounded deprecation window**; components pin their target version in the manifest, so the OS enumerates *exactly* which installed components still target the old contract and migrates/auto-ports them (proof-carrying migration machines), then drops it. This threads between Linux fragmentation (avoided — one versioned contract set, not N variants) and Win32 cruft (avoided — deprecation is bounded and *legible*, because you can see who is affected). Mechanically it is the same versioned-interface + multi-component-update machinery as [[package_system]].

## Concerns & Design Space

- **SDK & package tooling.** One toolchain that builds components, resolves versioned dependencies, and publishes to a local store ([[package_system]], [[store_and_economic_control]]) — not N disconnected tools.
- **Proof tooling.** Surface Omega's obligations as developer feedback: which `requires`/`ensures` are discharged, which need a `relax` scope, where a proof is the thing blocking the build.
- **Capability visualizer.** Render the per-component authority-flow report as a graph: accepts / uses / derives / **stores** / acquires / returns / releases. Stored authority is highlighted because it is the dangerous, long-lived case ([[capability_model]]).
- **Protocol explorer.** Diff `wire data` schemas across versions and *name the breaking change* before publish — the field that moved, the compatibility rule that broke ([[ipc_and_service_invocation]]).
- **Migration tester.** Prove a versioned-state migration is total over the prior shape, and flag the case it does not cover ([[versioned_state_and_migration]]).
- **Deadlock / quiescence checker.** Show whether an upgrade can reach quiescence, and which outstanding borrow or wait blocks the swap ([[updates_and_hot_swap]]).
- **Debugger, tracer, simulator, deterministic replay.** First-class, shared, and capability-governed — detailed in [[debugging_and_tracing]] and [[testing_and_simulation]].
- **Resource profiler & certification tool.** Effect ceilings and resource bounds as a pre-certification checklist the developer runs locally before the store gate ([[store_and_economic_control]]).
- **CI integration.** The same reports (authority, protocol, migration, deadlock) run in CI as gates, so the control-room view is enforced, not advisory.

## Key Questions

- **Stable-for-tooling artifacts — resolved:** the frozen tier-1 contracts (manifest, `wire data` schemas, proof certs, versioned-`data` + migrations, source state graph); compiler internals stay unstable, and formats like debug-info are tooling-side builds over exported raw material.
- **One-product surface — resolved:** every report is a typed `wire data` artifact over one fact base; the IDE, CI, and an LLM agent are three lenses, and a "fused view" is a saved query, not a monolithic UI.
- **GUI vs structured output — resolved:** structured typed artifacts are canonical (the agent is arguably the primary consumer); the GUI is a renderer. A tool is a core component + thin frontends in one package (`import`/`run`/`gui` materialize their subset); there is no library-vs-CLI binary distinction — "runnable" is just the `main` interface.

## Omega Leverage

- **Authority-flow reports** are the capability visualizer's data, unmodified.
- **Effect ceilings** drive the resource profiler and certification checklist.
- **Versioned `data` + migration reports** drive the migration tester and protocol explorer ([../../../../Omega/wiki/language_guide/chapter_22_versioned_data.md](../../../../Omega/wiki/language_guide/chapter_22_versioned_data.md)).
- **Quiescence / borrow-safety facts** drive the deadlock checker.
- **Proof obligations** ([../../../../Omega/wiki/language_guide/chapter_9_proof_obligations.md](../../../../Omega/wiki/language_guide/chapter_9_proof_obligations.md)) are the proof tooling's content. The tooling's job is *surfacing*, not deriving.

## Open Questions

- **Fused vs separate lenses — resolved:** one typed fact base, many lenses; a fused view is a saved query, not a UI to design up front.
- **Too opinionated — resolved:** opinionated only on the re-checked facts/gates (you cannot ship a bad fact); permissive on all presentation (route around any tool — harmless, because the gate re-checks regardless). Loose tools over one canonical fact base is not the Unix disease.

## Related
- [[capability_model]] — the authority graph the visualizer renders.
- [[versioned_state_and_migration]] — what the migration tester checks.
- [[updates_and_hot_swap]] — quiescence facts the deadlock checker surfaces.
- [[ipc_and_service_invocation]] — protocols the explorer diffs.
- [[debugging_and_tracing]] — the debugger and tracer surfaces.
- [[testing_and_simulation]] — the simulator and deterministic replay.
