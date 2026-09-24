# Repository Layout: the Cathedral source tree

> **Status: DRAFT (2026-07-02).** The canonical plan for how Cathedral's source
> is organized. This document is the architecture; the filesystem grows into
> it. The early boot path and its contracts occupy the first planned
> directories. Remaining directories are created only when real code lands
> (see [The no-scaffolds rule](#the-no-scaffolds-rule)).
>
> Method borrowed from Dolrus (`C:\Projects\Dolrus`): plan in one document,
> iterate the plan as its own phase, stress-test it against reality, never
> pre-create empty scaffolds. Conventions inherited from Omega
> (`../../../Omega/wiki/architecture/`): directories-are-the-architecture,
> machine-enforced downward dependencies, one-owner-per-concept, no invented
> abbreviations.

---

## North star: navigable by scanning

The layout has one job above all others: a human scanning the directories can
answer obvious structural questions without opening a file. The trust boundary
is the architecture, so the tree makes it visible.

Two scans answer two questions. Root tells you what the repository is.
`source/` tells you what the OS is.

| Question | Answer by scanning |
|---|---|
| What is this repository? | `wiki/` (its contracts, rationale, and explanations), `source/` (the OS), `tools/` (what builds it). |
| Where does the kernel end and userspace OS code begin? | `source/core/` is the kernel (the proved TCB). `source/services`, `drivers`, `libraries`, `applications` are userspace. |
| System service vs plain program vs boot code? | `source/services/` vs `source/applications/` vs `source/boot/`. |
| What must I trust for the system's invariants to hold? | `source/core/` + `source/contracts/`, plus the trust-critical services named in [`tcb.md`](tcb.md). Nothing else. |
| What talks to hardware? | `source/drivers/` (device programs) and `source/boot/` (firmware seam). `source/drivers/facts/` is pure hardware data with zero authority. |
| What's the frozen ABI everything targets? | `source/contracts/`. |
| What runs on my dev machine vs on the target OS? | `tools/` is host-side and never ships. `source/` owns target code, including ports awaiting integration. |

If a future change makes any of these answers require opening a file, the
change is wrong.

---

## The tree

Root separates the OS from everything about the OS. Inside `source/`, the OS is
ordered by trust, descending, so reading top to bottom teaches the architecture.

```
Cathedral/
├── README.md
├── wiki/          Documentation truth. wiki/spec owns current contracts; wiki/design owns
│                  rationale and direction; wiki/architecture owns this layout and the TCB;
│                  wiki/boot explains cross-cutting flows; wiki/proposals and wiki/drafts are
│                  non-normative work; wiki/decisions records ADRs; wiki/speculation parks
│                  non-committed exploration. wiki/README.md defines their authority.
│
├── tools/         Host-side, NEVER-SHIPS tooling — the SDK, the reference IDE, the debugger,
│                  the hostile simulator, CI gate runners, image assembly, the migration tester.
│                  If it runs on the developer's machine, not the target, it is here.
│
└── source/        THE OS — target code, including translated units awaiting integration.
    │
    ├── contracts/     THE FROZEN ABI everyone targets (governance tier 1). The capability-type
    │                  vocabulary, the kernel/syscall surface, IPC wire schemas, the boot handoff,
    │                  the component + manifest format, the checker's admission contract.
    │                  Depended-upon by everything; depends on nothing. Changes only by the
    │                  versioned-interface discipline (additive + migrated, never redefined).
    │
    ├── core/         THE PROVED KERNEL — the trusted computing base, and nothing more. The
    │                  capability arena, scheduler, memory/address-space manager, IPC region
    │                  manager, tickless timers, trusted-time keeper, trusted spawn/loader,
    │                  hot-swap engine, transaction commit coordinator, attestation reporter,
    │                  and the proof checker. Proofs live HERE, beside the code they cover —
    │                  never in a sibling that can lag. Small enough to audit in full. HARD EDGE.
    │
    ├── foundation/    The shared library usable INSIDE core/ AND above it — kernel-safe by
    │                  construction (no allocator assumptions, no ambient authority). Data
    │                  structures, the handle/arena discipline, ZII helpers. The ONE thing both
    │                  sides of the trust line share. Charter-bounded so it never sprawls.
    │
    ├── services/      Userspace system servers — one package per server. Compositor, audio,
    │                  network, storage/realm, activator/supervisor, preferences, Warden,
    │                  package/install, observability, print, power governor, clipboard, the
    │                  stock Matrix mediator (which also owns the shipped Matrix presets), and
    │                  the rest. A few are trust-critical minters (see tcb.md); that fact lives
    │                  in their manifest, not their tree position.
    │
    ├── drivers/       Userspace, capability-confined device programs — structurally just
    │                  programs holding device capabilities. Organized by class. drivers/facts/
    │                  holds pure hardware description data (register maps, descriptor layouts,
    │                  quirk tables) that holds ZERO capabilities — transcribable from datasheets,
    │                  reviewable without trust, testable without hardware.
    │
    ├── libraries/     Shared userspace packages that are NOT servers — transport (TCP/QUIC/TLS,
    │                  ICE), text shaping/locale, audio DSP filters, IPC/channel helpers, the
    │                  inference runtime. Ordinary code a program links.
    │
    ├── applications/  Default programs shipped with the OS — un-blessed capability-holders.
    │                  File browser, shell, task manager, world chooser, legibility agent, the
    │                  browser (open-web Matrix). None is special; each holds explicit granted
    │                  capabilities, never ambient authority. (A vendor telemetry collector, if
    │                  shipped, is an ordinary app and lands here — never in services/.)
    │
    └── boot/          The firmware seam, below core/. Per-firmware loaders (boot/uefi/ = the
                       Omega UEFI application). Lives in a different reality (pre-capability,
                       different memory rules) so it never pollutes core/'s invariants. The
                       handoff CONTRACT lives in contracts/, owned by the core side — the loader
                       implements it, never dictates it.
```

Day one this is `README.md` + `wiki/` + `source/`. `tools/` and each `source/`
subdirectory appear only when real code lands in them.

---

## The dependency law

Dependencies flow downward only among the `source/` subdirectories. The rule is
enforced mechanically (see [Enforcement](#enforcement)), not by vigilance.

```
applications  → libraries, foundation, contracts
services      → libraries, foundation, contracts
drivers       → libraries, foundation, contracts, drivers/facts
libraries     → foundation, contracts
boot          → foundation, contracts
core          → foundation, contracts
foundation    → (nothing)
contracts     → (nothing)
```

`wiki/` and `tools/` sit outside this graph. `wiki/` is prose. `tools/` is
host-side and ships nothing, though it may read `source/contracts/` for
schema-awareness.

Three rules make the kernel edge real:

- **Userspace never build-depends on `core/`.** Everyone targets the ABI in
  `contracts/`, and `core/` implements those contracts. In the rest of the
  tree's build graph, `core/` is a leaf that nobody depends on. This is what
  keeps the kernel swappable and the boundary crisp. It is the microkernel
  discipline: the contract is the coupling, not the implementation.
- **`boot/` depends on `contracts/`, never on `core/`'s internals.** It
  constructs the machine state the handoff contract promises. It does not reach
  into the kernel.
- **`contracts/` and `foundation/` are the two roots.** They depend on nothing
  and everything depends on them. Keep them small and stable, because churn
  here ripples everywhere.

---

## Reach: imports are declared, never ambient

The dependency law is enforced by the same discipline the OS uses at runtime: a
package reaches only what it declares. There is no reaching up the tree.

- **Designation is by package name, never by path.** Imports name a package and
  a symbol (`use contracts.UefiHandoff`), never a filesystem path (`../../`).
  Moving a package never breaks its importers, and the tree structure does not
  leak into the code. Omega already resolves imports this way (chapter 15).
- **The manifest is the reach-set.** Each package declares its dependency
  packages by content hash, per the pinned-closure model in
  `developer_experience`. A package may import only from its declared
  dependencies. A package it did not declare is not nameable. The manifest is
  the package's held reach-capabilities, an import invokes one, and an
  undeclared package is unforgeable because you cannot say its name.
- **The layer law becomes self-enforcing, not merely checked.** `boot`'s
  manifest lists `[contracts, foundation]` and omits `core`, so `boot` cannot
  express an import of `core`. The impossibility is the point, the same way
  ambient authority does not exist at runtime.
- **Two-sided contract.** `pub` (Omega visibility) says what a package offers.
  The manifest says what it may reach. Neither is the other's job.

Enforcement comes at two strengths:

- **Without a language change:** the ported layering test checks
  `import graph ⊆ declared-manifest dependencies` and fails the build on any
  undeclared reach. It is the same test that enforces the downward layer law.
- **Target (an Omega ask, tracked in `cathedral_alignment.md`):** the name
  resolver itself gates on the declared set, so a fully-qualified package path
  cannot bypass the manifest. Undeclared reach is unresolvable at compile time,
  not caught by a separate pass.

---

## Placement decision-procedures

Phrased so a stranger applies them without discussion. If behavior is X, it
belongs in Y (all paths under `source/`):

- **Proved, in the TCB: memory-safety / scheduling / authority-minting core** →
  `core/`. If it isn't small enough to audit in full, it doesn't belong here.
- **A frozen contract other components target** (a wire schema, a capability
  type, the syscall surface, the boot handoff) → `contracts/`.
- **Shared by both `core/` and userspace, kernel-safe** → `foundation/`.
  If only userspace uses it → `libraries/`.
- **A long-running userspace program that serves an endpoint or owns a resource**
  → `services/`.
- **A userspace program that drives a hardware device** → `drivers/`. Its
  register maps and quirk tables (pure data) → `drivers/facts/`.
- **Shared userspace code that isn't a server** → `libraries/`.
- **A default user-facing program holding granted (not ambient) authority** →
  `applications/`.
- **Firmware-facing loader code, pre-capability** → `boot/`.
- **Runs on the developer's machine, never ships to the target** → `tools/`
  (at root, outside `source/`).

Two tie-breakers:

- *Trust before subject.* A thing's trust tier decides its home before its topic
  does. A network-flow minter is `core/`-adjacent authority, not "the
  networking folder."
- *When unclear, it does not exist yet.* Don't invent a home for speculative
  code; the need will name the home.

---

## Where cross-cutting artifacts live

Artifacts live with their one owner. Only the frozen platform ABI centralizes.

| Artifact | Home |
|---|---|
| Tier-1 wire schemas, capability vocabulary, syscall/handoff/manifest formats | `source/contracts/` |
| A service's own IPC protocol | co-located with that service (one-owner rule) |
| A program's capability manifest | co-located; it is part of the package |
| Proof certificates | beside the code they prove (core's proofs in `source/core/`) |
| Matrix presets (Default / Locked-down / Throwaway / Trusted) | co-located with the stock mediator in `source/services/`; they are shipped data, not a source layer |
| Migration machines (`Upgradable<Old,New,Ctx>`) | beside the schema they migrate |
| Whole-system image compositions (bootable/testable configurations) | a build concern under `tools/` (image assembly); data, not source |
| Unit tests / canaries | beside the component (see [Testing](#testing)) |

There is no central `manifests/` or `schemas/` junk drawer, and no `worlds/`
directory. A "world" is a Matrix, a runtime realm, not a source concept. The
manifest is part of the component. The frozen contracts are the only thing
that earns a shared home.

---

## The TCB is distributed and enumerated

`core/` is the proved kernel, but the full must-trust set is larger. A handful
of userspace services mint authority: the trusted core is the origin of
authority, distributed across resource owners. The network broker mints flows,
the compositor mints seats and surfaces, the storage/realm service mints realms
and runs unseal, and the Warden holds the secure-element. These live in
`source/services/` for navigability. They are big userspace programs, and
forcing them into `core/` would blur the kernel edge. Their trust status is not
hidden:

- Each declares it in its manifest.
- [`tcb.md`](tcb.md) enumerates the exact audit surface: `core/` + `contracts/`
  + the named minting services. That document is the answer to "what must I
  trust," kept in one place rather than smeared across the tree.

This is the seL4 lesson of a small, fully-auditable trusted set, adapted to a
system where minting is distributed rather than monolithic.

---

## Naming conventions (inherited from Omega)

- **No invented abbreviations.** `applications`, not `apps`; `arguments`, not
  `args`. Standard domain acronyms (MMU, IPC, IOMMU, UEFI) are fine.
- **Names carry the layering.** A package is named for what it owns. Its
  directory path mirrors its name, and file names inside describe the behavior
  implemented.
- **Entry/structure declarations stay thin.** A package's top-level file
  declares structure and re-exports. Implementations live in behavior-named
  files, never in a catch-all.
- **A program and a library are the same kind of thing.** Both are an Omega
  package; "runnable" means it exposes a `main` interface (see
  `developer_experience`). The `services/` vs `libraries/` vs `applications/`
  split is role for navigation, not a format distinction.

---

## The no-scaffolds rule

A directory is an assertion that real code lives there. We do not create empty
directories or placeholder packages to mirror this plan; the rule is adopted
from Dolrus ADR-0001. Consequences:

- This document holds the full intended tree with per-region status markers.
  The filesystem holds only what exists.
- A directory is created when its first real resident lands, and gets a one-page
  `CHARTER.md` at that moment stating what belongs, what is out, and its
  dependency rules.
- Planned-but-unbuilt structure is legible here, with its status marked, never
  by perjuring the tree with empty folders.

Status legend: built, in-progress, planned, parked. Current reality
(2026-07-28):

- **built (boot-verified):** milestone 1 of the first-boot ladder is reached.
  An Omega UEFI application boots under QEMU/OVMF and prints "Hello from Omega"
  through the firmware's Simple Text Output protocol. The canonical compiling
  reference is Omega `samples/uefi_hello`, ea51376cd; Cathedral
  `source/boot/uefi/` and the milestone-1 path of `source/contracts/uefi/` are
  aligned to that boot-verified shape. No C, no host runtime, no hand-written
  assembly. `tools/boot-harness/` runs it end-to-end under QEMU/OVMF.
- **built (boot-verified):** milestone 2 (own the machine) is reached.
  `source/core/extent.omg` is the checked adapter for Omega's shared linear
  `Extent` and owner-authored root-provider requirement.
  `source/boot/uefi/own_machine.omg` runs the memory-map dance, calls
  `ExitBootServices`, and yields one receipt-backed `Extent in Granted`,
  carried through the post-firmware graph into owned idle. Both sit over the
  milestone-2 ABI in `source/contracts/uefi/boot_services.omg`. QEMU/OVMF
  prints the owned-memory report after that crossing. The report uses
  `99999+ MiB` as a lower bound when the exact value exceeds its five-digit
  FIFO-sized formatter, and saturates the conversion after 100,000 subtraction
  rounds. Each FIFO-readiness wait is capped at 1,000,000 status reads and
  parks owned on exhaustion. Live in this path: bounded map resize;
  fail-closed System/Boot Services table-header metadata admission (signature,
  shared well-formed UEFI revision, consumed-prefix size, and reserved zero;
  CRC still outstanding); a single whole-transaction stale-key refresh;
  revision-pinned aligned whole-descriptor traversal; runtime-region
  exclusion; descriptor type/attribute/geometry checks; selected-span
  disjointness auditing; and conservative numeric root-geometry validation.
  Richer physical-space/right/backing facts remain follow-on hardening rather
  than a substitute for the qualified root.
- **planned:** everything else (`foundation/`, `services/`, `libraries/`, and
  the driver programs under `drivers/`). No directory exists until real code
  lands in it.

---

## Testing

- **Unit tests co-locate with the code they test**, as adjacent files inside
  the package. There is no parallel `tests/` shadow tree. Every surveyed OS
  that built one (Haiku, Serenity) watched it drift.
- **Canaries** follow Omega's structure, outcome-first and then area:
  `canaries/pass/<area>/<behavior>/` and `canaries/fail/<area>/<behavior>/`
  with an expected-diagnostic file. They are named by behavior under test,
  never by the incident that motivated them.
- **Whole-system simulation** is a Matrix composition, not a source layer. A
  synthetic top-level Matrix hosts the components under test, assembled as an
  image under `tools/` and run in the hostile simulator.
- **A differential oracle**, as Omega runs one, is the standing mitigation for
  the parts that can't be proved.

---

## Enforcement

- **Machine-checked layering.** Port Omega's `omega-architecture-test`: read the
  package dependency graph, fail the build on any upward edge not in a
  `KNOWN_EXCEPTIONS` allowlist, and fail when a stale exception no longer
  matches a real edge, so the policy only ratchets tighter. The layer ranks are
  the order in [the dependency law](#the-dependency-law).
- **Charters as the local law.** Each `source/` subdirectory's `CHARTER.md`
  states its scope, non-goals, and dependency rules. It is cited when deciding
  where new code belongs.
- **Monorepo atomicity is the point.** A change to `core/` that breaks its
  proofs does not merge; the proofs update in the same commit. This is only
  possible because everything is one repo. It is the single biggest structural
  advantage Cathedral holds over seL4, whose proofs sit in a lagging sibling
  repo, and Redox, whose components sit across submodules. The layout exists to
  spend it.

---

## Omissions (negative space is part of the plan)

- **Single-language target code, no Rust crate dependency model.** Cathedral
  maintains its Omega implementation in this monorepo. Properly licensed
  derivative translations of useful representations, algorithms, and tests are
  permitted; Rust vendor trees and ambient-authority APIs are not imported.
  Primary-source facts, licensed translations, and Cathedral integration have
  distinct provenance and verification claims. See
  [`prior_art_and_hardware_facts.md`](prior_art_and_hardware_facts.md).
- **Ports live with their eventual owner, not in a parallel vendor hierarchy.**
  Studied source stays in gitignored `reference_code/`. Committed translations
  live under the existing `source/` ownership layers with `PORT.md`, retained
  notices, source/symbol mappings, tests, and blockers. Root
  `THIRD_PARTY_NOTICES.md` indexes the upstream pins and licenses. A translated
  package may land before Omega supports it, but an untypeable package must
  remain outside production build roots. Presence under `source/` alone does
  not mean code is compiled, tested, shipped, or integrated. The layer law,
  charters, and separately reviewed authority/lifecycle integration still apply.
- **No `session_logs/` or agent-runbook machinery.** Cathedral is human-driven.
  The autonomous-loop scaffolding that suits Dolrus would be the exact clutter
  this layout exists to avoid.
- **No `worlds/` directory.** A "world" is a runtime Matrix realm, not a source
  concept. Matrix presets are shipped data, kept with the mediator in
  `services/`; system images are a build concern under `tools/`.
- **Drivers are count-budgeted.** `drivers/` swallows every OS repo that lets
  breadth accrete ambiently; Linux's `drivers/` is two-thirds of the tree.
  Cathedral ships virtio/simulated devices plus one exemplary real driver per
  class first. Broad hardware support is a later purchase, logged as such,
  never an ambient accretion.

---

## Open judgment calls (flagged, not blocking)

Revisit these if reality argues otherwise.

1. **`foundation/` vs folding shared primitives into `libraries/`.** We keep
   `foundation/` separate because the kernel-safe-shared-code distinction is
   real, as Serenity's AK proves, and a charter keeps it from sprawling.
2. **Contracts central vs co-located.** Hybrid: the tier-1 frozen ABI
   centralizes in `contracts/`; per-service protocols co-locate. This is the
   one place the one-owner rule and the idea that interfaces stand on their own
   are balanced by hand.
