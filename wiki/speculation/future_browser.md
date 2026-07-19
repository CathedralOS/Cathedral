# Speculative: Future Browser Design

> **Status: SPECULATIVE — forward-looking exploration, not committed design.** Captures a design conversation (2026-06-15) about what "the browser" becomes on a capability-first OS. Decided chapters it touches: [[web_integration]], [[compatibility_and_legacy]], [[updates_and_hot_swap]], [[capability_model]], and the Omega `design_briefs/verified_gated_ml_optimizer.md` brief. Nothing here is settled; it is a coherent vision to revisit, not a spec.

## The reframe: decompose the browser, don't replace it

The web browser is ~6 distinct functions fused by historical accident:

| What the browser does | On Cathedral |
|---|---|
| Sandbox untrusted code | dissolves into confinement (the OS) |
| Capability-confine (camera/mic prompts) | dissolves into the capability model |
| Ephemeral, zero-install, run-by-reference | an OS *lifecycle* pattern |
| Cross-OS portability | *universality* — the hard remainder |
| Document / hypertext model | *declarative content* |
| Frictionless zero-trust-decision delivery | a *web-tier capability default* |

Cathedral dissolves the top two (isolation, capability-confinement) into the OS. "Kill the browser" is the wrong frame — the browser **decomposes**: its isolation role evaporates, and what remains are real problems that are *not* OS-isolation (portability, a content model, ephemeral delivery, frictionless safety). The browser stops being a special subsystem and becomes two thin shells over OS primitives: a **content viewer** and a **program host**.

## The artifact: Omega IR, tiered fidelity

A web-artifact is **Omega IR** — a frozen, content-addressed, capability-carrying program — not HTML+JS+WASM soup. It runs with *graceful capability-fidelity degradation*:

