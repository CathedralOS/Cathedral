# Chapter 00: Developer Experience

> The toolchain Cathedral hands a developer is not an editor plugin bolted onto a Unix. It is an opinionated distributed-systems IDE that shows authority, versions, and liveness before anything ships.

## The Legacy Model

On Unix the developer experience is a loose federation of independent tools: a compiler, a libc, a set of CLI tools, `gdb`, `strace`, `valgrind`, a package manager, a CI service, and a store. Each was invented separately and is unaware of the others. None of them can answer the questions that break production. What authority does this binary need? Where does it stash a credential for later? Is this schema migration total? Can this upgrade deadlock? Which protocol field did I just break? The information needed to answer them is dissolved across ELF symbols, runtime behavior, and tribal knowledge. The tooling sees bytes, so it shows you bytes.

## The Cathedral Model

Cathedral has a single SDK in which the facts the Omega compiler already produces are the developer surfaces. The platform should feel less like a code editor and more like a control room for a distributed system you happen to be authoring. The default verbs are not "edit / build / run" but "inspect authority / diff a protocol / simulate an upgrade / replay a trace."

The organizing promise is that the developer can see a problem before a user feels it. Your app requires these capabilities. This function stores that authority. This migration is incomplete. This upgrade can deadlock. This component blocks hot swap. This protocol change is breaking. Each of those is a report the developer sees before shipping.

## The decided mechanism

Most of the SDK is surfacing facts the compiler already produces. The debugger, simulator, replay, package tooling, and certification gate are covered in [[debugging_and_tracing]], [[testing_and_simulation]], [[package_system]], and [[store_and_economic_control]]. The decisions below structure the rest.

### Stable-for-tooling = the frozen tier-1 contracts, nothing new

The artifacts durable enough to build tooling on are the ones already frozen for platform-identity reasons ([[governance_and_extension_boundaries]]): the manifest (the authority-flow report made durable), numbered external schemas and selected codecs, proof certs, historical `data` shapes plus their conversions, and the source state graph. Churning any of them would fork the ABI, so their stability is a commitment that already exists. Nothing new is frozen for tooling's sake. Compiler internals (IR, lowering) stay unstable. Formats like debug-info are tooling-side builds over exported raw material, which is the lesson of RAD Debugger's RDI.

### Every report is a typed artifact; the IDE, CI, and an agent are three lenses

Each surface (authority graph, protocol diff, migration totality, quiescence) is a typed artifact encoded from an ordinary numbered schema. A human GUI, a CI gate, and an LLM agent consume the same artifact. Given the bet that LLMs author the proofs, the agent is arguably the primary consumer. That is why structured typed output is canonical rather than human-formatted text, and why the IDE is a renderer over it. The question of a fused view versus separate lenses dissolves. There is one fact base, and a fused view is a saved query.

### Opinionated about facts and gates, permissive about tools

The OS is opinionated about one thing: the facts are canonical and the gates re-check them. The manifest is the enforced ceiling. Proofs are re-verified at admission. Published external schemas and selected codecs must be compatible. Migrations must be total. CI runs these checks and the store's admission gate re-runs them regardless of the tool used, so a bad fact cannot ship. Everything else is free: editor, frontend language, GUI or CLI, whether you use the reference IDE.

This defuses the worry that a strong default becomes a wall developers route around. You can only route around presentation, which is harmless. You cannot route around facts, because they are re-checked. N loose tools sharing one canonical typed fact base is not the Unix disease, because the coordination lives in the fact base rather than in the tools. Identity is frozen; the surface is free.

### A tool is a core component plus thin frontends: there is no library-vs-CLI distinction

"Runnable" is not a binary-format property. It is a conventional interface, `main(args) -> ExitCode`, that the shell knows how to drive. A "library" is a component exposing a domain interface such as `search(...)`. A "CLI" is a component exposing `main`. Both are the same uniform component artifact, differing only in which interface is invoked and by whom. There is no separate executable format versus `.so` format and no `x` bit, which is also why an "executable" is not a blessed kind of file ([[cathedral-no-blessed-apps]]).

A tool is therefore a core component with a typed interface plus thin frontends, grouped in one package the way Cargo groups `[lib]` and `[[bin]]`. The `cli` frontend is a `main` over the core. The `gui` frontend is a surface over the core. `import` materializes the core, `run` materializes the core plus the cli, and the unused frontends are chunks nobody fetches. Dual use is the default. A `git`/`libgit2` split is an anti-pattern except across a provenance boundary, where a third-party frontend is a separate package composing the core. The single address space also collapses link versus call: embedding the core and invoking it as a walled service are one interface, and the choice between them is a placement decision.

Nothing forbids a monolith. One component exposing `main`, `surface`, and `search` at once is legal. It is paid for on two axes, because a component is one dependency-closure unit and one trust boundary. Every consumer, even a headless one that only calls `search`, materializes the union of all frontends' dependencies, so a GUI toolkit rides along on a server that never renders. Every consumer also instantiates a boundary carrying all the frontends' code and authority, so the library consumer inherits the GUI's attack surface. Splitting into separate objects avoids both costs: smaller closure, smaller boundary, each consumer pulling its subset. That is why the split is the default for a heavy frontend. Cathedral does not ban the monolith. It makes the cost legible, because the dependency and capability graphs show what a given entry point drags in, so the choice is made with eyes open rather than hidden as legacy bloat.

### Dependencies are pinned closures with content-dedup, never namespace fishing

