# CHARTER — `source/boot/`

**Scope.** The firmware seam, below `core/`. Per-firmware loaders that receive
control from firmware and construct the machine state `core/` expects. `uefi/` is
the Omega UEFI application (the milestone-1 entry). Boot code lives in a
different reality — pre-capability, different memory rules, possibly a restricted
subset of Omega — so it stays out of `core/` and never pollutes the proved
kernel's invariants.

**Depends on.** `contracts/` (the boot hand-off ABI) and `foundation/` only.
**Never `core/`'s internals** — boot constructs the state the hand-off contract
promises; it does not reach into the kernel. The hand-off contract is owned by
the core side and *implemented* by boot, never dictated by it.

**Non-goals.** No device drivers (those are `drivers/`), no capability-holding
service logic. Boot's job ends the moment control transfers to `core/` with the
capability world constructed — the single most important documented boundary in
the repository.

**Status (2026-08-12).** In-progress: `uefi/` boots the Omega image under
QEMU/OVMF, obtains and validates the runtime-strided memory map, refreshes the
whole map/key transaction once when `ExitBootServices` rejects a stale key,
admits one receipt-backed root extent after success, reports it through the
16550, and parks while retaining the root. Its fixed 64-KiB bootstrap storage
advertises 16 KiB first and grows once on exact `EFI_BUFFER_TOO_SMALL`, without
allocation or authority creation. The byte view has an explicit 8-byte
alignment anchor, and map acceptance requires an aligned stride and exact
descriptor count before typed traversal, as well as the exact supported UEFI
descriptor revision. Before any Boot Services dispatch, the enclosing system
table and Boot Services table must carry their standard signatures, agree on a
well-formed UEFI revision at or above the x86-64 profile's 2.0 floor, advertise
headers covering every field Cathedral reads, and keep the common reserved
header field zero; malformed table metadata parks without a firmware call.
Whole-table CRC validation remains outstanding.
The serial report is exact through 99,999 MiB and emits the honest bounded form
`99999+ MiB` for larger roots, so its five-digit formatter cannot wrap into
non-digit bytes. Page-to-MiB conversion saturates at 100,000 rounds because no
larger exact value affects that report. Each FIFO-readiness wait is capped at
1,000,000 status reads; exhaustion abandons the optional report and parks while
retaining the root.
Repeated stale-key rejection fails closed rather than looping indefinitely.
Before exiting firmware it excludes runtime-marked, hot-pluggable, and
specific-purpose descriptors and rejects an empty, unaligned,
page-count-overflowing, or non-representable conventional span. A second
full-map pass also requires every descriptor to have aligned, representable
physical and virtual geometry and a valid standard/OEM/OS-loader memory type,
permits only revision-1 standard/ISA attribute bits, requires
ISA-specific bits to carry their validity flag, and requires every other
physical range to be disjoint from the selected span. The exact
validated length then flows into the admitted grant. The
generated target-entry bridge,
physical-space/right/backing facts, and actual handoff into the proved core
remain outstanding.
