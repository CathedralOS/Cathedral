# Legacy timer-root source canary

This host-side canary compiles Cathedral's first maskable x86 interrupt-root
schema and checks the compiler artifacts that later installation must consume.
It pins the core `InterruptEntry::enter` requirement, exact linear
`InterruptAcknowledgement::Pending` admission, Cathedral's maskable-IRQ stack
class, masked preemption, deriver-owned interrupt return, and the selected
non-suspending/non-blocking `PortIo` provider body. It also uses Omega's generic
`inspect-terminal --machine LegacyPicTimerRoot::enter` surface to verify the
actual checked hard-root closure: root claim transfer to `Pic8259`, exact
`PortWrite(0x20, 0x20)` authority, acknowledgement boundary settlement, and
value-less normal return. The closure cardinality is pinned as well as its
ordering, so an extra machine, port write, settlement, or terminator fails the
canary instead of hiding behind the required rows.

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
