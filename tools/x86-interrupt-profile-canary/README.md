# x86 interrupt-profile source canary

This compile-only harness checks Cathedral's pure initial vector-to-stack
policy. The complete bootstrap exception floor is one total policy over slots
0–31: every slot is fatal-diagnostic, NMI/double-fault/machine-check derive
their normalized dedicated stack and hardware IST from the coupled class/index
records 2, 1, and 3 respectively, and every other slot uses the interrupted
kernel stack with IST zero. The harness also pins the first remapped legacy
timer to the shared maskable-IRQ class/index 4.

Run:

```sh
tools/x86-interrupt-profile-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order. Typed-artifact validation uses
`jq` and fails with an explicit dependency error when it is unavailable.

This canary grants no stack storage, root admission, IDT/TSS access, interrupt
authority, or machine control. It validates the authored policy data that later
WCSU/root admission and gate materialization must consume together.
