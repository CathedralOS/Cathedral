# Timer-root fixed-work source canary

This compile-only harness checks Cathedral's preallocated per-CPU timer-root
wake state and its bounded producer/consumer handoff. The publication leaf
first stores one already-captured monotonic time observation, then atomically
ORs one wake bit with `Publish` ordering. The ordinary-task leaf atomically
claims and clears that bit with `Receive` ordering, loads the observation only
after a successful claim, and returns either `Claimed` or `Idle`. Repeated
arrivals therefore coalesce instead of appending to an unbounded hard-root
queue. Both machines use a shared receiver, do not allocate, call a scheduler
or provider, suspend, block, crash, or loop, and publish complete frames over
only the two atomic fields.

Run:

```sh
tools/timer-root-work-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order.

These are executable fixed-work leaves, not a live root or timer-service task.
Clock capture remains a separate provider operation, fan-out remains ordinary
task work, and the installed-root path cannot yet supply preallocated state as
an internal claim. The canary therefore does not add a false hardware parameter
to `InterruptEntry::enter`, install an IDT entry, unmask IRQ0, or enable CPU
interrupts.
