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

**Status (2026-07-02).** In-progress: `uefi/` — milestone 1 of the first-boot
ladder (an Omega UEFI application that prints and returns, under QEMU/OVMF). Does
not compile yet; it targets Omega features still being implemented (see
`uefi/main.omg` and `../../../Omega/wiki/cathedral_alignment.md` item 7).
