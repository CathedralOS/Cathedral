# Speculative: Future Browser Design

> **Status: SPECULATIVE.** A forward-looking exploration, not committed design. Captures a design conversation (2026-06-15) about what "the browser" becomes on a capability-first OS. Design chapters it touches: [[web_integration]], [[compatibility_and_legacy]], [[updates_and_hot_swap]], [[capability_model]], and the Omega `design_briefs/verified_gated_ml_optimizer.md` brief. Nothing here is committed; it is a coherent vision to revisit, not a spec.

## The reframe: decompose the browser, don't replace it

The web browser is about six distinct functions fused by historical accident:

| What the browser does | On Cathedral |
|---|---|
| Sandbox untrusted code | dissolves into confinement (the OS) |
| Capability-confine (camera/mic prompts) | dissolves into the capability model |
| Ephemeral, zero-install, run-by-reference | an OS *lifecycle* pattern |
| Cross-OS portability | *universality*, the hard remainder |
| Document / hypertext model | *declarative content* |
| Frictionless zero-trust-decision delivery | a *web-tier capability default* |

Cathedral dissolves the top two rows (isolation, capability-confinement) into the OS. "Kill the browser" is the wrong frame. The browser decomposes: its isolation role evaporates, and what remains are real problems that are not OS-isolation (portability, a content model, ephemeral delivery, frictionless safety). The browser stops being a special subsystem and becomes two thin shells over OS primitives: a **content viewer** and a **program host**.

## The artifact: Omega IR, tiered fidelity

A web-artifact is **Omega IR**: a frozen, content-addressed, capability-carrying program, not HTML+JS+WASM soup. It runs with graceful capability-fidelity degradation:

