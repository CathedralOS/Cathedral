# Chapter 02: Driver Model

> Drivers are ordinary components — isolated, capability-limited, and restartable — with hard effect ceilings on the hardware they touch, never privileged kernel blobs.

## The Legacy Model

A traditional driver is a binary blob loaded into the kernel's address space with *total* authority: it can touch any memory, program any DMA engine, take any interrupt, and a single bug crashes — or silently corrupts — the whole system. There is no capability boundary between "this NIC driver" and "all of physical memory." Driver crashes are fatal; updates mean reboots; certification is a signature on an opaque blob. And because every PC has thousands of devices, the driver surface area is the single largest reason OS projects die before shipping.

## The Cathedral Model

Drivers are components like any other ([[component_model]]): isolated, holding only the specific capabilities for *their* device, restartable after a crash without taking down the system, and upgradable in place via the hot-swap machinery ([[updates_and_hot_swap]]). A driver's hardware reach is an **effect ceiling of boundary-trait service identities** plus narrow capabilities over its register extents, DMA loans/IOMMU domain, and interrupt line. There are no ambient `device_io`/`memory_map` keyword powers and no arbitrary kernel driver blobs.

**Coverage is the deciding constraint — but confinement makes it an *effort* problem, not a *safety* one.** Driver surface area is what kills OS projects, but a bad or foreign driver is always *safe* (contained), so the combinatorics of arbitrary hardware no longer threaten the system's isolation — only *who writes and maintains each driver*. The strategy follows: **tolerate freely, reward Omega, lead with class drivers.** Most hardware speaks a standard *class* protocol (USB HID / mass-storage, NVMe, AHCI, XHCI, HD Audio), so one proved-Omega driver per class covers the bulk with no per-device work; the proprietary long tail (a GPU blob) runs *confined and second-class* — which still beats an in-kernel blob. Proved-Omega drivers earn the premium (proof, zero-copy, live-upgrade, more trust); confined C/blob drivers work and stay safe but lose it, and their faults still cost *their own* device's availability, data, and performance (contained ≠ harmless). LLM-assisted authoring and porting lower the native cost further; the real bottleneck for the tail is undocumented *hardware specs*, not code volume.

### The decided mechanism

The model is **confinement over trust**: don't try to make a driver trustworthy — make its trustworthiness *not matter*. A driver runs as an ordinary user-mode component, walled by hardware and confined by capabilities, so a driver that is buggy, hostile, miscompiled, or written in plain C can corrupt only the one device it already owns. Proved-Omega drivers are rewarded (more trust, less overhead) but never *required* for system safety — the only honest stance while the toolchain is young, and what lets Cathedral accept third-party or vendor-written drivers without each one being a system-wide risk.