- **Capable host (Cathedral):** compile the IR to a native exe; run it as a **walled child of a sandbox-host component** (the "browser" = the recursive-provider gatekeeper, [[capability_model]]); grant its capabilities **up front**; it runs at native speed.
- **Legacy host (today's browsers, vestigial future OSes):** lower the IR to **WASM**, interpret/JIT in the existing sandbox. Coarse gating (declared manifest → sandbox perms), but it *runs everywhere* — the "works on vestigial OSes" tier.

Same artifact, two tiers; the capability requirements travel *in* the artifact (legible everywhere, enforced as finely as the host allows). Don't invent a new bytecode — **ride WASM** for the legacy tier (the JVM-applet / Flash / Silverlight graveyard is full of new-bytecode bets): native on Cathedral, WASM everywhere else, Omega as the source/IR that lowers to both.

## Why native tabs are fast (not slow)

The MMU doesn't slow *compute* — translation is transparent (TLB). The only cost is a **boundary crossing** (a syscall/IPC to the gatekeeper), paid *only when you cross*. A well-built gatekeeper grants capabilities **up front** (hand the tab a framebuffer region, a network handle) and stays **out of the hot path** — the tab then touches its granted resources directly at native speed.

So a web-tab is **native exes running, plain and simple** — and *faster* than a real browser, because it skips the JS-interpret/JIT/DOM tax entirely. The "seam cost" only bites *crossing-heavy* hot paths (zillions of tiny syscalls), which a tab isn't. **PCC is not needed** here — the hardware wall + runtime capability-deny already make foreign code safe; proof-carrying code is only the advanced option to let foreign code earn *un-walled, core-tier* speed, which a tab never needs.

## Content vs. code: keep the commons declarative

Most of today's web is already program-soup, so "ship a program" isn't a regression. But for the *durable commons* — anything you want searchable, archivable, accessible, and re-runnable in 200 years — the content must stay **declarative typed data**, rendered by a *standard viewer program*, not locked inside a bespoke app. Programs die with their platform; declarative content + a frozen spec survive. The web's original sin was letting code metastasize over content (measurably wrecking accessibility, indexability, performance); Cathedral splits them cleanly — **declarative content + capability-confined program islands**, instead of declarative content + ambient-authority JS. A *cleaner* web, not a dead one.

## Frictionless + safe: the web-tier default

The web's magic is *zero trust decisions* — click, it runs, it's gone, no install prompt. Preserve it with a **web-tier near-empty default capability set**: a shipped artifact runs by default with display + sandboxed compute and *nothing ambient* (no files, no network beyond its origin, no devices) — safe-by-default exactly like a web page — and *escalates* (camera, storage) only through the picker. Frictionless and safe, structurally.

## 200-year durability

Durability comes from the artifact being a **frozen, formally-specified, content-addressed core IR**. A native binary dies with its ISA; a *specified* IR can be re-implemented by any future host from the spec alone. So: freeze the core IR semantics as a permanent commons artifact; the durable web = declarative content + the frozen spec; the *programs* are ephemeral, the *format and data* eternal. (Same "freeze a small core, evolve above it" discipline as the bootstrap/TCB story.)

## Profile-guided artifact replacement (it falls out of hot-swap)

Because the **IR is retained** as the re-compilable source, later package/update
builds may use an exported workload profile to produce a better AOT artifact.
Cathedral itself does not compile that IR into host code while the component is
running. Every replacement arrives through ordinary artifact validation,
admission, and quiescence ([[updates_and_hot_swap]]):

1. the package ships a prebuilt baseline artifact;
2. observe hot paths / input profile (the scheduler + observability already see them);
3. feed an authorized profile into a later external/package build;
4. verify, validate, and admit the resulting immutable artifact;
5. **hot-swap** the running component to it through the normal replacement path.

Better than V8/HotSpot in three ways: the **IR is the re-compilable source** (you can't re-optimize a binary you don't have the IR for — a second justification for shipping IR over native); a re-opt swap is the **trivial identity hot-swap** (same data shape, no state transform); and **hot-swap replaces deopt** — instead of speculative per-call-site guards + deoptimization, each build is *proved equivalent* (changes speed, never behavior), and on *profile drift* you swap the whole component to a build optimized for the new profile. Omega does *soundly* what V8 does *speculatively*, because the types it would speculate on are proven. Optimized builds cache by `(IR hash + profile)`, so the OS accumulates fast builds of popular artifacts over time.

Constraints: swaps happen at **quiescence** (between messages / at `await` — constant for event-driven tabs, not mid-compute-loop); only worth it for **hot, long-lived** components (`remaining runtime × speedup > recompile cost`). Determinism/TCB intact: re-optimization moves *timing*, never *behavior*.

## Non-goals / honest limits

- **You don't kill the global web.** Network effect + cross-OS commons + walled-garden risk. You *ride* it (HTTP transport, WASM legacy tier) and are the best host; the legacy web runs in the sandbox/legacy-box ([[compatibility_and_legacy]]).
- **Keep it ungated.** The right to run unsigned artifacts is non-negotiable, or you rebuild the app-store gatekeeper the web exists to escape.
- **Legacy tier is coarse.** On a dumb host you get declared-manifest → sandbox-perm gating, not Cathedral-grade fine capabilities. Best-effort, like WASM today.

## Open questions

- **Where to freeze the core IR** — too low = target-coupled + unstable; too high = host does all codegen + more per-load work. The one real dial.
- **The portable capability-manifest format** — express "this program needs these capabilities" so Cathedral enforces it natively, a 2026 browser maps it to sandbox perms, and a 2226 host can still parse it. This *is* the serialized-capability load-bearing hole ([[distributed_boundary]], gap register) wearing a web hat.
- **The content/code boundary** — exactly where declarative content ends and the confined program-island begins (the thing the web got wrong).
- **Optimized-build cache privacy** — sharing `(IR + profile)` builds across users leaks profile info; keep local or aggregate carefully.

## Related
- [[web_integration]] — the current-web integration chapter (origin-as-principal, WebView); this doc is the radical-future sibling.
- [[compatibility_and_legacy]] — the sandbox→VM continuum that runs the legacy web.
- [[updates_and_hot_swap]] — the hot-swap machinery the re-optimization rides.
- [[capability_model]] — the recursive-provider / sandbox-host gatekeeper pattern.
