# Repository Layout — the Cathedral source tree

> **Status: DRAFT (2026-07-02).** The canonical plan for how Cathedral's source
> is organized. This document is the architecture; the filesystem grows into
> it. No code exists yet — every directory below is described here *before* it
> is created, and directories are created only when real code lands in them
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

The layout has one job above all others: **a human scanning the directories can
answer obvious structural questions without opening a file.** The trust
boundary is the architecture, so the tree makes it visible.

Two scans, two questions. **Root** tells you what the *repository* is;
**`source/`** tells you what the *OS* is.

| Question | Answer by scanning |
|---|---|
| What is this repository? | `wiki/` (why it's built this way), `source/` (the OS), `tools/` (what builds it). |
| Where does the kernel end and userspace OS code begin? | `source/core/` is the kernel (the proved TCB). `source/services`, `drivers`, `libraries`, `applications` are userspace. |
| System service vs plain program vs boot code? | `source/services/` vs `source/applications/` vs `source/boot/`. |
| What must I trust for the system's invariants to hold? | `source/core/` + `source/contracts/`, plus the trust-critical services named in [`tcb.md`](tcb.md). Nothing else. |
| What talks to hardware? | `source/drivers/` (device programs) and `source/boot/` (firmware seam). `source/drivers/facts/` is pure hardware data with zero authority. |
| What's the frozen ABI everything targets? | `source/contracts/`. |
| What runs on my dev machine vs on the target OS? | `tools/` is host-side and never ships. Everything in `source/` ships. |

If a future change makes any of these answers require opening a file, the
change is wrong.

---

## The tree

Root separates the OS from everything *about* the OS. Inside `source/`, the OS
is ordered by trust, descending — reading top to bottom teaches the architecture.

```
Cathedral/
├── README.md
├── wiki/          Design truth. wiki/design (the OS design chapters), wiki/architecture
│                  (this doc, the TCB map, ownership rules), wiki/decisions (numbered ADRs),
│                  wiki/speculation (parked, non-committed exploration).
│
├── tools/         Host-side, NEVER-SHIPS tooling — the SDK, the reference IDE, the debugger,
│                  the hostile simulator, CI gate runners, image assembly, the migration tester.
│                  If it runs on the developer's machine, not the target, it is here.
│
└── source/        THE OS — everything that ships or runs on the target machine.
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

Day one this is `README.md` + `wiki/` + `source/`; `tools/` and each `source/`
subdirectory appear only when real code lands in them.

---

## The dependency law

Dependencies flow **downward only**, among the `source/` subdirectories.
Enforced mechanically (see [Enforcement](#enforcement)), not by vigilance.

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

`wiki/` and `tools/` sit outside this graph — `wiki/` is prose, `tools/` is
host-side and ships nothing (it may *read* `source/contracts/` for
schema-awareness).

The load-bearing rules that make the kernel edge real:

- **Userspace never build-depends on `core/`.** Everyone targets the ABI in
  `contracts/`; `core/` *implements* those contracts. From the rest of the tree's
  build graph, `core/` is a leaf — depended on by nobody. This is what keeps the
  kernel swappable and the boundary crisp (the microkernel discipline: the
  contract is the coupling, not the implementation).
- **`boot/` depends on `contracts/`, never on `core/`'s internals.** It
  constructs the machine state the handoff contract promises; it does not reach
  into the kernel.
- **`contracts/` and `foundation/` are the two roots** — depend on nothing,
  depended on by everything. Keep them small and stable; churn here ripples
  everywhere.

---

## Placement decision-procedures

Phrased so a stranger applies them without discussion. *If behavior is X, it
belongs in Y* (all paths under `source/`):

- **Proved, in the TCB — memory-safety / scheduling / authority-minting core** →
  `core/`. If it isn't small enough to audit in full, it doesn't belong here.
- **A frozen contract other components target** (a wire schema, a capability
  type, the syscall surface, the boot handoff) → `contracts/`.
- **Shared by both `core/` and userspace, kernel-safe** → `foundation/`.
  (If only userspace uses it → `libraries/`.)
- **A long-running userspace program that serves an endpoint / owns a resource**
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
  does — a network-flow minter is `core/`-adjacent authority, not "the
  networking folder."
- *When genuinely unclear, it does not exist yet.* Don't invent a home for
  speculative code; the need will name the home.

---

## Where cross-cutting artifacts live

The rule: **artifacts live with their one owner; only the frozen platform ABI
centralizes.**

| Artifact | Home |
|---|---|
| Tier-1 wire schemas, capability vocabulary, syscall/handoff/manifest formats | `source/contracts/` |
| A service's own IPC protocol | co-located with that service (one-owner rule) |
| A program's capability manifest | co-located — it is part of the package |
| Proof certificates | beside the code they prove (core's proofs in `source/core/`) |
| Matrix presets (Default / Locked-down / Throwaway / Trusted) | co-located with the stock mediator in `source/services/` — they are shipped data, not a source layer |
| Migration machines (`Upgradable<Old,New,Ctx>`) | beside the schema they migrate |
| Whole-system image compositions (bootable/testable configurations) | a build concern under `tools/` (image assembly) — data, not source |
| Unit tests / canaries | beside the component (see [Testing](#testing)) |

There is no central `manifests/` or `schemas/` junk drawer, and no `worlds/`
directory — a "world" is a Matrix (a runtime realm), not a source concept. The
manifest is part of the component; the frozen contracts are the only thing that
earns a shared home.

---

## The TCB is distributed — and made honest

`core/` is the proved kernel, but the full must-trust set is larger: a handful
of userspace services *mint authority* ("the trusted core as origin-of-authority,
distributed across resource owners" — the network broker mints flows, the
compositor mints seats/surfaces, the storage/realm service mints realms and runs
unseal, the Warden holds the secure-element). These live in `source/services/`
for navigability (they are big userspace programs; forcing them into `core/`
would blur the kernel edge), but their trust status is **not hidden**:

- Each declares it in its manifest.
- [`tcb.md`](tcb.md) enumerates the exact audit surface: `core/` + `contracts/`
  + the named minting services. That document is the answer to "what must I
  trust," kept in one place rather than smeared across the tree.

This is the seL4 lesson (a small, fully-auditable trusted set) adapted to
Cathedral's reality (minting is distributed, not monolithic).

---

## Naming conventions (inherited from Omega)

- **No invented abbreviations.** `applications`, not `apps`; `arguments`, not
  `args`. Standard domain acronyms (MMU, IPC, IOMMU, UEFI) are fine.
- **Names carry the layering.** A package is named for what it owns; its
  directory path mirrors its name; file names inside describe the behavior
  implemented.
- **Entry/structure declarations stay thin.** A package's top-level file
  declares structure and re-exports; implementations live in behavior-named
  files, never in a catch-all.
- **A program and a library are the same kind of thing** (an Omega package;
  "runnable" = it exposes a `main` interface — see `developer_experience`). The
  `services/` vs `libraries/` vs `applications/` split is *role for navigation*,
  not a format distinction.

---

## The no-scaffolds rule

**A directory is an assertion that real code lives there.** We do not create
empty directories or placeholder packages to mirror this plan (Dolrus ADR-0001,
adopted). Consequences:

- This document holds the *full* intended tree with per-region status markers.
  The filesystem holds only what exists.
- A directory is created when its first real resident lands, and gets a one-page
  `CHARTER.md` at that moment (what belongs, what is explicitly out, its
  dependency rules).
- Planned-but-unbuilt structure is legible *here*, with honest status — never by
  perjuring the tree with empty folders.

Status legend: **built** · **in-progress** · **planned** · **parked**.
Current reality (2026-07-02): everything is **planned**. The first code will be
`source/boot/uefi/` + a minimal `source/core/` (the first-boot ladder in
`../../../Omega/wiki/cathedral_alignment.md`), and `source/contracts/` grows the
boot handoff alongside them.

---

## Testing

- **Unit tests co-locate with the code they test** (adjacent files inside the
  package). There is NO parallel `tests/` shadow tree — every surveyed OS that
  built one (Haiku, Serenity) watched it drift.
- **Canaries** follow Omega's structure: outcome-first, then area —
  `canaries/pass/<area>/<behavior>/` and `canaries/fail/<area>/<behavior>/` with
  an expected-diagnostic file. Named by behavior under test, never by the
  incident that motivated them.
- **Whole-system simulation** is a Matrix composition, not a source layer — a
  synthetic top-level Matrix hosting the components under test, assembled as an
  image under `tools/` and run in the hostile simulator.
- A **differential oracle** (as Omega runs one) is the standing mitigation for
  the parts that can't be proved.

---

## Enforcement

- **Machine-checked layering.** Port Omega's `omega-architecture-test`: read the
  package dependency graph, fail the build on any upward edge not in a
  `KNOWN_EXCEPTIONS` allowlist, and fail when a stale exception no longer matches
  a real edge (so the policy only ratchets tighter). The layer ranks are the
  order in [the dependency law](#the-dependency-law).
- **Charters as the local law.** Each `source/` subdirectory's `CHARTER.md`
  states its scope, non-goals, and dependency rules — cited when deciding where
  new code belongs.
- **Monorepo atomicity is the point.** A change to `core/` that breaks its proofs
  does not merge — the proofs update in the same commit. Only possible because
  everything is one repo; it is the single biggest structural advantage Cathedral
  holds over seL4 (proofs in a lagging sibling repo) and Redox (components across
  submodules). The layout exists to spend it.

---

## Deliberate omissions (negative space is part of the plan)

- **No generic OS-independent crates, no `crates.io`-style external dependency
  model.** Cathedral is all in-house, single-language (Omega). The rust-osdev
  model (generic `x86_64`/`uefi`/`bootloader` crates) is refused: it churns
  third-party unsafe code inside the TCB, its ambient-authority idioms
  (`Port::new(0x60)`) contradict the capability model, and it hands the boot
  contract to outsiders. Hardware *facts* are transcribed as data in
  `source/drivers/facts/`; hardware *drivers* are ours.
- **No third-party quarantine directory — because there is no third-party code.**
  If that ever changes, it enters through one named, policy-governed location,
  and the all-in-house invariant becomes a browsable boundary rather than a
  matter of vigilance. Until then, its absence is the claim.
- **No `session_logs/` or agent-runbook machinery.** Cathedral is human-driven;
  the autonomous-loop scaffolding that suits Dolrus would be the exact clutter
  this layout exists to avoid.
- **No `worlds/` directory.** A "world" is a runtime Matrix realm, not a source
  concept. Matrix presets are shipped data (with the mediator in `services/`);
  system images are a build concern (`tools/`).
- **Drivers are count-budgeted.** `drivers/` swallows every OS repo that lets
  breadth accrete ambiently (Linux's `drivers/` is two-thirds of the tree).
  Cathedral ships virtio/simulated devices plus one exemplary real driver per
  class first; broad hardware support is a deliberate later purchase, logged as
  such, never an ambient accretion.

---

## Open judgment calls (flagged, not blocking)

Decided as noted; revisit if reality argues otherwise.

1. **`foundation/` vs folding shared primitives into `libraries/`.** Kept
   separate — the kernel-safe-shared-code distinction is real (Serenity's AK
   proves the value) and a charter keeps it from sprawling.
2. **Contracts central vs co-located.** Hybrid: tier-1 frozen ABI centralizes in
   `contracts/`; per-service protocols co-locate. This is the one place the
   one-owner rule and the interfaces-are-first-class idea are balanced by hand.
