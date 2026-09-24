# Chapter 01: Debugging & Tracing

> How a developer observes, pauses, and steps a live Cathedral component without a per-breakpoint kernel round-trip, without stopping the world, and without ambient root. Everything here is up for reconsideration.

## The Legacy Model

Legacy debugging is built on `int3`. The debugger overwrites an instruction with a one-byte software breakpoint that traps into the kernel. Every hit is a kernel round-trip: trap, context-switch to the debugger, evaluate the predicate there, resume. A conditional breakpoint ("stop when `id == 4071`") pays the full round-trip on every iteration, so it is often unusably slow on a hot loop. The whole mechanism also rides on ambient authority. `ptrace` and root let one process read and rewrite another's memory because of who it is, not because of anything it holds. Tracing (`strace`, `dtrace`) is a separate, line-oriented, ephemeral stream: not structured, not durable, not replayable.

## The Cathedral Model

Cathedral reconsiders the whole stack. A breakpoint predicate is compiled and evaluated in-process, with no kernel round-trip per hit, so conditional breakpoints and data watchpoints are cheap. Debugging supports time travel: deterministic replay of a recorded run, shared with [[testing_and_simulation]], so a bug is re-examined rather than re-hunted. Tracing is durable structured events rather than a text stream, feeding the same pipeline as observability ([[observability_and_introspection]]). Debugging is hot-swap-aware: you can debug across a live migration, over versioned state, while the component upgrades underneath you.

The debugger is itself authority. Debugging component `X` is `Capability<Debug<X>>`, granted, attenuated, leased, and revoked through the capability model, never ambient `ptrace`. Live introspection is possible without stopping the world. Pausing is one mode, not the only one.

## Debug authority is "become X": governed, not contained

```omega
// Debug authority is a held, attenuable value — not a uid check.
Capability<Debug<PaymentService>>      // full: pause, step, read AND write/inject
attenuate -> Capability<Observe<PaymentService::Events>>  // read-only trace, no write
```

A full `Debug<X>` is not "read X." It includes write and code injection, which is running arbitrary code as X, so it inherits every capability X holds. Debug a service that holds a network flow and a `Charge` capability, and the injected predicate can exfiltrate through X's own network. No sandbox makes this safe, because real debugging needs write: set a variable, skip a frame, fix and continue. The model is therefore governance rather than containment.

- **Read-only is a capability, not the headline.** `Observe<X::Events>` is the attenuated tier, and it is safe. It is offered, but most debugging wants write, so the grant that matters is the dangerous one.
- **Consent at attach.** Attaching a debugger for the first time raises a permission gesture that enumerates the full capability bundle the session will inherit: X's flows, secrets, charge authority. It is the same human-permission surface as any other grant ([[human_permission_ux]]). The user consents to "become X" while seeing what that means.
- **The danger is legible.** "How dangerous is `Debug<X>`?" is the query "what does X hold?" Debugging a sandboxed component is cheap. Debugging a network-and-secrets service is a high-stakes grant, and the system can tell which is which, so it gates the latter like an irreversible action.
- **The floor that does hold.** The injected code cannot exceed X's authority. It runs as X, confined by X's own capability bundle, never above it.

## The fast path: evaluate the predicate in-process

The expensive part of a conditional breakpoint was never the trap as such. It is that the condition is evaluated in the debugger, a separate process, so a hot-loop breakpoint pays a full cross-process round-trip on every iteration even when the condition is false. Fleury's RAD Debugger pays about 2 s for 10 000 iterations. The fix is to move the predicate into the target process and escalate to the debugger, one round-trip, once, only on a true hit. GDB's in-process agent and "fast tracepoints" already ship this, and the `UDB` debugger measures it at about 1000×.

Cathedral does this as a built-in operation rather than the `ptrace` plus library-injection hack legacy systems force, because it owns the loader and the address space. On a hit the kernel's handler reflects back into the same process, to a small **debug agent** that holds only the debug capability's authority. The agent evaluates the predicate locally against the stopped thread and resumes without a second process. Two details decide whether it is fast.

- **Continue without a second trap.** The naive resume restores the original byte, single-steps, and re-arms `int3`. That single-step is itself a kernel trap, which doubles the cost. The agent instead keeps the overwritten instruction in a small trampoline and executes it out of line, relocating it if it is instruction-pointer-relative, then jumps back. Each hit is one entry rather than two.
- **The predicate is lowered differently for native and foreign code.** For an Omega component the toolchain compiles the predicate inline and type-checks it against the component's own `data` fields. For a foreign binary the agent runs a portable bytecode virtual machine, which is GDB's approach. The condition compiles to platform-independent bytecode the agent interprets, so no per-platform compiler is needed in the target.

