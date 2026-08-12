# owned-machine handoff contract source canary

This compile-only harness checks Cathedral's existing milestone-2/3 handoff
machine independently of the unresolved UEFI target-entry bridge. It pins each
fresh memory map to its returned key, requires a stale-key
`EFI_INVALID_PARAMETER` result from `ExitBootServices` to discard the old
candidate and restart that whole transaction, and admits the qualified extent
grant only after success. Other failures park without acquiring an extent; the
post-firmware serial/owned-idle graph retains the one successful grant.

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
execute `hlt`. The current fixed 16-KiB map buffer still has no larger-capacity
fallback; oversize maps fail closed.
