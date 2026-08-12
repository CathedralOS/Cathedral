# owned-machine handoff contract source canary

This compile-only harness checks Cathedral's existing milestone-2/3 handoff
machine independently of the unresolved UEFI target-entry bridge. It pins each
fresh memory map to its returned key, requires a stale-key
`EFI_INVALID_PARAMETER` result from `ExitBootServices` to discard the old
candidate and restart that whole transaction, and admits the qualified extent
grant only after success. Other failures park without acquiring an extent; the
post-firmware serial/owned-idle graph retains the one successful grant.

Before the irreversible exit, the selected conventional span must be nonempty,
page-aligned, small enough for exact page-to-byte conversion, and have a
representable one-past end. The validated byte length stays in the same
map/key transaction through successful exit and becomes the exact grant
geometry. Rejection parks inside firmware without calling the provider.

Map storage is a fixed 64-KiB zero-initialized field, not an allocation or
memory capability. An explicit leading `u64` makes the byte view 8-byte
aligned; successful map results additionally require an 8-byte-multiple
descriptor stride, a whole number of descriptors, and the exact supported UEFI
descriptor revision (version 1) before any typed recast. An unknown revision is
a malformed success and parks unowned.
Cathedral advertises 16 KiB first; only exact
`EFI_BUFFER_TOO_SMALL` with a reported requirement in `(16 KiB, 64 KiB]`
permits one retry advertising the full backing. A second capacity failure,
oversize requirement, malformed success, or unrelated error parks unowned.

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
