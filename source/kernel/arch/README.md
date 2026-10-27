# Machine mechanisms

Boot selects a concrete implementation from the two hardware owners:

- [isa/x86_64](isa/x86_64/README.md): translation encoding, CPU profiles,
  interrupt arrival/exception policy and x2APIC MSR operations.
- [board/pc](board/pc/README.md): the PC PIC/PIT preparation and acknowledgement
  path. Choosing the legacy timer vector is board policy.

The package imports pure hardware facts; it imports neither shared kernel state
nor firmware nor distribution behavior. It does not provide a pretend multi-ISA
facade: only this x86-64 prototype exists. Boot names the mechanism it uses. The
[Rust arch interface](../../../source-rs/kernel/arch/README.md) is the reference
for a future shared interface when another implementation actually needs it.

[FRONTIER.md](FRONTIER.md) preserves older proof and implementation notes.
Current verification and compiler compatibility gaps live in
[TASKS_RUST.md](../../../TASKS_RUST.md). Numeric validators do not grant memory
access or installation authority; instruction leaves retain their declared
machine/port authority.