### Fleury's plea: "user-level `int3`", and the two costs inside it

Working debugger authors ask OS developers for user-level `int3`: let the breakpoint trap be handled in the same process instead of forcing `int3`, then the kernel, then waking the separate debugger on every hit. The plea bundles two distinct costs, and only one is the OS's to give.

- **The cross-process round-trip** is pure OS software: the kernel waking a separate debugger process and servicing its register reads. It is removable, and it is the real ask. Cathedral removes it by construction. The in-process agent means the trap is serviced in the target, with no second process. This half is granted everywhere.
- **The ring transition** is hardware. On stock x86 and ARM the breakpoint exception is routed to ring 0 or EL1, and no OS can deliver it straight to user mode. It is not removable in software. Only binary rewriting (replacing the trap with a branch) or a custom ISA with a user-mode exception vector removes it.

Cathedral grants the round-trip half everywhere and the ring-transition half only on a custom ISA. The matrix below shows where each lands.

## Trap-free breakpoints are an ISA-dominated story, not a pre-baked-probe one

A runtime breakpoint can be set at any address, so the trap-free path cannot be a probe the compiler baked in ahead of time. There is no way to pre-reserve every site, and breakpoints are placed long after compilation. The trap-free mechanism is instead runtime in-place patching. At the moment the breakpoint is set, the instruction at that address is overwritten with a branch to a freshly allocated trampoline holding the predicate and the relocated original. Whether that patch is clean is decided by the instruction set, not by who compiled the code. The compiler's role is to hand over **safe-patch metadata** (instruction boundaries, instruction-pointer-relative sites, relocation), not to pre-bake probes.

Common to every cell below: the cross-process round-trip is gone because of the in-process agent, the predicate is evaluated in-process and only escalates on a true hit, debug is a held capability, and the displaced instruction goes to a trampoline. The table shows only what differs.

| Target (ISA · code) | Trap-free runtime breakpoint? | Safe in-place single-instruction patch? | Kernel entry on hot/false path | Predicate lowered to | Default → trap-free path |
|---|---|---|---|---|---|
| x86 · Omega | Not by plain patch | No: a 5-byte `jmp` clobbers trailing bytes, and code may branch into the patch | One per hit (`int3` default); zero if recompiled or rewritten | Inlined native (toolchain codegen, type-checked vs `data`) | `int3`, in-process handled. Trap-free only via recompile-the-function-and-hot-swap, coarse pre-reserved slots, or dynamic binary instrumentation |
| x86 · Non-Omega | Only via heavy dynamic binary instrumentation | No: the same hazard, and no metadata, so variable-length x86 must be decoded | One per hit (`int3`) unless instrumented | Portable bytecode in the injected agent's virtual machine (GDB-style) | `int3` plus bytecode VM. Trap-free only via full dynamic binary instrumentation (relocate to a code cache) |
| ARM64 · Omega | Yes | Yes: fixed-width, atomic `NOP`↔`B`, architecturally blessed under concurrent execution | None (stays in EL0) | Inlined native; instruction-pointer-relative fixups known from metadata | In-place branch patch at runtime, no pre-reservation. `BRK` fallback only for an un-relocatable site |
| ARM64 · Non-Omega | Yes | Yes: fixed-width; instruction-pointer-relative fixup recovered by (easy) decode | None | Portable bytecode in the injected agent's virtual machine | In-place branch patch (decode-driven). `BRK` fallback for an un-relocatable site |
| Custom ISA · either | Yes, including the actual stop | Yes: fixed-width, with a reserved breakpoint encoding by design | None, ever | Inlined (Omega) / virtual machine (foreign) | User-mode breakpoint exception vector: literal user-level `int3`, zero ring transition |

The shape of the differences:

