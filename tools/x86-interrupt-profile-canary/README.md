# x86 interrupt-profile source canary

This compile-only harness checks Cathedral's pure initial vector-to-IST policy.
It pins the four v1 assignments as single records: double fault, NMI, and
machine check use distinct class/index pairs 1/2/3, while the first remapped
legacy timer uses the shared maskable-IRQ class/index 4.

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
