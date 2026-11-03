# PIC timer handler checks

A timer interrupt must tell its controller when it has finished. This canary
checks Cathedral’s real PIC handler: it passes its one-time acknowledgement to
the PIC completion operation, which writes the end-of-interrupt command to the
master controller. Callers that forget the acknowledgement or consume it twice
must fail checking for that specific reason.

Run with an optimized Omega compiler:

```sh
OMEGA_BIN=/path/to/omega tools/legacy-timer-root-canary/run.sh
```

The script lowers an ordinary caller of the handler using `inspect-terminal`
for `uefi_x86_64`. It follows calls from that entry and checks the three-machine
chain, linear argument transfers, one port write, port/value constants `0x20`,
and the absence of extra services or control flow. It then checks the two
invalid callers and their acknowledgement diagnostics. Logs stay in `build/`.
The script uses an installed `omega` when `OMEGA_BIN` is unset.

The Build file explicitly selects the acknowledgement’s storage representation.
That fixes its size and alignment; it does not construct an acknowledgement or
grant installation authority. The source calling policy requests masked entry
on Cathedral’s IRQ stack class and a generated interrupt return.

This is checking and Terminal lowering evidence. It does not execute port I/O,
emit or install an interrupt adapter, provision a stack, publish an IDT, or
prove repeated hardware returns. Those remain the kernel boot acceptance. The
old compiler JSON and fixed-fuel assertions are removed; this test makes no
replacement claim about those reports.