- **IOMMU is the hardware floor.** Device DMA bypasses the CPU and its MMU entirely, so the MMU cannot contain a device — only the IOMMU can ([[kernel_architecture]]). Cathedral therefore *requires* an IOMMU and treats a machine without one as a weaker-guarantee target rather than contorting the architecture around legacy hardware. This retires the "safe DMA without an IOMMU" question.
- **The capability manifest.** A driver instance starts with exactly: a plan-derived placed view over its mapped register `Extent`; DMA submission authority plus its IOMMU domain; its IRQ endpoint; and service channels. Nothing ambient — its blast radius is precisely those grants.
- **The core owns the confinement machinery.** The privileged core programs the IOMMU and maps MMIO; the driver only *requests*. It must be this way: programming the IOMMU *is* the confinement, so a driver that could shape its own DMA domain could unconfine itself. The driver sits outside the TCB; the core's hardware-access broker is inside it.
- **Interrupt = message.** The only kernel-mode code on the path is a tiny, device-agnostic IRQ stub: it acks/masks the line (no storms) and turns the interrupt into a wakeup for the parked driver actor (the scheduler's existing park/unpark, [[scheduler_and_resources]]). No device-specific code runs privileged. Extreme-throughput devices skip interrupts and **poll** — the driver busy-reads a DMA ring with no kernel hop, the shared-region primitive ([[ipc_and_service_invocation]]) as the ring.
- **Restart is the easy cousin of hot-swap.** A crash discards state instead of preserving it — no quiescence-snapshot, no migration. The core tears down the dead instance's IOMMU domain and capabilities *first* (cutting the device off RAM before that memory is reused), resets the device to a known state, and spawns a fresh instance. Recovery is **visible and typed**, never a silent auto-replay: the client's channel reports the restart and the layer holding the semantics (an FS journal, a protocol's retry) decides what to redo, with a standard reconnect helper so visible is not painful ([[error_model_and_recovery]]).
- **Device reset is core-driven; containment before cleanliness.** "Reset to a known state" is a *hardware* property, not a driver promise, so the core drives it over the dead driver's head and splits two things. **Containment** — the device can't *hurt* anything — is guaranteed instantly by the IOMMU teardown above. **Cleanliness** — the device back to known-good — is best-effort up a ladder: soft-quiesce (asks the driver; optional) → **FLR** (function-level reset; the core writes PCIe config space, driver-independent, and the silicon wipes the driver's state) → bus/port reset → power-cycle → mark-dead. So what we *require* is that the **device** support a core-drivable reset (a hardware admission check) — never that the *driver* reset correctly. A reset available only at bus level cascades to siblings, so reset domains form a tree the core coordinates. Irreversible physical effects (a printed page) aren't resettable at all: reset restores *controllability*, not the world, and the in-flight-unknown is surfaced by the visible+typed recovery above.
- **DMA copy strategy: zero-copy primary, bounce fallback, one call.** A driver asks the broker to submit an authorized extent; the broker chooses direct IOMMU mapping or a bounce buffer and returns a linear transfer token. Device-read excludes CPU mutation, device-write excludes CPU access, and consuming completion returns the loan after required cache/fence work. The token may survive task suspension. Reset/revocation completes IOMMU invalidation and acknowledgement before reuse. The direct/bounce crossover remains a measured provider policy.

## Concerns & Design Space

- **Capability-limited hardware access.** A driver holds capabilities for its MMIO range, its DMA windows, and its IRQs — not ambient hardware power ([[capability_model]]).
- **DMA isolation.** The dangerous core: a device that can DMA anywhere defeats every software boundary. Requires IOMMU-backed, capability-scoped DMA windows.
- **Interrupts become messages above a real entry root.** Hardware still enters a tiny target-specific boundary root with a pinned `CallPlan + StatePlan`; installation records it in the external-root ledger. The root acknowledges/masks and signals the driver endpoint. Device-specific processing remains in the ordinary driver task. DMA extents are external loans; registers are plan-derived placed views. Interrupt delivery and outstanding loans both pin live replacement.
- **Crash recovery & restartability.** A driver crash is contained and recovered, not fatal ([[error_model_and_recovery]]); device state is re-established on restart.
- **Hot plugging & device discovery/matching.** Discovery enumerates devices; matching binds a device to a driver under explicit, auditable policy.
- **New-device trust.** A freshly hot-plugged device is not trusted by default. A new input device cannot drive the trusted path or inject events until authorized, the BadUSB defense, and a new storage device appears as an untrusted realm the user browses and shares from explicitly rather than something apps are auto-granted ([[human_permission_ux]], [[windowing_and_compositor]]).
- **Peripheral classes are ordinary devices.** Printers, scanners, cameras, and USB peripherals are ordinary device-plus-driver-plus-capability instances rather than special subsystems. Printing is a userspace print service rendering to a page description, plus a printer driver, plus a "may print here" capability, with the spooler a queue component; the OS adds no print subsystem beyond the driver and the service.
- **Foreign filesystems and removable media.** A drive formatted for another OS (FAT, exFAT, NTFS) is mounted as a realm by a translating filesystem driver that implements the realm interface over the foreign layout ([[filesystem_as_database]]). You read and write files, but the native features (content-addressing, integrity, type metadata, snapshots, sealing) are absent because the foreign format cannot hold them. So a removable drive is an interop-versus-features choice: keep it foreign so it travels to other systems, or reformat it native for the full feature set. The foreign parser runs as a confined driver, so a malicious removable filesystem degrades from a kernel compromise to a contained driver fault.
- **Versioned driver APIs.** Kernel↔driver and driver↔client interfaces use ordinary numbered schemas, layout/codec policies, immutable historical shapes, and checked conversions. Live driver replacement is a separate component protocol; neither requires `wire data` nor `Versioned<T>` syntax.
- **User-mode, contained not trusted.** Drivers run in user mode, confined by capabilities and the IOMMU, so a driver's *correctness is never relied on for system safety* (see the decided mechanism above). The driver↔hardware contract is a `boundary`; proved Omega is rewarded, not required.
- **Synthetic devices.** A driver presents a device interface, so a component can serve a *synthetic* device to a child instead of real hardware: a virtual NIC, framebuffer, sound, or block device. This is the recursive-provider pattern ([[capability_model]]) at the hardware edge, and it is what lets a virtual machine present hardware to a guest and a simulator hand mock devices to code under test ([[testing_and_simulation]]).
- **Firmware updates, device permissions & certification.** Firmware flashing is a capability-gated operation; device access is granted, not assumed; driver certification flows through the store ([[store_and_economic_control]]).
- **Zero value.** Optional device selection may use an explicit `Empty | Live(Device)` sum. Zero-fill never mints a register extent, DMA loan, IRQ, or provider grant. A synthetic null device is an explicitly selected provider, not forged authority hidden in zero bits ([[omega_substrate]]).

## Key Questions

The confinement model (above) settles the trust question; the residue is device-side and sizing:

- **Device reset residue.** The containment/cleanliness split + core-driven ladder (above) handle the common case; the residue is the *wedged* device that ignores even FLR (forcing bus/power escalation or mark-dead), and coordinating the **shared-bus cascade** where resetting one device restarts its siblings. Shared with hot-swap's device-quiescence corner.
- **The minimal hardware broker.** The core owns IOMMU/MMIO programming and the IRQ stub — what is the *smallest* such broker surface, and how little device knowledge can it carry while staying generic ([[kernel_architecture]])?
- **Quiescence for hot-swap, not crash.** Upgrading a *live* driver (vs. restarting a dead one) still needs the device quiesced enough to snapshot — which interrupt and power paths can reach that, and which force a brief device-quiesce ([[power_management]], [[updates_and_hot_swap]])?

## Omega Leverage

- **Drivers as components with boundary-trait effect ceilings** make hardware reach a checkable, audited fact without blessed lowercase keywords. See [Omega effects](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md).
- **Extents, placed views, linear external loans, and entry-root plans** scope MMIO/DMA/IRQ access to one device ([[hardware_foundation_profile]]).
- **Ordinary numbered schemas + layout/codec policies** give compatible driver APIs; component identities and quiescence govern live replacement.
- **Boundary traits** model the device edge: the hardware contract is a `boundary` whose guarantees are accepted but whose effects and authority are bounded.
- **Blocking-boundary modeling** lets a driver call that waits on hardware declare what can unblock it, rather than being an opaque wait.

## Open Questions

- Can a device's half-state always be driven to a known-good reset, or do some devices make crash-restart lossy in ways the client cannot fully paper over?
- How much *reward* should a proved-Omega driver earn over a confined C one — extra trust, lower overhead, fewer IOMMU round-trips — given safety no longer depends on it?
- What is the IOMMU's real cost on the hot path (remap + cache-flush), and where does that put the zero-copy/bounce threshold once there is hardware to profile?

## Related
- [[component_model]] — drivers are components; this is the hard case.
- [[error_model_and_recovery]] — driver crash containment and restart.
- [[kernel_architecture]] — what hardware authority the privileged core retains.
- [[capability_model]] — capability-limited register/DMA/IRQ access.
- [[store_and_economic_control]] — driver certification and distribution.
