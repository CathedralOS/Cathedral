# Timer-root fixed-work source canary

This compile-only harness checks Cathedral's preallocated per-CPU timer-root
wake state and its bounded publication leaf. The exact checked body first
stores one already-captured monotonic time observation, then atomically ORs one
wake bit with `Publish` ordering. Repeated arrivals therefore coalesce instead
of appending to an unbounded hard-root queue. The machine uses a shared receiver,
does not allocate, call a scheduler or provider, suspend, block, crash, or loop,
and has one complete frame containing only the two atomic fields.

Run:

```sh
tools/timer-root-work-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order.

This is the executable fixed-work leaf, not a live root. Clock capture remains
a separate provider operation, and the installed-root path cannot yet supply
preallocated state as an internal claim. The canary therefore does not add a
false hardware parameter to `InterruptEntry::enter`, install an IDT entry,
unmask IRQ0, or enable CPU interrupts.
