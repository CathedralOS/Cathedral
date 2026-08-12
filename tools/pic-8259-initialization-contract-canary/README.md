# 8259 PIC initialization contract source canary

This compile-only harness checks Cathedral's existing checked 8259 provider
surface for remapping both controllers while masked and subsequently unmasking
only the master timer input. It pins the ordered ICW/OCW port-write sequences
and retained `PortIo` reach of those two operations.

Run:

```sh
tools/pic-8259-initialization-contract-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order. Artifact validation uses `jq`
and fails with an explicit dependency error when it is unavailable.

The canary compiles and inspects provider code but never calls it. It does not
exercise the separately covered acknowledgement/root path, perform port I/O,
publish an IDT entry, enable CPU interrupts, or grant `PortIo` authority.
