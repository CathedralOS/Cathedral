# Chapter 00: Developer Experience

> The toolchain Cathedral hands a developer: not an editor plugin bolted onto a Unix, but a very opinionated distributed-systems IDE that makes authority, versions, and liveness *visible* before anything ships.

## The Legacy Contract

On Unix the developer experience is a loose federation of independent tools: a compiler, a libc, a set of CLI tools, `gdb`, `strace`, `valgrind`, a package manager, a CI service, and a store, each invented separately and unaware of the others. None of them can answer the questions that actually break production: *what authority does this binary need? where does it stash a credential for later? is this schema migration total? can this upgrade deadlock? which protocol field did I just break?* The information needed to answer them is dissolved across ELF symbols, runtime behavior, and tribal knowledge. The tooling sees bytes, so it shows you bytes.

## What Cathedral Wants

A single, opinionated SDK where the *facts the Omega compiler already produces* are first-class developer surfaces. The platform should feel less like a code editor and more like a control room for a distributed system you happen to be authoring. The default verbs are not "edit / build / run" but "inspect authority / diff a protocol / simulate an upgrade / replay a trace."

The organizing promise: **the developer can SEE it before a user feels it.** Your app *requires these capabilities*; this function *stores that authority*; this migration is *incomplete*; this upgrade *can deadlock*; this component *blocks hot swap*; this protocol change is *breaking*. Each of those is a report, not a surprise.

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

- Which compiler artifacts are stable enough to build durable tooling on, versus which are implementation detail that will churn?
- What is the *minimum* set of surfaces (authority graph, protocol diff, migration report, quiescence report) that makes the IDE feel like one product?
- How much of this is a GUI versus structured CLI output that other tools consume?

## Omega Leverage

- **Authority-flow reports** are the capability visualizer's data, unmodified.
- **Effect ceilings** drive the resource profiler and certification checklist.
- **Versioned `data` + migration reports** drive the migration tester and protocol explorer ([../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md)).
- **Quiescence / borrow-safety facts** drive the deadlock checker.
- **Proof obligations** ([../../../../Omega/wiki/language_guide/chapter_9_proof_obligations.md](../../../../Omega/wiki/language_guide/chapter_9_proof_obligations.md)) are the proof tooling's content. The tooling's job is *surfacing*, not deriving.

## Open Questions

- Does the IDE present authority/version/liveness as one fused graph view, or as separate lenses? A fused view is powerful but may be illegible.
- How opinionated is too opinionated — where does a strong default become a wall developers route around, recreating the same loose, uncoordinated tooling inside Cathedral?

## Related
- [[capability_model]] — the authority graph the visualizer renders.
- [[versioned_state_and_migration]] — what the migration tester checks.
- [[updates_and_hot_swap]] — quiescence facts the deadlock checker surfaces.
- [[ipc_and_service_invocation]] — protocols the explorer diffs.
- [[debugging_and_tracing]] — the debugger and tracer surfaces.
- [[testing_and_simulation]] — the simulator and deterministic replay.
