# CHARTER — `source/core/`

**Scope.** The proved kernel — the trusted computing base, and nothing more: the
capability arena, scheduler, memory/address-space manager, IPC, timers, trusted
time, spawn/loader, hot-swap engine, transaction coordinator, attestation, and
the proof checker. Small enough to audit in full. Proofs live **here**, beside
the code they cover — never in a sibling that can lag.

**Depends on.** `foundation/` and `contracts/` only. **Nothing depends on
`core/` at build time** — userspace targets the ABI in `contracts/`, which
`core/` implements, so `core/` is a leaf in the build graph. `core/` stays
firmware-neutral: the UEFI-specific memory-map walk is `boot/`'s job; `core/`
provides the checked adapter for Omega's owner-authored `ExtentRootProvider`;
the admitted boot build supplies firmware-neutral geometry and receives one
qualified root at a time. `Arena`
remains reserved for bounded allocation authority over backing storage; it is
not the concrete range capability.

**Non-goals.** No device drivers, no userspace service logic, no firmware ABIs.
If it isn't small enough to audit in full, it doesn't belong here.

**Status (2026-07-28).** In-progress: `extent.omg` — milestone 2 now imports
Omega's linear `{ base: addr, length: u64 }` carrier and supplies the selected
checked `ExtentRootProvider::grant` adapter. The UEFI boot build admits that
exact provider plan, obtains one `Extent in Granted` after successful
`ExitBootServices`, and carries it through the serial-report graph into the
owned-idle loop. Physical-space, rights, backing-containment, and checked
split/merge facts remain later resource-frontier work.
The generational `{slot, generation}` authority graph
that makes capabilities unforgeable + revocable is the
`capability_lifecycle` arc, later.

The interrupt-provider bootstrap also has checked 8259 PIC and 8254 PIT
port-operation helpers plus checked x2APIC one-shot, stop, and acknowledgement
MSR helpers. They retain `PortIo` or `MachineControl` reach from parsed
instruction contracts and deliberately stop short of installing an IDT,
unmasking a live root, enabling CPU interrupts, or asserting a platform timer
frequency. They are provider code, not ambient driver access; invoking them
remains ordered after exception-IDT publication.
`x86_interrupt_profile.omg` composes the pure vector and stack facts into the
four initial vector-to-stack assignments without minting any of those missing
authorities.
