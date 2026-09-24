# Chapter 02: Driver Model

> Drivers are ordinary components: isolated, capability-limited, and restartable, with hard reach ceilings on the hardware they touch. They are never privileged kernel blobs.

## The Legacy Model

A traditional driver is a binary blob loaded into the kernel's address space with total authority. It can touch any memory, program any DMA engine, and take any interrupt, and a single bug crashes or silently corrupts the whole system. There is no capability boundary between "this NIC driver" and "all of physical memory." Driver crashes are fatal, updates mean reboots, and certification is a signature on an opaque blob. Because every PC has thousands of devices, the driver surface area is the single largest reason OS projects die before shipping.

## The Cathedral Model

Drivers are components like any other ([[component_model]]). They are isolated, hold only the capabilities for their own device, restart after a crash without taking down the system, and upgrade in place through the hot-swap machinery ([[updates_and_hot_swap]]). A driver's hardware reach is a **reach ceiling** of boundary-trait service identities plus narrow capabilities over its register extents, DMA loans and IOMMU domain, and interrupt line. There are no ambient `device_io`/`memory_map` keyword powers and no arbitrary kernel driver blobs.

### Coverage is an effort problem, not a safety one

Coverage is the deciding constraint, and confinement turns it from a safety problem into an effort problem. Driver surface area is what kills OS projects. But a bad or foreign driver is always contained, so the combinatorics of arbitrary hardware no longer threaten the system's isolation. What remains is who writes and maintains each driver.

The strategy follows: tolerate freely, reward Omega, lead with class drivers. Most hardware speaks a standard class protocol (USB HID and mass-storage, NVMe, AHCI, XHCI, HD Audio), so one proved-Omega driver per class covers the bulk with no per-device work. The proprietary long tail, a GPU blob say, runs confined and second-class, which still beats an in-kernel blob. Proved-Omega drivers earn the premium: proof, zero-copy, live-upgrade, more trust. Confined C or blob drivers work and stay safe but lose the premium, and their faults still cost their own device's availability, data, and performance. Contained is not harmless. LLM-assisted authoring and porting lower the native cost further. The real bottleneck for the tail is undocumented hardware specs, not code volume.

### The decided mechanism

The model is **confinement over trust**: rather than trying to make a driver trustworthy, make its trustworthiness not matter. A driver runs as an ordinary user-mode component, walled by hardware and confined by capabilities, so a driver that is buggy, hostile, miscompiled, or written in plain C can corrupt only the one device it already owns. Proved-Omega drivers are rewarded with more trust and less overhead but are never required for system safety. That is the only defensible stance while the toolchain is young, and it is what lets Cathedral accept third-party or vendor-written drivers without each one being a system-wide risk.

