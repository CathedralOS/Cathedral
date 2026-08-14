# x86 interrupt-profile source canary

This compile-only harness checks Cathedral's pure initial vector-to-stack
policy. The complete bootstrap exception floor is one total policy over slots
0–31: every slot is fatal-diagnostic, NMI/double-fault/machine-check derive
their normalized dedicated stack and hardware IST from the coupled class/index
records 2, 1, and 3 respectively, and every other slot uses the interrupted
kernel stack with IST zero. The harness also pins the first remapped legacy
timer to the shared maskable-IRQ class/index 4.

The same harness pins Cathedral's pure gate-policy validators. The single-slot
helper checks exact vector, IST, fatal disposition, fixed
present/ring-0/interrupt-gate attributes, and zero reserved field. The
table-level validator accepts only a fixed 32-entry candidate after a checked,
terminating scan accumulates that decision for every slot; a failure cannot
return a partially checked table. Neither result says anything about handler
entry identity or selector: those require the admitted source resolver and
boot-selected code-segment fact before materialization.

All candidate fields and successful-result payloads remain runtime-relevant:
the policy check must return the complete candidate that it inspected, not an
erased proof-shaped shell. The artifact check also pins the wrapper's exact
by-value candidate/result transfer frame and the recursive scanner's internal
no-write frames separately.

Run:

```sh
tools/x86-interrupt-profile-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order. Typed-artifact validation uses
`jq` and fails with an explicit dependency error when it is unavailable.

This canary grants no stack storage, root admission, IDT/TSS access, interrupt
authority, or machine control. It validates authored policy data and a
complete-floor but still partial-field prepublication check that later
WCSU/root admission and gate materialization must consume with the
still-missing identity and selector evidence.