- **Capable host (Cathedral).** Compile the IR to a native exe and run it as a walled child of a sandbox-host component (the "browser" is the recursive-provider gatekeeper, [[capability_model]]), capabilities granted up front, at native speed.
- **Legacy host (today's browsers, vestigial future OSes).** Lower the IR to WASM and interpret or JIT it in the existing sandbox. Gating is coarse (declared manifest to sandbox permissions), but it runs everywhere: the "works on vestigial OSes" tier.

Same artifact, two tiers. The capability requirements travel in the artifact, legible everywhere and enforced as finely as the host allows. We do not invent a new bytecode for the legacy tier, because the JVM-applet, Flash, and Silverlight graveyard is full of new-bytecode bets. We ride WASM, with Omega as the source and IR that lowers to both.

## Why native tabs are fast (not slow)

A native tab pays for isolation only at boundary crossings, which are rare. The MMU does not slow compute, since translation is transparent through the TLB. The only cost is a boundary crossing (a syscall or IPC to the gatekeeper), paid only when you cross. A well-built gatekeeper grants capabilities up front (hand the tab a framebuffer region, a network handle) and stays out of the hot path. The tab then touches its granted resources directly at native speed.

A web-tab is native exes running, faster than a real browser because it skips the JS-interpret/JIT/DOM tax. The seam cost only bites crossing-heavy hot paths (zillions of tiny syscalls), which a tab is not. PCC is not needed here, since the hardware wall plus runtime capability-deny already make foreign code safe. Proof-carrying code is only the advanced option that lets foreign code earn un-walled, core-tier speed, which a tab never needs.

## Content vs. code: keep the commons declarative

The durable commons stays declarative typed data, rendered by a standard viewer program, not locked inside a bespoke app. Most of today's web is already program-soup, so "ship a program" is not a regression. But anything you want searchable, archivable, accessible, and re-runnable in 200 years must stay declarative, because programs die with their platform and declarative content plus a frozen spec survive. The web's original sin was letting code metastasize over content, which measurably wrecked accessibility, indexability, and performance. Cathedral splits them, declarative content plus capability-confined program islands instead of declarative content plus ambient-authority JS, which leaves the web cleaner rather than dead.

## Frictionless + safe: the web-tier default

The web's magic is zero trust decisions: click, it runs, it's gone, no install prompt. Cathedral preserves that with a near-empty **web-tier default capability set**, frictionless and safe by structure. A shipped artifact runs by default with display plus sandboxed compute and nothing ambient (no files, no network beyond its origin, no devices), exactly like a web page, and escalates (camera, storage) only through the picker.

## 200-year durability

Durability comes from the artifact being a frozen, formally-specified, content-addressed core IR. A native binary dies with its ISA; a specified IR can be re-implemented by any future host from the spec alone. Freeze the core IR semantics as a permanent commons artifact, and the durable web is declarative content plus the frozen spec: programs ephemeral, format and data eternal. This is the same "freeze a small core, evolve above it" discipline as the bootstrap/TCB story.

## Profile-guided artifact replacement (it falls out of hot-swap)

Because the IR is retained as the re-compilable source, later package or update builds may use an exported workload profile to produce a better AOT artifact. Cathedral itself does not compile that IR into host code while the component is running. Every replacement arrives through ordinary artifact validation, admission, and quiescence ([[updates_and_hot_swap]]):

1. the package ships a prebuilt baseline artifact;
2. observe hot paths / input profile (the scheduler + observability already see them);
3. feed an authorized profile into a later external/package build;
4. verify, validate, and admit the resulting immutable artifact;
5. hot-swap the running component to it through the normal replacement path.

This beats V8 and HotSpot in three ways. The IR is the re-compilable source (you cannot re-optimize a binary you have no IR for), a second justification for shipping IR over native. A re-opt swap is the trivial identity hot-swap: same data shape, no state transform. And hot-swap replaces deopt. Instead of speculative per-call-site guards plus deoptimization, each build is proved equivalent (changes speed, never behavior), and on profile drift you swap the whole component to a build optimized for the new profile. Omega does soundly what V8 does speculatively, because the types it would speculate on are proven. Optimized builds cache by `(IR hash + profile)`, so the OS accumulates fast builds of popular artifacts over time.

Constraints: swaps happen at quiescence (between messages or at a semantic suspension point, not wherever the timer can preempt), and only for hot, long-lived components (`remaining runtime × speedup > recompile cost`). Determinism and the TCB stay intact, because re-optimization moves timing, never behavior.

## Non-goals and limits

- **You don't kill the global web.** Network effect, cross-OS commons, walled-garden risk. You ride it (HTTP transport, WASM legacy tier) and are the best host. The legacy web runs in the sandbox/legacy-box ([[compatibility_and_legacy]]).
- **Keep it ungated.** The right to run unsigned artifacts is non-negotiable, or you rebuild the app-store gatekeeper the web exists to escape.
- **Legacy tier is coarse.** On a dumb host you get declared-manifest to sandbox-permission gating, not Cathedral-grade fine capabilities. Best-effort, like WASM today.

## Open questions

- **Where to freeze the core IR.** Too low is target-coupled and unstable; too high means the host does all codegen and more per-load work. The one real dial.
- **The portable capability-manifest format.** Express "this program needs these capabilities" so that Cathedral enforces it natively, a 2026 browser maps it to sandbox permissions, and a 2226 host can still parse it. This is the serialized-capability load-bearing hole ([[distributed_boundary]], gap register) wearing a web hat.
- **The content/code boundary.** Exactly where declarative content ends and the confined program-island begins, the thing the web got wrong.
- **Optimized-build cache privacy.** Sharing `(IR + profile)` builds across users leaks profile info; keep local or aggregate carefully.

## Related
- [[web_integration]] — the current-web integration chapter (origin-as-principal, WebView); this doc is the radical-future sibling.
- [[compatibility_and_legacy]] — the sandbox-to-VM continuum that runs the legacy web.
- [[updates_and_hot_swap]] — the hot-swap machinery the re-optimization rides.
- [[capability_model]] — the recursive-provider / sandbox-host gatekeeper pattern.
