# Legacy timer-root source canary

This host-side canary compiles Cathedral's first maskable x86 interrupt-root
schema and checks the compiler artifacts that later installation must consume.
It pins the core `InterruptEntry::enter` requirement, exact linear
`InterruptAcknowledgement::Pending` admission, Cathedral's maskable-IRQ stack
class, masked preemption, deriver-owned interrupt return, and the selected
non-suspending/non-blocking `PortIo` provider body.

Run:

```sh
tools/legacy-timer-root-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order.

The canary installs no IDT, executes no `lidt`, unmasks no IRQ, enables no CPU
interrupts, and does not boot QEMU. Those steps remain ordered after table and
stack materialization.
