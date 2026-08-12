# owned-machine handoff contract source canary

This compile-only harness checks Cathedral's existing milestone-2/3 handoff
machine independently of the unresolved UEFI target-entry bridge. It pins the
successful `ExitBootServices` route into the qualified extent grant and the
subsequent post-firmware serial/owned-idle graph, while ensuring failure routes
park without acquiring an extent.

Run:

```sh
tools/owned-machine-handoff-contract-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order. Artifact validation uses `jq`
and fails with an explicit dependency error when it is unavailable.

The canary compiles and inspects `Main::own_machine` but never calls it. It does
not select or generate the UEFI program-entry stub, compose physical firmware
inputs with semantic roots, call firmware, grant memory, perform port I/O, or
execute `hlt`.
