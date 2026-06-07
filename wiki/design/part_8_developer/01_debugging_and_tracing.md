# Chapter 01: Debugging & Tracing

> How a developer observes, pauses, and steps a live Cathedral component — without a kernel round-trip per breakpoint hit, without stopping the world, and without ambient root. Everything here is up for reconsideration.

## The Legacy Model

Legacy debugging is built on `int3`: the debugger overwrites an instruction with a one-byte software breakpoint that traps into the kernel. Every hit is a kernel round-trip — trap, context-switch to the debugger, evaluate the predicate there, resume. This makes a *conditional* breakpoint ("stop when `id == 4071`") pay the full round-trip on every iteration, so it is often unusably slow on a hot loop. Worse, the whole mechanism rides on ambient authority: `ptrace`/root lets one process read and rewrite another's memory because of *who it is*, not because of anything it *holds*. Tracing (`strace`, `dtrace`) is a separate, line-oriented, ephemeral stream — not structured, not durable, not replayable.

## The Cathedral Model

Reconsider the whole stack. A breakpoint predicate should be **compiled and evaluated in-process** — no kernel round-trip per hit — so conditional and data/watchpoints are cheap. Debugging should support **time-travel**: deterministic **replay** of a recorded run (shared with [[testing_and_simulation]]) so a bug is re-examined, not re-hunted. Tracing should be **durable structured events**, not a text stream, feeding the same pipeline as observability ([[observability_and_introspection]]). And debugging must be **hot-swap-aware**: you can debug across a live migration, over versioned state, while the component upgrades underneath you.

Critically: **the debugger is itself authority.** Debugging component `X` is `Capability<Debug<X>>`, granted, attenuated, leased, and revoked through the capability model — never ambient ptrace. Live introspection should be possible *without stopping the world*; pausing is one mode, not the only one.

```omega
// Debug authority is a held, attenuable value — not a uid check.
Capability<Debug<PaymentService>>            // full: pause, step, read state
attenuate -> Capability<Observe<PaymentService::Events>>  // read-only trace
```

## How the in-process predicate actually works

This has to work for any compiled code, including a C++ app the Omega toolchain never saw, so the mechanism is deliberately language-agnostic.

The breakpoint stays a one-byte `int3` (or `BRK` on ARM). A one-byte patch is atomic and clobbers nothing around it, which a five-byte `jmp` patch cannot promise. What changes is where the trap goes. On x86 and ARM a breakpoint exception always enters the kernel first, because exception vectors are kernel-owned and there is no way to deliver `int3` straight to user code. The expensive part of the legacy path is everything after that kernel entry: stop the target, wake the debugger process, schedule it, have it read and write registers over a syscall interface, evaluate the predicate there, and resume.

So the kernel's breakpoint handler instead reflects the hit back into the *same* process, to a small **debug agent** the OS loaded into that process's address space. No context switch, no second process, no ptrace. The agent evaluates the compiled predicate locally against the stopped thread's registers and memory, and the common case where the predicate is false resumes without ever leaving the process. Only a true hit escalates to the real debugger over IPC. A hot-loop conditional breakpoint then pays one cheap kernel entry per iteration instead of a full cross-process round trip.

Two details decide whether this is actually fast:

- **Continue without a second trap.** The naive resume restores the original byte, single-steps, and re-arms `int3`, but that single-step is itself a kernel trap, doubling the cost. The agent instead keeps the overwritten instruction in a small trampoline and executes it out of line (relocating it if it is instruction-pointer-relative), then jumps back, so each hit is one entry rather than two.
- **The agent is confined.** It runs in the target's address space but holds only the debug capability's authority, nothing ambient, so a predicate cannot read or move authority the debugger was not granted. This is the same question the capability model already forces ([[capability_model]]): an in-process probe is sandboxed against the component it lives in.

The zero-trap extreme is full dynamic binary instrumentation: rewrite the whole function once, inline the predicate, and run at near native speed with no per-hit trap at all. That buys the most speed and pays for it in complexity, because the five-byte-jump hazard is now real (relocating instructions, fixing branches that target into the patched region, quiescing threads mid-patch). It is the right tool when even one kernel entry per hit is too much, and overkill otherwise. The reflected-`int3` path and the rewrite path are two points on one curve, chosen by how hot the site is.

Because Cathedral owns the loader and the address space, loading the per-process agent and registering the fast reflection is a first-class operation rather than the injection hack `ptrace` forces on legacy systems. None of it depends on the app being written in any particular language.

## Exploratory: trap-free instrumentation for toolchain-compiled code

For code the Omega toolchain compiled, which includes the OS itself and Omega apps, the trap can disappear entirely. This part is speculative rather than core.

OS components are language-isolated in one address space ([[kernel_architecture]]), so there is no ring boundary between a debug agent and the code it watches, and a breakpoint can be a compiler-inserted guarded call rather than an `int3` at all. The predicate is compiled inline at the site, type-checked against the component's own `data` fields, and proven not to disturb the surrounding code. No exception, no kernel entry, no out-of-line dance, and the cost of a false predicate drops to a branch.

This only applies where the toolchain controls codegen, so it never covers a C++ binary or any foreign code, which always falls back to the reflected-`int3` mechanism above. It is attractive precisely for the hardest layer to debug, the OS's own components, and it is what lets the source and lowered state graphs be stepped directly. Whether the toolchain should carry a sanctioned probe form with its own effect and authority ceiling, so this cannot smuggle authority past the model, is the open language question.

## Concerns & Design Space

- **In-process predicates.** Compile the breakpoint condition into the component's own evaluation context so the common case (predicate false) never leaves the process. The debugger is notified only on a true hit.
- **Data / watchpoints.** Triggered on typed-state mutation, expressed against Omega `data` fields rather than raw addresses.
- **Time-travel / replay.** Re-run a deterministic recording under virtual time ([[time_and_clocks]]); the same machinery [[testing_and_simulation]] uses.
- **Structured durable tracing.** Trace events are versioned `wire data`, written to the observability pipeline, queryable after the fact ([[observability_and_introspection]]).
- **State-graph debugging.** Step the *source* state graph and inspect the *lowered* graph — both are first-class Omega artifacts, so "which state am I in, which transition fired" is answerable directly, not reconstructed from a stack.
- **Hot-swap-aware debugging.** Debug across a swap: hold a debug session while the component migrates versioned state, and map breakpoints across versions ([[updates_and_hot_swap]]).
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
- Can an outstanding debug borrow block a hot swap, and is that acceptable back-pressure (mirrors [[capability_lifecycle]]'s borrow-vs-swap tension)?
- Is the last per-hit kernel entry removable on stock hardware for foreign code, or does fully trap-free conditional breakpointing there always require binary rewriting?

## Related
- [[observability_and_introspection]] — tracing as durable structured events.
- [[testing_and_simulation]] — deterministic replay machinery.
- [[time_and_clocks]] — virtual time under replay.
- [[capability_model]] — debug as a held, governed capability.
- [[updates_and_hot_swap]] — debugging across a live swap.
