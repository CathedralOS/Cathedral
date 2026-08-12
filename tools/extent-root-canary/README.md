# Qualified extent-root source canary

This compile-only harness checks Cathedral's first memory-authority provider
without booting QEMU or requiring OVMF. An isolated admission frontier selects
`CathedralExtentRootProvider`, crosses `ExtentRootProvider::grant` once with
ordinary geometry, and retains the resulting linear `Extent::Granted` claim in
a closed state loop. The artifact checks prove that the fact originates at the
exact provider receipt, that the adapter returns the same linear claim it
received, and that the claim's content is the canonical half-open interval
from `base` through `base + length`.

Run:

```sh
tools/extent-root-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order. Artifact validation uses `jq`
and fails with an explicit dependency error when it is unavailable.

This isolates the provider contract from the still-outstanding generated UEFI
entry-stub bridge and from the much larger serial boot graph. It proves the
current checked adapter and admission shape, not that firmware supplied valid
geometry. It does not claim physical-space, rights, backing-containment,
mapping, or allocator facts; those remain later resource-frontier work.
