# x86 exception-vector fact-table source canary

This compile-only harness checks Cathedral's pure pre-timer x86 exception
vector table. It forces every named slot from 0 through 31 into the typed
artifact, including reserved slots and the profile-qualified AMD names at
28–30, and pins the declared table cardinality at 32.

The same harness checks the total exception delivery-shape fact needed by a
future generated entry stub. Double fault, invalid TSS, segment-not-present,
stack fault, general protection, page fault, alignment check, and control
protection are the exact error-code-bearing set. Reserved slots remain explicit,
ordinary exceptions are classified as arriving without an error code, and the
optional AMD meanings at 28–30 require a selected CPU profile instead of being
silently assigned to every x86-64 target.

Run:

```sh
tools/x86-exception-vectors-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order. Typed-artifact validation uses
`jq` and fails with an explicit dependency error when it is unavailable.

The snapshot and delivery categories record hardware facts only. They do not
admit optional vendor exceptions, create an entry stub, generate or publish
gates, provision stacks, install an IDT, or grant entry, root, or
machine-control authority.
