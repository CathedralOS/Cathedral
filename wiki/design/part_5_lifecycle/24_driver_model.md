# Chapter 24: Driver Model

> Drivers are ordinary components — isolated, capability-limited, and restartable —
> with hard effect ceilings on the hardware they touch, never privileged kernel blobs.

## The Legacy Contract

A traditional driver is a binary blob loaded into the kernel's address space with
*total* authority: it can touch any memory, program any DMA engine, take any
interrupt, and a single bug crashes — or silently corrupts — the whole system.
There is no capability boundary between "this NIC driver" and "all of physical
memory." Driver crashes are fatal; updates mean reboots; certification is a
signature on an opaque blob. And because every PC has thousands of devices, the
driver surface area is the single largest reason OS projects die before shipping.

## What Cathedral Wants

Drivers are components like any other ([[09_component_model]]): isolated, holding
only the specific capabilities for *their* device, restartable after a crash without
taking down the system, and upgradable in place via the hot-swap machinery
([[23_updates_and_hot_swap]]). A driver's hardware reach is an **effect ceiling** —
`device_io`, `memory_map` — plus narrow capabilities over *its* registers, DMA
regions, and interrupt lines, and nothing more. No arbitrary kernel driver blobs.

**The early wedge matters more here than anywhere.** Driver surface area is what
kills OS projects, so the first target is a *controlled hardware class* — a TV box,
appliance, kiosk, router, thin client, dev board, or a single locked-down laptop SKU
— where the device set is small, known, and stable. "All PCs" is the bad first
wedge; that path is pain (see [[00_vision_and_non_goals]]).

## Concerns & Design Space

- **Capability-limited hardware access.** A driver holds capabilities for its MMIO
  range, its DMA windows, and its IRQs — not ambient hardware power
  ([[03_capability_model]]).
- **DMA isolation.** The dangerous core: a device that can DMA anywhere defeats every
  software boundary. Requires IOMMU-backed, capability-scoped DMA windows.
- **Interrupt handling & power management.** Interrupts as scheduled work that must
  respect quiescence during swap; power transitions as typed states ([[14_power_management]]).
- **Crash recovery & restartability.** A driver crash is contained and recovered, not
  fatal ([[13_error_model_and_recovery]]); device state is re-established on restart.
- **Hot plugging & device discovery/matching.** Discovery enumerates devices;
  matching binds a device to a driver under explicit, auditable policy.
- **Versioned driver APIs.** The kernel↔driver and driver↔client interfaces are
  `wire data` + versioned `data`, so driver upgrades follow the same compatibility
  proofs as everything else.
- **User-mode drivers & verified interfaces.** Prefer user-mode where the device
  class allows; the driver↔hardware contract is a verified boundary.
- **Firmware updates, device permissions & certification.** Firmware flashing is a
  capability-gated operation; device access is granted, not assumed; driver
  certification flows through the store ([[36_store_and_economic_control]]).

## Key Questions

- What is the minimum trusted hardware-access broker, and how much driver logic can
  live outside the privileged core ([[26_kernel_architecture]])?
- How is DMA made safe without an IOMMU, if a wedge device lacks one — or is "has an
  IOMMU" a hardware prerequisite for the first wedge?
- How does a driver re-establish device state after a crash without a window where the
  hardware is in an unknown configuration?
- Which interrupt and power paths can meet quiescence for hot swap, and which force
  coexistence or a brief device-quiesce?

## Omega Leverage

- **Drivers as components with `effects` ceilings** — `device_io`, `memory_map` —
  make a driver's hardware reach a checkable, audited fact, per the standard effect
  vocabulary in [../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md).
- **Capabilities as values + domains** scope MMIO/DMA/IRQ access to one device.
- **Versioned driver APIs** via `wire data` + versioned `data` give compatible
  driver upgrades and a stable kernel↔driver contract.
- **Boundary traits** model the device edge: the hardware contract is a `boundary`
  whose guarantees are accepted but whose effects and authority are bounded.
- **Blocking-boundary modeling** lets a driver call that waits on hardware declare
  what can unblock it, rather than being an opaque wait.

## Open Questions

- How far can DMA isolation rely on hardware (IOMMU) vs. needing software shadowing,
  and what does that cost the first wedge?
- Can a meaningfully large fraction of drivers be authored in proved Omega, or is a
  bounded unsafe boundary unavoidable for the lowest register pokes?
- Where is the line between a verified user-mode driver and the privileged broker it
  must call — and who is in the TCB ([[26_kernel_architecture]])?

## Related
- [[09_component_model]] — drivers are components; this is the hard case.
- [[13_error_model_and_recovery]] — driver crash containment and restart.
- [[26_kernel_architecture]] — what hardware authority the privileged core retains.
- [[03_capability_model]] — capability-limited register/DMA/IRQ access.
- [[36_store_and_economic_control]] — driver certification and distribution.
