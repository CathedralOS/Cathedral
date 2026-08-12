# UEFI calling-policy source canary

This compile-only harness pins the named
`UefiX86_64CallingPolicy: UefiX86_64 satisfies CallingPolicy` evidence and the
UEFI entry schema that consumes it. It deliberately excludes the boot
package's provider grants and lockfile, so a separate trust re-approval cannot
hide a calling-policy regression. The typed policy check also pins the actual
Microsoft x64 entry plan: RCX/RDX indirect parameter copies, their disjoint
stack homes, the volatile-register set, 16-byte alignment, 32-byte shadow
space, call/return control, provider-selected stack, and machine-state policy.

Run:

```sh
tools/uefi-calling-policy-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order.

The typed-artifact assertion uses `jq`; the harness fails with an explicit
dependency error when it is unavailable.
