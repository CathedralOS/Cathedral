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
qualified root at a time. Allocation strategies remain ordinary packages over
qualified backing extents; no compiler-owned `Arena` competes with the concrete
range capability.

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

The first Cathedral-owned page-table policy now explicitly selects the
four-level, 48-bit canonical-address QEMU/UEFI-x64 bootstrap profile with LA57
disabled and the four 9-bit index shifts fixed at 39, 30, 21, and 12. This is a
Cathedral target policy, not inferred universal x86-64 behavior. The source
now rejects the noncanonical 48-bit address hole and derives the four bounded
page-walk indexes plus byte offset while retaining the original numeric address.
Those coordinates grant no mapping fact. A second ordinary helper checks 4-KiB
alignment and the 52-bit physical-address envelope, then retains the candidate
address with its bounded 40-bit PFN. That numeric geometry grants no physical
source, frame, backing, or mapping authority. A detached PTE composer derives
only that PFN and copies every other schema field from explicit caller data;
it does not interpret the role-dependent bit, select a leaf/link policy, or
grant placement or installation authority. The source state represents one
4-KiB candidate as 512 existing 8-byte x86 PTE values. A checked 512-step scan
accepts only exact all-zero/non-present encoding and returns the whole ordinary
candidate; it does not establish storage, hierarchy page count, mapping, or
installation. Physical placement still requires qualified pre-reserved backing;
hierarchy storage and link construction remain.

The interrupt-provider bootstrap also has checked 8259 PIC and 8254 PIT
port-operation helpers plus checked x2APIC one-shot, stop, and acknowledgement
MSR helpers. They retain `PortIo` or `MachineControl` reach from parsed
instruction contracts and deliberately stop short of installing an IDT,
unmasking a live root, or enabling CPU interrupts. A Cathedral-owned masked
preparation transaction now selects the QEMU/bootstrap-only 100 Hz periodic PIT
policy (divisor 11,932 / `0x2e9c`), remaps the PIC with every input masked, and
then programs PIT channel 0 with its fixed low/high bytes. The separate
timer-unmask operation remains ordered after exception-IDT publication. One
ordinary route/rate policy now carries the remapped timer vector and coupled
maskable-IRQ class/IST assignment beside that fixed PIT choice; it invokes no
provider and grants no gate, stack, controller-state, or acknowledgement claim.
Production LAPIC timing remains calibrated, tickless, and one-shot. These are
provider operations, not ambient driver access.
`x86_interrupt_profile.omg` composes the pure vector and stack facts into the
four initial vector-to-stack assignments and one total fatal-diagnostic
bootstrap exception-floor policy over slots 0–31. The latter couples each
stack choice to the exact hardware IST index and the source-authored exception
delivery category without minting any of those missing authorities. Delivery
for slots 28–30 remains `CpuProfileRequired`; this policy does not select a CPU
profile or an entry-frame normalization plan. A complete role-labeled
stack-class set now groups double fault, NMI, machine check, and maskable IRQ
policy into one candidate;
its pure validator derives the expected set internally and rejects any
class/index drift before later WCSU-derived storage provisioning. It assigns no
bytes and mints no `StackLease`. The pure gate-policy validator internally derives
that policy for the requested table slot, rejects vector, IST, delivery,
disposition, gate-attribute, or reserved-field drift, and returns only an
ordinary `PolicyConsistent` candidate. A table-level wrapper now scans one fixed
32-entry candidate with checked decreasing fuel and accepts only after every
slot passes; no partial table escapes. It deliberately does not vouch for the
handler entry identities or selectors; those still need the admitted resolver
and boot-selected code-segment source facts. The bootstrap fatal-diagnostic
leaf now records one normalized 0–31 vector in preallocated atomic state,
release-publishes its validity, and unconditionally aborts without calls or a
normal-return path. It remains below the external-entry seam: generated stubs,
exception calling/preemption plans, and admitted internal-state binding are not
invented by that leaf. `legacy_timer_root.omg` now supplies
the source-level normalized entry/provider canary under Cathedral's
maskable-IRQ stack and masked `InterruptReturn` plan. Its checked PIC path emits
the master EOI, then consumes the exact pending acknowledgement through
normalized completion.
The fixed-work timer handoff now records one caller-supplied monotonic
observation in preallocated atomic state and publishes one coalescing wake bit
with release ordering. Its ordinary-task leaf claims and clears that marker
with receive ordering before loading the latest published observation, yielding
either one claimed wake or an idle result. Both halves are independently
checked as terminating and call-free. Wiring the producer into the timer root
still waits for the installed-root path to supply that state as an internal
claim; adding a hardware entry parameter would falsify the normalized
`InterruptEntry::enter` contract.
Omega independently verifies that terminal closure and derives its exact
five-unit fixed-fuel certificate. Omega's canonical terminal installation
record now seals emitter-derived per-function and per-call stack facts and
reproduces the internal artifact bound after decode. A nonzero decoded demand
can now bind exact installed bytes and entry, compose with the selected
provider, and survive in the general installed-root report. Emitting that record
from this source-root path and accounting for the interrupt entry adapter are
the next blocker; this root cannot claim a complete WCSU until both exist.
The package still publishes no IDT, provisions no stack, unmasks no IRQ, and
performs no installation.
