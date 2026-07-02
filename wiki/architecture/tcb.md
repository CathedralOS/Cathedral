# The Trusted Computing Base — Cathedral's audit surface

> **Status: DRAFT (2026-07-02).** The single answer to *"what must I trust for
> the system's invariants to hold?"* Derived from the design chapters; this
> document **owns** the enumeration, so a component's trust status is decided
> here (and mirrored in its manifest), not inferred from its position in the
> [source tree](repository_layout.md).
>
> The goal is not zero trust — that is impossible — but a **small, enumerable,
> honestly-stated** trusted set. Everything not on this list is
> *contained-not-trusted*: it may be fully adversarial and still cannot break
> the system's invariants, because the capability system, the IOMMU, and the
> checker confine it.

---

## What "trusted" means here

A component is in the TCB if a defect or compromise in it can violate a
system-wide invariant (memory safety, authority confinement, the integrity of
the capability graph). Trust is a property of **what a component can do to the
whole system**, not of how important it is. A file browser is critical to daily
use and holds broad capabilities, yet is **not** trusted — it can only misuse
what it was granted, and revocation contains it. The compositor is trusted, not
because it is important, but because it *mints* seats and surfaces: a bug in it
can hand authority nobody granted.

The trusted set is the union of six regions. Read top to bottom: it descends
from what sits below us, through the anchor, into the proved core, and out to
the distributed minters.

---

## 1. Below Cathedral — trusted, not authored by us

- **Platform firmware + the hardware root of trust.** On commodity x86 the
  firmware, ME/PSP, and DRAM-init blobs sit *below* the OS; Cathedral is the
  firmware's child and honors that sandbox. Measured boot anchors the chain in
  hardware. On an owned-firmware reference platform this region shrinks to code
  we authored (see the Omega boot brief's firmware seam), but the hardware root
  is irreducible.
- **The silicon.** Constant-time guarantees, capability tags (CHERI where
  available), and the IOMMU are hardware facts we depend on. `{hardware}` is a
  permanent member of the trust base.

*Source: `part_5_lifecycle/03_boot_and_trust_chain`,
`part_2_components/05_power_management` (below-SoC boundary).*

## 2. The anchor — the checker and its inputs

The deepest trust bottoms out in `{seed, checker, specs, hardware}`:

- **The proof checker** — Omega's own first-party lattice checker, the single
  component that re-checks every code-certificate and proof-certificate before
  admission. Auditing your own checker is the irreducible core.
- **The bootstrap seed** — a hand-audited compiler seed, so no trusting-trust
  binary sits in the lineage.
- **The specifications** — the soft, load-bearing entry: a proof shows
  code-meets-spec, never spec-meets-intent (that gap is unformalizable). Kept
  small, legible, and tested; "proven" is never sold as "correct."

*Source: `part_5_lifecycle/04_kernel_architecture` (the trust-base sharpening).*

## 3. `source/core/` — the proved kernel, trusted in full

The proved single-address-space core. Small enough to audit in its entirety;
its proofs live beside it. Members:

- Capability grant arena — the live authority graph.
- Scheduler.
- Memory & address-space manager.
- IPC region & lease manager.
- Tickless timer subsystem.
- Trusted-time keeper.
- Trusted spawn / component loader.
- Hot-swap / replacement engine.
- Transaction commit coordinator.
- Attestation reporter.
- Hardware-access broker (IOMMU / MMIO gatekeeper).
- Generic interrupt stub.
- Boundary-provider registry (only registered providers may mint host/hardware
  boundaries — the anti-escape-hatch).

*Source: `part_5_lifecycle/04_kernel_architecture`,
`part_1_authority/01_capability_lifecycle`, `part_2_components/*`.*

## 4. `source/contracts/` — the frozen ABI, trusted by definition

The capability vocabulary, syscall surface, wire schemas, boot handoff, and
manifest format are not executing code, but they are the agreed *definitions*
everything targets. A change here redefines the platform, so the contract layer
is trusted the way a constitution is: it governs by being the reference, and
evolves only by the versioned-interface discipline.

## 5. `source/services/` — the distributed minters

The trusted core is *distributed across resource owners*: a small set of
userspace services mint authority against the metal. Each lives in `services/`
for navigability but is TCB, and each is meant to stay "small enough to be
TCB-worthy" (the minimal-broker pattern — route tickets, never hold authority —
shrinks several of these).

| Service | Why trusted — what it mints |
|---|---|
| **Compositor (root)** | Mints seats and surfaces; the boundary provider for display/input hardware and the trusted path. |
| **Network broker / demux** | Mints flow capabilities; owns the single NIC path. The QUIC classifier loads into its router domain — `RegisterClassifier` is privileged. |
| **Storage / realm service** | Mints realm roots and runs the `unseal` primitive; the origin of storage authority. |
| **Warden (root)** | Holds the secure-element device capability; the root key-manager. (Non-root Wardens are contained.) |
| **Admission gate** | The install-time re-check (manifest-vs-policy, re-verified proof certificates); held by the machine owner. |

*Source: `part_1_authority/07_sessions_and_login` (the distributed origin of
authority), `part_3_communication/01_networking`,
`part_6_human_surface/00_windowing_and_compositor`,
`part_1_authority/08_wallet_and_credentials`,
`part_7_governance/03_store_and_economic_control`.*

## 6. `source/boot/` — measured, pre-capability

The UEFI loader shim is tiny and measured; it is trusted because a compromised
loader compromises the measured chain, but it hands off to the proved core the
moment the capability world is constructed. Its authored surface is small by
design.

---

## Explicitly NOT trusted (contained-not-trusted)

Everything else, confined by construction:

- **All drivers** — IOMMU-confined; a compromised driver kills its device, not
  the machine.
- **All applications** — capability-confined; hold only granted, revocable
  authority (file browser, shell, task manager, browser, legibility agent).
- **All libraries** — ordinary linked code with no ambient authority.
- **The inference model / AI agents** — the model is a *tool* holding no
  authority; the agent is the principal, and injection is contained by
  minimizing its standing authority (never eliminated, but bounded).
- **Telemetry collectors, foreign/legacy code, the compatibility box** —
  walled; foreign binaries run behind the hardware wall with no proof guarantee.

The line is not intra-vs-cross-app; it is **proven-connected-graph vs unproven
boundary**. Proof (and therefore trust that can be *earned* rather than
*assumed*) stops at the first unproven or foreign node.

---

## Keeping this honest

- This document is the one place the trusted set is enumerated; every entry
  cites the chapter that puts it there. When a chapter adds or removes a minter,
  update this map in the same change.
- Each trusted service also declares its status in its own manifest; a
  service in `services/` whose manifest claims minting authority but is not
  listed here is a discrepancy to resolve, not a silent grant.
- The standing pressure is *shrink this list*. Every minter that can be
  reduced to a ticket-router (holding no authority itself) should be, and moved
  to the contained set.