- **ISA is the dominant axis, not Omega-ness.** ARM64 already gives trap-free runtime breakpoints to foreign code. x86 denies them even to Omega code without recompile-and-swap or dynamic binary instrumentation. Fixed instruction width matters more than provenance.
- **Omega-ness changes only two things.** The predicate is inlined native instead of running in a bytecode virtual machine, and the safe-patch metadata is handed over instead of recovered by decoding. On ARM that is a convenience. On x86 it is what makes coarse pre-reserved slots and recompile-and-swap possible at all.
- **Only a custom ISA removes the last kernel entry,** the one on a true stop, by delivering the breakpoint exception straight to a user-mode handler. Everywhere else a true hit still takes one ring transition. The trap-free rows only remove it from the false, hot path.
- **The x86 default is `int3` with in-process handling,** the achievable half of Fleury's plea, not trap-free. An Omega component on x86 reaches trap-free only by recompiling the affected function with the predicate inlined and hot-swapping it in. That is real, but it pays a recompile plus a swap per breakpoint set, so it is a hot-loop tool, not the default.
- **The "Omega" rows assume walled native code.** Raw in-place patching is forbidden for code running in the single address space (next section). Those rows describe walled or temporarily walled native code, where the raw path is safe.

### Native code in the SAS: checked instrumentation, never raw patches

The matrix's raw in-place patching assumes you may write a branch and a trampoline into the target's memory. In the single address space that is forbidden. The SAS is safe for one reason, that only checker-verified code is ever admitted, and an unchecked trampoline injected into it would be a universal escape, with no MMU wall and no proof bounding it. SAS-native trap-free debugging therefore installs **checked instrumentation**: a recompiled function that is re-checked and hot-swapped, or a toolchain-generated, checker-verified patch. Never raw bytes. Because that instrumentation is checked, the checker confines it to the debugged component's references and capabilities. That is why `Capability<Debug<X>>` stays bounded to X in the SAS instead of becoming access to the whole address space.

The cheap raw path (`int3`, hand-written trampolines) is walled-only. Inside a hardware wall it is fine, because injection compromises only that already-untrusted wall. The practical consequence is a placement choice. To debug an app, temporarily wall it for the session: drop it to an MMU domain, use the cheap raw mechanism, and re-admit it afterward. Walled execution is slower while debugging, which is acceptable. Only the SAS core itself must pay checker-verified instrumentation per breakpoint, because it cannot be walled: it is the wall-enforcer. That is also where you want every instruction checked anyway, and proof caching keeps re-checking a one-function patch cheap.

This is one of the SAS's accumulating costs, and a real data point in the MMU→CHERI vs SAS-via-PCC question ([[kernel_architecture]]). Cheap in-place debugging is a property the walled and CHERI worlds get for free and the proof-isolated SAS must work to recover.

## Stopping, snapshots, and swapping under a live system

Stopping a component means stopping its time, and it is all-stop. A breakpoint stops all the component's threads, because freezing a single thread is almost never useful. The debugger hosts the component as a Matrix and owns its clock ([[time_and_clocks]]), so "stopped" means the component's virtual time does not advance. A hot-swap requested against a stopped component is frozen: requested but not running, since it cannot apply while time is stopped. On resume, time flows, the pending swap applies, and breakpoints over changed code get the standard "binary changed, this breakpoint moved or is no longer active" re-bind-or-invalidate treatment. When a walked-away session blocks a critical patch, the valve is not to force the swap through the freeze. The host revokes the debug Matrix, since the grant is revocable, tears the session down, and relaunches patched.

Consistent state without stopping is a separate, best-effort mode. Live introspection (production tracing, looking without disturbing) does not freeze. It takes a best-effort **consistent cut** in the Chandy–Lamport sense: each task records at its own next transition boundary, and the in-flight channel messages are captured. That is well-defined because Omega is message passing over owned state with no shared mutable memory. A task wedged in a long IO or foreign loop is reported as a straggler and never allowed to stall the snapshot, so a live peek never inherits the freeze path's stall risk.

## Debugger as a Matrix

Replay, time travel, and fault injection are one mechanism. The debugger hosts the component in a Matrix and owns its clock, randomness, network, and input ([[testing_and_simulation]]). Replay serves the recorded input stream. Time travel, "what if time jumped," and inject-a-partition serve modified inputs. Adversarial testing serves hostile inputs. The debugger, the replayer, and the hostile simulator are the same recursive provider serving a synthetic world. Recording is affordable because Cathedral is deterministic given its effects: you record the bounded nondeterministic input stream (clock, randomness, message arrivals, scheduler interleavings), not all state. Production runs a light structured trace always on, with full recording armed by the debug grant. The simulator records fully.

## Cross-version breakpoints: the OS supplies material, the debugger author builds the format

