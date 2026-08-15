# x86 legacy interrupt-bootstrap composition canary

This compile-only harness checks that Cathedral's already-implemented x86
exception and legacy-timer pieces agree as one pre-installation bootstrap
bundle. It joins the pure exception/IST policy and IDT gate layout with the
checked PIC-remap, PIT-programming, and timer-unmask provider leaves. The
Cathedral-owned masked preparation transaction composes only the remap and PIT
programming phases; timer unmasking remains deliberately separate.

The canary pins these cross-package facts:

- dedicated exception assignments remain within vectors 0–31;
- the complete 0–31 exception policy selects those dedicated assignments and a
  fatal current-stack/IST-zero fallback for every other slot;
- the timer assignment and the PIC master remap both select vector `0x20`;
- that timer uses the shared maskable-IRQ stack/IST class 4;
- the IDT gate remains a pure fixed 16-byte plan;
- PIC remapping ends with both controllers masked;
- PIT programming is a distinct command → low byte → high byte phase;
- the legacy provider owns a QEMU/bootstrap-only 100 Hz policy, selecting the
  rounded divisor 11,932 (`0x2e9c`) from the PC-compatible 1,193,182 Hz input;
- masked provider preparation calls PIC remapping first and PIT programming
  second with exact low/high bytes `0x9c`, `0x2e` and no caller-selected rate;
- its checked frame is complete and limited to `self.pic` and `self.pit`; each
  hardware leaf remains confined to its exact receiver/input frame;
- masked provider preparation retains only `PortIo`, performs no direct
  assembly, and cannot unmask the timer;
- timer unmasking is a later distinct operation admitting only master IRQ0;
- the hardware leaves retain only `PortIo`, while policy/layout machines reach
  nothing.

Run:

```sh
tools/x86-legacy-bootstrap-composition-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order. Artifact validation uses `jq`.

This is deliberately pre-installation acceptance. It does not materialize or
publish an IDT, provision stacks, invoke any provider, unmask real hardware,
enable CPU interrupts, or claim the still-blocked WCSU and checked
IDT-installation milestones. The selected periodic rate belongs only to the
first QEMU/PIC bring-up path; production LAPIC timing remains tickless and
one-shot.
