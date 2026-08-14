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
- masked provider preparation calls PIC remapping first and PIT programming
  second, forwarding the caller's divisor bytes exactly;
- its checked frame is complete and limited to those divisor inputs plus
  `self.pic` and `self.pit`; each hardware leaf remains confined to its exact
  receiver/input frame;
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
enable CPU interrupts, choose a timer frequency, or claim the still-blocked
WCSU and checked IDT-installation milestones.
