# Chapter 02: Driver Model

> Drivers are ordinary components — isolated, capability-limited, and restartable — with hard effect ceilings on the hardware they touch, never privileged kernel blobs.

## The Legacy Model

A traditional driver is a binary blob loaded into the kernel's address space with *total* authority: it can touch any memory, program any DMA engine, take any interrupt, and a single bug crashes — or silently corrupts — the whole system. There is no capability boundary between "this NIC driver" and "all of physical memory." Driver crashes are fatal; updates mean reboots; certification is a signature on an opaque blob. And because every PC has thousands of devices, the driver surface area is the single largest reason OS projects die before shipping.

## The Cathedral Model

Drivers are components like any other ([[component_model]]): isolated, holding only the specific capabilities for *their* device, restartable after a crash without taking down the system, and upgradable in place via the hot-swap machinery ([[updates_and_hot_swap]]). A driver's hardware reach is an **effect ceiling** — `device_io`, `memory_map` — plus narrow capabilities over *its* registers, DMA regions, and interrupt lines, and nothing more. No arbitrary kernel driver blobs.

**Driver scope is the deciding constraint here.** Driver surface area is what kills OS projects, so the design targets a *constrained, known, stable device set* rather than arbitrary hardware. A small fixed device set is far more tractable than supporting "all PCs," which is explicitly out of scope for the model (see [[vision_and_non_goals]]): the combinatorics of arbitrary hardware defeat the proof and isolation guarantees the rest of the system depends on.

## Concerns & Design Space

- **Capability-limited hardware access.** A driver holds capabilities for its MMIO range, its DMA windows, and its IRQs — not ambient hardware power ([[capability_model]]).
- **DMA isolation.** The dangerous core: a device that can DMA anywhere defeats every software boundary. Requires IOMMU-backed, capability-scoped DMA windows.
- **Interrupts are inbound messages.** A device signals its driver by unparking a driver task on a channel condition (a completion-queue post or doorbell), the scheduler's one wait primitive ([[scheduler_and_resources]]), so there is no special interrupt-handler context. DMA regions are capability-leased shared memory between device and driver (the shared-region primitive across the hardware boundary, [[ipc_and_service_invocation]]); registers are a narrow MMIO capability. Interrupt delivery must respect quiescence during a swap, and power transitions are typed states ([[power_management]]).
- **Crash recovery & restartability.** A driver crash is contained and recovered, not fatal ([[error_model_and_recovery]]); device state is re-established on restart.
- **Hot plugging & device discovery/matching.** Discovery enumerates devices; matching binds a device to a driver under explicit, auditable policy.
- **Versioned driver APIs.** The kernel↔driver and driver↔client interfaces are `wire data` + versioned `data`, so driver upgrades follow the same compatibility proofs as everything else.
- **User-mode drivers & verified interfaces.** Prefer user-mode where the device class allows; the driver↔hardware contract is a verified boundary.
- **Synthetic devices.** A driver presents a device interface, so a component can serve a *synthetic* device to a child instead of real hardware: a virtual NIC, framebuffer, sound, or block device. This is the recursive-provider pattern ([[capability_model]]) at the hardware edge, and it is what lets a virtual machine present hardware to a guest and a simulator hand mock devices to code under test ([[testing_and_simulation]]).
- **Firmware updates, device permissions & certification.** Firmware flashing is a capability-gated operation; device access is granted, not assumed; driver certification flows through the store ([[store_and_economic_control]]).

## Key Questions

- What is the minimum trusted hardware-access broker, and how much driver logic can live outside the privileged core ([[kernel_architecture]])?
- How is DMA made safe without an IOMMU, if a target device lacks one — or is "has an IOMMU" a hardware prerequisite for the target hardware?
- How does a driver re-establish device state after a crash without a window where the hardware is in an unknown configuration?
- Which interrupt and power paths can meet quiescence for hot swap, and which force coexistence or a brief device-quiesce?

## Omega Leverage

- **Drivers as components with `effects` ceilings** — `device_io`, `memory_map` — make a driver's hardware reach a checkable, audited fact, per the standard effect vocabulary in [../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md).
- **Capabilities as values + domains** scope MMIO/DMA/IRQ access to one device.
- **Versioned driver APIs** via `wire data` + versioned `data` give compatible driver upgrades and a stable kernel↔driver contract.
- **Boundary traits** model the device edge: the hardware contract is a `boundary` whose guarantees are accepted but whose effects and authority are bounded.
- **Blocking-boundary modeling** lets a driver call that waits on hardware declare what can unblock it, rather than being an opaque wait.

## Open Questions

- How far can DMA isolation rely on hardware (IOMMU) vs. needing software shadowing, and what does that cost on a constrained device set?
- Can a meaningfully large fraction of drivers be authored in proved Omega, or is a bounded unsafe boundary unavoidable for the lowest register pokes?
- Where is the line between a verified user-mode driver and the privileged broker it must call — and who is in the TCB ([[kernel_architecture]])?

## Related
- [[component_model]] — drivers are components; this is the hard case.
- [[error_model_and_recovery]] — driver crash containment and restart.
- [[kernel_architecture]] — what hardware authority the privileged core retains.
- [[capability_model]] — capability-limited register/DMA/IRQ access.
- [[store_and_economic_control]] — driver certification and distribution.
