# Chapter 39: Debugging & Tracing

> How a developer observes, pauses, and steps a live Cathedral component — without a kernel round-trip per breakpoint hit, without stopping the world, and without ambient root. Everything here is up for reconsideration.

## The Legacy Contract

Legacy debugging is built on `int3`: the debugger overwrites an instruction with a one-byte software breakpoint that traps into the kernel. Every hit is a kernel round-trip — trap, context-switch to the debugger, evaluate the predicate there, resume. This makes a *conditional* breakpoint ("stop when `id == 4071`") pay the full round-trip on every iteration, so it is often unusably slow on a hot loop. Worse, the whole mechanism rides on ambient authority: `ptrace`/root lets one process read and rewrite another's memory because of *who it is*, not because of anything it *holds*. Tracing (`strace`, `dtrace`) is a separate, line-oriented, ephemeral stream — not structured, not durable, not replayable.

## What Cathedral Wants

Reconsider the whole stack. A breakpoint predicate should be **compiled and evaluated in-process** — no kernel round-trip per hit — so conditional and data/watchpoints are cheap. Debugging should support **time-travel**: deterministic **replay** of a recorded run (shared with [[40_testing_and_simulation]]) so a bug is re-examined, not re-hunted. Tracing should be **durable structured events**, not a text stream, feeding the same pipeline as observability ([[33_observability_and_introspection]]). And debugging must be **hot-swap-aware**: you can debug across a live migration, over versioned state, while the component upgrades underneath you.

Critically: **the debugger is itself authority.** Debugging component `X` is `Capability<Debug<X>>`, granted, attenuated, leased, and revoked through the capability model — never ambient ptrace. Live introspection should be possible *without stopping the world*; pausing is one mode, not the only one.

```omega
// Debug authority is a held, attenuable value — not a uid check.
Capability<Debug<PaymentService>>            // full: pause, step, read state
attenuate -> Capability<Observe<PaymentService::Events>>  // read-only trace
```

## Concerns & Design Space

- **In-process predicates.** Compile the breakpoint condition into the component's own evaluation context so the common case (predicate false) never leaves the process. The debugger is notified only on a true hit.
- **Data / watchpoints.** Triggered on typed-state mutation, expressed against Omega `data` fields rather than raw addresses.
- **Time-travel / replay.** Re-run a deterministic recording under virtual time ([[12_time_and_clocks]]); the same machinery [[40_testing_and_simulation]] uses.
- **Structured durable tracing.** Trace events are versioned `wire data`, written to the observability pipeline, queryable after the fact ([[33_observability_and_introspection]]).
- **State-graph debugging.** Step the *source* state graph and inspect the *lowered* graph — both are first-class Omega artifacts, so "which state am I in, which transition fired" is answerable directly, not reconstructed from a stack.
- **Hot-swap-aware debugging.** Debug across a swap: hold a debug session while the component migrates versioned state, and map breakpoints across versions ([[23_updates_and_hot_swap]]).
- **Debug as governed authority.** `Capability<Debug<X>>` is auditable in the authority graph; a production debug session is a recorded, revocable grant.

## Key Questions

- What can an in-process predicate safely touch, and how is *it* sandboxed so a debug probe cannot become a backdoor into the component's authority?
- Does live introspection without pausing need a consistent snapshot, and how is that taken cheaply over typed state?
- How are breakpoints and watchpoints expressed across a version boundary when the underlying `data` shape changed mid-session?

## Omega Leverage

- **Inspectable source + lowered state graphs** ([../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md](../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md)) make "where is execution, and why" a query over a real artifact.
- **Capabilities as values** make `Capability<Debug<X>>` an ordinary, attenuable grant rather than a new privileged syscall.
- **Virtual time + deterministic graphs** make replay debugging sound.
- **Versioned data + migration** let a session survive a hot swap.
- Omega likely must grow a sanctioned **in-process probe** form — compiled predicates with their own effect/authority ceiling — so debugging cannot smuggle authority past the model.

## Open Questions

- Is record-everything replay affordable in production, or only in the simulator, with a lighter always-on trace in production?
- Can an outstanding debug borrow block a hot swap, and is that acceptable back-pressure (mirrors [[04_capability_lifecycle]]'s borrow-vs-swap tension)?

## Related
- [[33_observability_and_introspection]] — tracing as durable structured events.
- [[40_testing_and_simulation]] — deterministic replay machinery.
- [[12_time_and_clocks]] — virtual time under replay.
- [[03_capability_model]] — debug as a held, governed capability.
- [[23_updates_and_hot_swap]] — debugging across a live swap.
