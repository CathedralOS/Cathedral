# 0001: Repository Layout

Status: **accepted** (2026-07-02); licensed-port policy clarified 2026-09-20
by `PORT-000` in [`TASKS_RUST_OSDEV_PORTS.md`](../../TASKS_RUST_OSDEV_PORTS.md).

## Context

Cathedral is entering implementation. Before the first line of runtime code, the
source tree needs a fixed shape. Otherwise directories accrete ad hoc, the
trust boundary becomes invisible in the layout, and the repository becomes the
"littered with incoherent folders" hellscape the project wants to avoid. Linux
is the failure case: `drivers/` dwarfs everything and no privilege edge is
scannable.

The governing requirement, stated by the repository owner: it must be stupidly
easy to navigate. A human scanning the folder structure should answer obvious
questions (where does the kernel end, what is a service vs a program vs boot
code) without opening a file.

Two reference repositories informed the decision. Dolrus (`C:\Projects\Dolrus`)
demonstrated the method: plan the whole tree in one prose document, iterate the
plan as its own phase before writing code, stress-test it against a mature
codebase, and never pre-create empty scaffolds. Omega (`../../../Omega/`),
Cathedral's implementation language, demonstrated the enforcement: directories
are the architecture, downward-only dependencies are checked by a build-failing
test, and each durable concept has exactly one owner. A survey of Linux,
Fuchsia, seL4, Redox, SerenityOS, Haiku, Theseus, MINIX 3, and Plan 9 supplied
the failure modes to avoid.

The full design and rationale live in
[`../architecture/repository_layout.md`](../architecture/repository_layout.md).
This ADR records the locked decisions.

## Decision

1. **Root separates the OS from everything about it.** The repository root holds
   `README.md`, `wiki/` (documentation truth), `tools/` (host-side, never ships), and
   `source/` (the OS). The OS is not sprayed across root; it lives under one
   roof. Root answers "what is this repository"; `source/` answers "what is the
   OS."

2. **`source/` is split by trust, descending.** Its subdirectories, in layer
   order, are: `contracts/` (the frozen ABI everyone targets), `core/` (the
   proved kernel, the TCB and nothing more), `foundation/` (the kernel-safe
   library shared by core and userspace), `services/`, `drivers/`, `libraries/`,
   `applications/`, `boot/`. The trust boundary is scannable: `core/` is the
   kernel edge; everything else is userspace or the firmware seam.

3. **Dependencies flow downward only, enforced mechanically.** The dependency
   law is recorded in the layout document and will be enforced by a port of
   Omega's `omega-architecture-test` (fail the build on any upward edge; a stale
   allowlisted exception also fails, so the policy only tightens). The
   load-bearing rule: userspace never build-depends on `core/`. It targets
   `contracts/`, which `core/` implements, so the kernel is a leaf in the build
   graph.

4. **Roles are for navigation; the manifest is the security fact.** A driver, a
   service, and an application are the same kind of thing: an Omega package
   holding capabilities. They are grouped separately for human legibility, not
   because any is blessed. Each directory's `CHARTER.md` states this.

5. **The TCB is distributed and enumerated in one place.** A few `services/`
   mint authority and are therefore trusted. That fact lives in their manifests
   and in [`../architecture/tcb.md`](../architecture/tcb.md), never in tree
   position. `core/` stays the small, fully-auditable proved kernel.

6. **No empty scaffolds.** Directories and packages are created only when real
   code lands in them. The full intended tree lives in the layout document with
   per-region status markers; the filesystem holds only what exists. The rule
   is adopted from Dolrus ADR-0001.

7. **Artifacts live with their one owner; only the frozen ABI centralizes.** No
   `manifests/`, `schemas/`, or `worlds/` junk drawers. A manifest is part of
   its package, a "world" is a runtime Matrix (not a source concept), and only
   the tier-1 contracts earn a shared home in `contracts/`.

8. **Single-language target implementation, monorepo ownership.** Cathedral
   maintains Omega target code in one repository without importing Rust crates
   or their ambient-authority APIs. Properly licensed derivative translations
   may preserve useful representations, algorithms, and tests. Primary-source
   facts, licensed derivatives, and clean Cathedral integration remain distinct
   provenance categories under the
   [porting policy](../architecture/prior_art_and_hardware_facts.md).
   Translations live in their eventual ownership layer, with co-located
   `PORT.md` and tests, upstream mapping, and retained notices indexed in
   `THIRD_PARTY_NOTICES.md`. Studied source checkouts remain gitignored in
   `reference_code/`; no Rust vendor tree is committed. Untypeable translations
   stay out of production build roots, and integration remains a separate
   authority/lifecycle review. A source translation is not proof of ABI or
   hardware behavior. Everything is one repository, so a change to `core/` and
   its proofs is one atomic commit. That is the structural advantage over
   seL4's and Redox's split repos, and the layout exists to spend it.

## Consequences

- The trust boundary is legible from the tree; `tcb.md` answers "what must I
  trust" in one document.
- New work has a decision procedure ("if behavior is X, it belongs in Y") and a
  machine-checked layering law, so placement is not re-litigated per change.
- Day one, the tree is `README.md` + `wiki/` + `source/`; `tools/` and each
  `source/` subdirectory appear only as real code arrives. The first residents
  will be `source/boot/uefi/` and a minimal `source/core/`, with
  `source/contracts/` growing the boot handoff alongside them (the first-boot
  ladder in `../../../Omega/wiki/cathedral_alignment.md`).
- Charters and the layering-enforcement test are follow-up work, created with
  their first subjects rather than up front.

## Follow-up

- Add each `source/` subdirectory's `CHARTER.md` when its first resident lands.
- Port `omega-architecture-test` to check Cathedral's package dependency graph
  once there is a graph to check.
- Keep `tcb.md` in sync: any chapter change that adds or removes a minter updates
  the map in the same commit.
- Record later structural choices as new ADRs only when they are not already
  implied by this decision and the layout document.
