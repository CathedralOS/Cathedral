# x86 bootstrap stack-class-set source canary

This compile-only harness checks Cathedral's complete pre-provisioning stack
policy. One role-labeled record requires double-fault, NMI, machine-check, and
maskable-IRQ entries together, retaining each coupled analysis class/hardware
IST pair as 1/1, 2/2, 3/3, and 4/4. A pure source validator derives that policy
internally, checks both fields for every role, and returns either the complete
runtime-relevant candidate or rejection. Omitting or swapping a role cannot
produce a policy-consistent partial set.

Run:

```sh
tools/x86-stack-class-set-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order.

The record deliberately carries no byte size or storage. Omega derives stack
demand from WCSU and target calling plans, while the current source surface has
no external-root `StackLease` carrier. This canary therefore grants no stack,
placement, TSS/IDT, entry, gate, selector, root, or installation authority.