- **IOMMU is the hardware floor.** Device DMA bypasses the CPU and its MMU entirely, so the MMU cannot contain a device. Only the IOMMU can ([[kernel_architecture]]). Cathedral requires an IOMMU and treats a machine without one as a weaker-guarantee target rather than contorting the architecture around legacy hardware. This retires the "safe DMA without an IOMMU" question.
- **The capability manifest.** A driver instance starts with exactly four things: a plan-derived placed view over its mapped register `Extent`; DMA submission authority plus its IOMMU domain; its IRQ endpoint; and service channels. Nothing is ambient, and its blast radius is those grants.
- **The core owns the confinement machinery.** The privileged core programs the IOMMU and maps MMIO. The driver only requests. It must be this way, because programming the IOMMU is the confinement, so a driver that could shape its own DMA domain could unconfine itself. The driver sits outside the TCB. The core's hardware-access broker is inside it.
- **Interrupt = message.** The only kernel-mode code on the path is a tiny, device-agnostic, fixed-work IRQ root. It acknowledges or masks the line and turns the interrupt into a wakeup for the parked driver actor, using the scheduler's existing park/unpark ([[scheduler_and_resources]]). Level state such as the timer uses a preallocated coalescing wake. Information-bearing edges use a bounded ring with a declared overflow policy. No device-specific fan-out runs privileged. Extreme-throughput devices skip interrupts and poll: the driver busy-reads a DMA ring with no kernel hop, using the shared-region primitive ([[ipc_and_service_invocation]]) as the ring.
- **Restart is the easy cousin of hot-swap.** A crash discards state instead of preserving it, so there is no quiescence snapshot and no migration. The core tears down the dead instance's IOMMU domain and capabilities first, cutting the device off from RAM before that memory is reused. It then resets the device to a known state and spawns a fresh instance. Recovery is visible and typed, never a silent auto-replay. The client's channel reports the restart, and the layer holding the semantics (an FS journal, a protocol's retry) decides what to redo. A standard reconnect helper keeps visible from being painful ([[error_model_and_recovery]]).
- **Device reset is core-driven.** "Reset to a known state" is a hardware property, not a driver promise, so the core drives it over the dead driver's head. What Cathedral requires is that the device support a core-drivable reset, which is a hardware admission check. It never requires that the driver reset correctly. A reset available only at bus level cascades to siblings, so reset domains form a tree the core coordinates. Irreversible physical effects, such as a printed page, are not resettable at all. Reset restores controllability, not the world, and the in-flight unknown is surfaced by the visible, typed recovery above.
- **Containment before cleanliness.** Reset splits two properties. **Containment**, meaning the device cannot hurt anything, is guaranteed instantly by the IOMMU teardown above. **Cleanliness**, meaning the device is back to known-good, is best-effort up a ladder: soft-quiesce (asks the driver; optional), then FLR (function-level reset: the core writes PCIe config space, driver-independent, and the silicon wipes the driver's state), then bus/port reset, then power-cycle, then mark-dead.
- **DMA copy strategy: zero-copy primary, bounce fallback, one call.** A driver asks the broker to submit an authorized extent. The broker chooses direct IOMMU mapping or a bounce buffer and returns a linear transfer token. Device-read excludes CPU mutation, device-write excludes CPU access, and consuming completion returns the loan after required cache/fence work. The token may survive task suspension. Reset/revocation completes IOMMU invalidation and acknowledgement before reuse. The direct/bounce crossover remains a measured provider policy.

## Concerns & Design Space

- **Capability-limited hardware access.** A driver holds capabilities for its MMIO range, its DMA windows, and its IRQs, not ambient hardware power ([[capability_model]]).
- **DMA isolation.** This is the dangerous core. A device that can DMA anywhere defeats every software boundary, so DMA windows are IOMMU-backed and capability-scoped.
- **Interrupts become messages above a real entry root.** Hardware still enters a tiny target-specific boundary root with a pinned `CallPlan + StatePlan`, and installation records it in the external-root ledger. The root acknowledges/masks and signals the driver endpoint. Device-specific processing remains in the ordinary driver task. DMA extents are external loans; registers are plan-derived placed views. Interrupt delivery and outstanding loans both pin live replacement.
- **DMA reach is per transfer, not ambient.** Omega refuses an external loan unless an admitted borrower contract or hardware-isolation receipt is bound to the exact loan, borrower/direction, address-space provenance/era, and lent base/length. A whole-buffer receipt cannot authorize a smaller subrange transfer, so activation stacks and other storage outside that Extent remain unreachable to the agent.
- **Crash recovery & restartability.** A driver crash is contained and recovered, not fatal ([[error_model_and_recovery]]). Device state is re-established on restart.
- **Hot plugging & device discovery/matching.** Discovery enumerates devices. Matching binds a device to a driver under declared, auditable policy.
- **New-device trust.** A freshly hot-plugged device is not trusted by default. A new input device cannot drive the trusted path or inject events until authorized, which is the BadUSB defense. A new storage device appears as an untrusted realm the user browses and shares from by gesture, rather than something apps are auto-granted ([[human_permission_ux]], [[windowing_and_compositor]]).
- **Peripheral classes are ordinary devices.** Printers, scanners, cameras, and USB peripherals are ordinary device-plus-driver-plus-capability instances rather than special subsystems. Printing is a userspace print service rendering to a page description, plus a printer driver, plus a "may print here" capability, with the spooler a queue component. The OS adds no print subsystem beyond the driver and the service.
- **Foreign filesystems and removable media.** A drive formatted for another OS (FAT, exFAT, NTFS) is mounted as a realm by a translating filesystem driver that implements the realm interface over the foreign layout ([[filesystem_as_database]]). You read and write files, but the native features (content-addressing, integrity, type metadata, snapshots, sealing) are absent because the foreign format cannot hold them. A removable drive is therefore an interop-versus-features choice: keep it foreign so it travels to other systems, or reformat it native for the full feature set. The foreign parser runs as a confined driver, so a malicious removable filesystem degrades from a kernel compromise to a contained driver fault.
- **Evolving driver APIs.** Kernel-to-driver and driver-to-client interfaces use ordinary numbered schemas, layout/codec policies, immutable historical shapes, checked conversions, and channel-declared compatibility windows. Live driver replacement is a separate component protocol over running providers and retained state.
- **User-mode, contained not trusted.** Drivers run in user mode, confined by capabilities and the IOMMU, so a driver's correctness is never relied on for system safety (see the decided mechanism above). The driver-to-hardware contract is a `boundary`. Proved Omega is rewarded, not required.
- **Synthetic devices.** A driver presents a device interface, so a component can serve a synthetic device to a child instead of real hardware: a virtual NIC, framebuffer, sound, or block device. This is the recursive-provider pattern ([[capability_model]]) at the hardware edge. It is what lets a virtual machine present hardware to a guest and a simulator hand mock devices to code under test ([[testing_and_simulation]]).
- **Firmware updates, device permissions & certification.** Firmware flashing is a capability-gated operation. Device access is granted, not assumed. Driver certification flows through the store ([[store_and_economic_control]]).
- **Zero value.** Optional device selection may use an `Empty | Live(Device)` sum. Zero-fill never mints a register extent, DMA loan, IRQ, or provider grant. A synthetic null device is a selected provider, not forged authority hidden in zero bits ([[omega_substrate]]).

## Key Questions

The confinement model above answers the trust question. What remains is device-side and sizing:

- **Device reset residue.** The containment/cleanliness split and the core-driven ladder handle the common case. What remains is the wedged device that ignores even FLR, forcing bus or power escalation or mark-dead, and coordinating the shared-bus cascade where resetting one device restarts its siblings. This is shared with hot-swap's device-quiescence corner.
- **The minimal hardware broker.** The core owns IOMMU/MMIO programming and the IRQ stub. What is the smallest such broker surface, and how little device knowledge can it carry while staying generic ([[kernel_architecture]])?
- **Quiescence for hot-swap, not crash.** Upgrading a live driver, as opposed to restarting a dead one, still needs the device quiesced enough to snapshot. Which interrupt and power paths can reach that, and which force a brief device-quiesce ([[power_management]], [[updates_and_hot_swap]])?

## Omega Leverage

- **Drivers as components with boundary-trait reach ceilings** make hardware reach a checkable, audited fact without blessed lowercase keywords. See [Omega reach](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md).
- **Extents, placed views, linear external loans, and entry-root plans** scope MMIO/DMA/IRQ access to one device ([[hardware_foundation_profile]]).
- **Ordinary numbered schemas + layout/codec policies** give compatible driver APIs. Component identities and quiescence govern live replacement.
- **Boundary traits** model the device edge: the hardware contract is a `boundary` whose guarantees are accepted but whose effects and authority are bounded.
- **Blocking-boundary modeling** lets a driver call that waits on hardware declare what can unblock it, rather than being an opaque wait.

## Open Questions

- Can a device's half-state always be driven to a known-good reset, or do some devices make crash-restart lossy in ways the client cannot fully paper over?
- How much reward should a proved-Omega driver earn over a confined C one (extra trust, lower overhead, fewer IOMMU round-trips), given that safety no longer depends on it?
- What is the IOMMU's real cost on the hot path (remap plus cache-flush), and where does that put the zero-copy/bounce threshold once there is hardware to profile?

## Related
- [[component_model]] — drivers are components; this is the hard case.
- [[error_model_and_recovery]] — driver crash containment and restart.
- [[kernel_architecture]] — what hardware authority the privileged core retains.
- [[capability_model]] — capability-limited register/DMA/IRQ access.
- [[store_and_economic_control]] — driver certification and distribution.
