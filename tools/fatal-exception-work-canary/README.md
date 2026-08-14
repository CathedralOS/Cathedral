# Fatal-exception fixed-work source canary

This compile-only harness checks Cathedral's bootstrap fatal-diagnostic leaf.
The exact checked body records one normalized architectural vector in
preallocated atomic state, publishes a one-bit validity marker with `Publish`
ordering, and then unconditionally `Abort`s. It has no allocation, service or
provider calls, suspension, blocking, loop, cleanup path, or normal return. Its
complete write frame contains only the two diagnostic fields.

Run:

```sh
tools/fatal-exception-work-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order.

This leaf is deliberately below the external entry seam. Its vector argument
is normalized internal input from a future generated per-vector stub, not a
claim that x86 supplies a semantic vector parameter. The canary binds no
exception ABI or preemption plan, creates no handler identity or selector
evidence, provisions no stack, materializes no gate, and installs no IDT.
