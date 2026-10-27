# PC board mechanisms

Start at [timer/timer.omg](timer/timer.omg): `prepare_masked` remaps and masks
both PICs, then programs the PIT's fixed bootstrap rate. Its role/gate checks
select the PC vector while using the CPU owner's stack policy.

[Pic](timer/pic.omg) and [PIT](timer/pit.omg) are the bounded port-operation
leaves. [root.omg](timer/root.omg) settles an interrupt through the PIC after
using the x86 arrival contract. Preparation does not install an IDT, publish a
stack, unmask the IRQ or enable interrupts. These remain staged prototypes;
see the [recorded frontier](../../FRONTIER.md).
