# x2APIC provider contract source canary

This compile-only harness checks Cathedral's four checked x2APIC provider
leaves: configure a divided one-shot, arm it, stop it, and realize the timer
acknowledgement. It pins their separation, input domains, checked `wrmsr`
sequences, and retained `MachineControl` reach.

Run:

```sh
tools/x2apic-provider-contract-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order. Artifact validation uses `jq`
and fails with an explicit dependency error when it is unavailable.

The canary compiles and inspects provider code but never calls it. It admits no
x2APIC mode, calibrates no clock, selects no root, publishes no IDT entry,
enables no interrupt, and grants no `MachineControl` capability.
