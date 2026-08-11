# UEFI calling-policy source canary

This compile-only harness pins the named
`UefiX86_64CallingPolicy: UefiX86_64 satisfies CallingPolicy` evidence and the
UEFI entry schema that consumes it. It deliberately excludes the boot
package's provider grants and lockfile, so a separate trust re-approval cannot
hide a calling-policy regression.

Run:

```sh
tools/uefi-calling-policy-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order.
