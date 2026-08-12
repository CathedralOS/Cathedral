# x86 exception-vector fact-table source canary

This compile-only harness checks Cathedral's pure pre-timer x86 exception
vector table. It forces every named slot from 0 through 31 into the typed
artifact, including reserved slots and the profile-qualified AMD names at
28–30, and pins the declared table cardinality at 32.

Run:

```sh
tools/x86-exception-vectors-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order. Typed-artifact validation uses
`jq` and fails with an explicit dependency error when it is unavailable.

The snapshot records identities only. It does not admit optional vendor
exceptions, generate or publish gates, provision stacks, install an IDT, or
grant entry, root, or machine-control authority.