A Cathedral program cannot "expect grep on the system" and dynamically link it by name. There is no global namespace, no `PATH`, no `LD_LIBRARY_PATH`. Legacy dynamic linking conflated two things, and Cathedral keeps one and drops the other. Sharing, meaning one physical copy and no static bloat, is kept through content-addressed dedup. Ambient name resolution, meaning binding whatever the system provides by name, is gone.

A program has two ways to reach code it does not contain. The common case is to declare a dependency in its closure by hash: `grep@hash` is in the app's complete, pinned closure, so the app never fishes, "not found" cannot happen, and it never gets the wrong version. The other case is to hold a capability to a provider, handed over by its host or a broker, when it wants the environment's live instance.

Physically, identical objects are stored once and refcounted. Installing fetches only the chunks you lack. An already-present object is `refcount++`. GC reclaims an object when the last reference drops. Every app logically holds its whole closure while physically deduping. Dedup is by exact content hash, so two apps share a file only if it is bit-identical, and different pinned versions are different files with no conflict. This gives every good property of shared libraries with no DLL hell, because it is dedup-by-content rather than resolve-by-name.

### ABI evolution: additive by default, versioned-coexistence for breaks

Tier-1 contracts still evolve. Additive changes (new numbered fields, stable identities, reader tolerance) are forward-compatible and cover most evolution without a break. A breaking change is a new contract version, and the OS serves both over a bounded deprecation window. Components pin their target version in the manifest, so the OS enumerates which installed components still target the old contract, migrates or auto-ports them with proof-carrying conversion machines, and then drops the old version. This threads between Linux fragmentation and Win32 cruft. Fragmentation is avoided because there is one versioned contract set, not N variants. Cruft is avoided because deprecation is bounded and legible, since you can see who is affected. Mechanically it is the same versioned-interface and multi-component-update machinery as [[package_system]].

## Concerns & Design Space

- **SDK & package tooling.** One toolchain builds components, resolves versioned dependencies, and publishes to a local store ([[package_system]], [[store_and_economic_control]]), rather than N disconnected tools.
- **Proof tooling.** Omega's obligations surface as developer feedback: which `requires`/`ensures` are discharged, which need a `relax` scope, where a proof is the thing blocking the build.
- **Capability visualizer.** The per-component authority-flow report renders as a graph: accepts, uses, derives, stores, acquires, returns, releases. Stored authority is highlighted because it is the dangerous, long-lived case ([[capability_model]]).
- **Protocol explorer.** Numbered schemas and their selected wire policies are diffed across versions, and the breaking change is named before publish: the field that moved, the compatibility rule that broke ([[ipc_and_service_invocation]]).
- **Migration tester.** A versioned-state migration is proved total over the prior shape, and the case it does not cover is flagged ([[versioned_state_and_migration]]).
- **Deadlock / quiescence checker.** Shows whether an upgrade can reach quiescence, and which outstanding borrow or wait blocks the swap ([[updates_and_hot_swap]]).
- **Debugger, tracer, simulator, deterministic replay.** Shared, capability-governed, and part of the platform rather than bolted on. Detailed in [[debugging_and_tracing]] and [[testing_and_simulation]].
- **Resource profiler & certification tool.** Effect ceilings and resource bounds as a pre-certification checklist the developer runs locally before the store gate ([[store_and_economic_control]]).
- **CI integration.** The same reports (authority, protocol, migration, deadlock) run in CI as gates, so the control-room view is enforced, not advisory.

## Key Questions

- Which artifacts are stable enough to build tooling on? The frozen tier-1 contracts and nothing else: manifest, numbered external schemas and codec policies, proof certs, historical data plus conversions, source state graph. Compiler internals stay unstable, and formats like debug-info are tooling-side builds over exported raw material.
- Is there one product surface or many? One typed fact base. Every report is a typed, schema-encoded artifact over it, and the IDE, CI, and an LLM agent are three lenses. A fused view is a saved query, not a monolithic UI.
- Is the canonical output a GUI or structured data? Structured typed artifacts are canonical, since the agent is arguably the primary consumer, and the GUI is a renderer. A tool is a core component plus thin frontends in one package, where `import`, `run`, and `gui` materialize their subset. There is no library-vs-CLI binary distinction; "runnable" is the `main` interface.

## Omega Leverage

- Authority-flow reports are the capability visualizer's data, unmodified.
- Effect ceilings drive the resource profiler and certification checklist.
- Historical-schema and conversion reports drive the migration tester and protocol explorer ([Omega Chapter 14: Versioned Data](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md#versioned-data)).
- Quiescence and borrow-safety facts drive the deadlock checker.
- Proof obligations ([Omega Chapter 9: Proof Obligations](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_9_proof_obligations.md)) are the proof tooling's content. The tooling's job is surfacing, not deriving.

## Open Questions

- How long is the bounded deprecation window for a breaking tier-1 contract, and who sets it?
- Who builds the tooling-side formats, such as a debug-info format over the exported raw material, and does the reference IDE ship one?

## Related
- [[capability_model]] — the authority graph the visualizer renders.
- [[versioned_state_and_migration]] — what the migration tester checks.
- [[updates_and_hot_swap]] — quiescence facts the deadlock checker surfaces.
- [[ipc_and_service_invocation]] — protocols the explorer diffs.
- [[debugging_and_tracing]] — the debugger and tracer surfaces.
- [[testing_and_simulation]] — the simulator and deterministic replay.
