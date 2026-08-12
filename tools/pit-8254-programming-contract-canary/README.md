# 8254 PIT programming contract source canary

This compile-only harness checks Cathedral's existing checked 8254 provider
leaf for programming channel 0 as a binary rate generator. It pins the source
boundary of two already-validated divisor bytes, the control-word write, and
the required low-byte-before-high-byte reload order.

Run:

```sh
tools/pit-8254-programming-contract-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order. Artifact validation uses `jq`
and fails with an explicit dependency error when it is unavailable.

The canary compiles and inspects provider code but never calls it. It chooses
no divisor or timer frequency, performs no port I/O, unmasks no IRQ, publishes
no IDT entry, enables no CPU interrupt, and grants no `PortIo` authority.
