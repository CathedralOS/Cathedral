# local-APIC register-map source canary

This compile-only harness checks Cathedral's pure correspondence table between
xAPIC MMIO register offsets and their x2APIC MSR identities. It pins the nine
architectural pairs currently transcribed in `facts::local_apic`, including the
ID, version, task-priority, EOI, spurious-vector, and timer registers.

Run:

```sh
tools/local-apic-register-map-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order. Typed-artifact validation uses
`jq` and fails with an explicit dependency error when it is unavailable.

The snapshot records addresses only. It performs no CPUID or mode admission,
maps no MMIO page, reads or writes no MSR, configures no timer, sends no EOI,
and grants no `MachineControl`, interrupt, or acknowledgement authority.
