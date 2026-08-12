# x86 page-table layout source canary

This compile-only harness checks Cathedral's pure x86-64 paging-entry schema
and layout policy. It pins one fixed 8-byte, 8-aligned container whose fourteen
logical fields tile all 64 bits exactly, including the 40-bit page-frame number
at physical-address bits 12 through 51. The compile root also keeps the sibling
x86 IDT-gate fact live against the same current core-layout dependency.

Run:

```sh
tools/x86-page-table-layout-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order. Typed-artifact validation uses
`jq` and fails with an explicit dependency error when it is unavailable.

The canary validates geometry only. It mints no `Extent`, proves no frame
ownership, installs no table, invalidates no TLB, and grants no mapping or
machine-control authority.