A breakpoint expressed against the source state graph or `data` fields survives a hot-swap by riding the migration's field correspondence, the stable field numbers between V1 and V2. A field the migration removes flags its breakpoint. A migration that restructures state, by splitting a field or computing one, needs a **debug projection** carried alongside it: how a V1 watchpoint maps onto V2. Turning this into an evolving, fast debug-info format is a debugger-author follow-up, not an OS deliverable. The OS supplies the raw material: native IR, immutable historical schemas with stable field numbers, and checked conversion and replacement machines. RAD Debugger's RDI is the prior art to study. It discarded PDB and DWARF for these speed-and-structure reasons.

## Concerns & Design Space

- **Data / watchpoints.** Triggered on typed-state mutation, expressed against Omega `data` fields rather than raw addresses.
- **Structured durable tracing.** Trace events use ordinary numbered schemas with a selected durable codec, are written to the observability pipeline, and remain queryable after the fact ([[observability_and_introspection]]).
- **State-graph debugging.** Step the source state graph and inspect the lowered graph. Both are Omega artifacts, so "which state am I in, which transition fired" is answered directly rather than reconstructed from a stack.
- **Zero value.** A zero `Capability<Debug<X>>` is the inert null-object capability ([[omega_substrate]]). It holds no debug authority, so pause, step, and read are accepted and return nothing rather than crashing or escalating. Zero debug authority is the same value as least privilege, the natural default for a component handed no debug grant.

## Key Questions

- Can a breakpoint predicate be sandboxed? No. A full `Debug<X>` is "become X": write and inject running as X, inheriting all of X's capabilities, bounded only by X's own authority, and not containable, because debugging needs write. The defense is the attach-time consent gesture that enumerates the inherited bundle, the legibility of the danger (what X holds), and `Observe<X::Events>` as the attenuated read-only tier.
- How is a consistent snapshot taken? In two modes. Debugging freezes: all-stop, and stopping the Matrix stops the component's time. Live introspection takes a best-effort consistent cut, Chandy–Lamport over the message graph with stragglers flagged rather than stalled, which the no-shared-mutable-memory model makes well-defined.
- How does a breakpoint survive a version change? By riding the migration's field correspondence. A restructuring migration carries a debug projection. The versioned debug-info format is a debugger-author follow-up (study RDI), with the OS supplying IR, versioned schemas, and the migration.
- How much must replay record? The bounded nondeterministic input stream, since the component is deterministic given its effects, not all state. Production runs a light always-on trace with full recording armed by the debug grant, and the simulator records fully.
- What happens to a hot swap during a debug session? A stopped component has stopped time, so a pending swap is frozen-requested and applies on resume, re-binding or invalidating breakpoints. A critical patch blocked by a walked-away session is handled by the host revoking the debug Matrix, not by forcing the swap.
- Is the last kernel entry removable? Not on stock hardware for trap-based breakpointing, since the ring transition is hardware. Trap-free runtime breakpoints come from in-place patching, clean on ARM64 and hazardous on x86, where the options are dynamic binary instrumentation or recompile-and-swap. Literal user-level `int3`, with zero ring transition even on a true stop, needs a custom ISA.

## Omega Leverage

- Inspectable source and lowered state graphs ([Omega Chapter 4: States And Transitions](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_4_states_transitions.md)) make "where is execution, and why" a query over a real artifact.
- Capabilities as values make `Capability<Debug<X>>` an ordinary, attenuable grant rather than a new privileged syscall.
- Virtual time plus deterministic graphs make replay debugging sound, and the debugger-as-Matrix owns clock, randomness, network, and input.
- Historical state schemas plus checked replacement let a session survive a hot swap and give cross-era breakpoints their field correspondence.
- Omega likely must grow a sanctioned in-process probe form, meaning compiled predicates with their own effect and authority ceiling, plus the safe-patch metadata the runtime patcher needs (instruction boundaries, instruction-pointer-relative sites). Foreign predicates lower to a portable bytecode the agent interprets; native predicates inline.

## Open Questions

- On x86, is per-breakpoint recompile-and-hot-swap worth its cost on a hot loop, versus accepting the one `int3` kernel entry? This is a real tradeoff with no default yet.

## Related
- [[observability_and_introspection]] — tracing as durable structured events.
- [[testing_and_simulation]] — deterministic replay and the synthetic-world machinery.
- [[time_and_clocks]] — virtual time the debugger-as-Matrix owns.
- [[capability_model]] — debug as a held, governed capability.
- [[human_permission_ux]] — the attach-time consent gesture for "become X".
- [[updates_and_hot_swap]] — debugging across a live swap.
