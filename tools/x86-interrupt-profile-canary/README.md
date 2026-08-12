# x86 interrupt-profile source canary

This compile-only harness checks Cathedral's pure initial vector-to-stack
policy. The complete bootstrap exception floor is one total policy over slots
0–31: every slot is fatal-diagnostic, NMI/double-fault/machine-check derive
their normalized dedicated stack and hardware IST from the coupled class/index
records 2, 1, and 3 respectively, and every other slot uses the interrupted
kernel stack with IST zero. The harness also pins the first remapped legacy
timer to the shared maskable-IRQ class/index 4.

The same harness pins Cathedral's pure gate-policy validator. A
`PolicyConsistent` candidate matches the total policy derived internally for
the requested table slot: exact vector, IST, and fatal disposition plus the
fixed present/ring-0/interrupt-gate attributes and zero reserved field. The
result intentionally says nothing about the candidate's handler entry identity
or selector: those require the admitted source resolver and boot-selected
code-segment fact before materialization.

Run:

```sh
tools/x86-interrupt-profile-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order. Typed-artifact validation uses
`jq` and fails with an explicit dependency error when it is unavailable.

This canary grants no stack storage, root admission, IDT/TSS access, interrupt
authority, or machine control. It validates authored policy data and a partial
prepublication check that later WCSU/root admission and gate materialization
must consume with the still-missing identity and selector evidence.
