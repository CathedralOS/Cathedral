# rust-osdev porting queue

> **Status:** ready for agent pickup.  This queue deliberately separates
> mechanical translation from Cathedral integration.  A translated unit may
> land before Omega can compile it, but its status must say exactly what has
> and has not been verified.

## Goal

Build a broad Omega corpus from the permissively licensed
[`rust-osdev`](https://github.com/rust-osdev) ecosystem while the language and
compiler mature.  Preserve useful ABI shapes, hardware facts, pure algorithms,
quirk knowledge, and tests now; defer live authority, device access, and boot
integration until the required Omega mechanisms exist.

This is not a request to reproduce Rust's `unsafe`, pointer, volatile, locking,
or ambient-I/O APIs.  Those become explicit Omega boundary and authority seams.
The code on either side of such a seam can still be translated and tested.

## Pinned starting snapshots

These pins make independent agents translate the same source.  Updating a pin
is a separate reviewed task, never an incidental part of a port.

| Project | Upstream revision |
|---|---|
| [`uefi-rs`](https://github.com/rust-osdev/uefi-rs) | `c0facddf9ba42b74906a37fca2869e6cdbc8da6a` |
| [`x86_64`](https://github.com/rust-osdev/x86_64) | `cc35c876d3badb57df54a66e22f7768a52be95f2` |
| [`acpi`](https://github.com/rust-osdev/acpi) | `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5` |
| [`pci_types`](https://github.com/rust-osdev/pci_types) | `eff856b6bab81adbe1bda0ab750009f3786cb8d3` |
| [`virtio-spec-rs`](https://github.com/rust-osdev/virtio-spec-rs) | `ad565bd701e93fa47bc288d45701e1ae53ac3c12` |
| [`uart_16550`](https://github.com/rust-osdev/uart_16550) | `653455f17e92c67a67e44b4c4b6eaba22411e57b` |
| [`pic8259`](https://github.com/rust-osdev/pic8259) | `136052bcbf081b9382fc3d72ba04b83b30f664b4` |

The projects above offer MIT or Apache-2.0 licensing.  The setup task below must
verify that fact at each pinned revision and retain the applicable notices;
this table is not a substitute for the license files.

## Rules for every port

1. Clone studied Rust into the gitignored `reference_code/` reading room.  Do
   not commit a Rust vendor tree.
2. Record the upstream URL, exact revision, upstream paths, license, source-file
   mapping, and deviations in a `PORT.md` beside the translated package.
3. Preserve `MIT OR Apache-2.0` for substantially translated code unless the
   repository owner makes a different compatible choice.  Retain notices and
   identify modified files.
4. Put translated code in its eventual Cathedral ownership layer, but do not
   add an untypeable package to a production build root.  `PORT.md` must report
   its current state as one of:
   `inventoried`, `transcribed`, `typechecked`, `tested`, or `integrated`.
5. Translate upstream unit tests alongside behavior.  When Omega cannot yet
   compile a test, retain the test case as a clearly labelled fixture or vector
   and record the exact compiler blocker.  Never mark it as passing.
6. Use comments of the form
   `PORT-BLOCKED[omega:<short-name>]: <required behavior>` at an unresolved
   seam.  Do not guess language semantics merely to make a translation look
   complete.
7. Pure parsing, validation, layout, encoding, and state-transition logic must
   remain separate from authority-bearing calls.  Parsing bytes never grants
   port I/O, MMIO, physical-memory ownership, firmware-service authority, or
   permission to execute an instruction.
8. Treat upstream `unsafe` as a review marker, not something to transliterate.
   State the invariant it relied on, then either prove it in ordinary Omega or
   leave an explicit boundary seam.
9. Prefer primary specifications for numeric hardware and ABI facts.  Cite the
   relevant specification section and use upstream as a cross-check.  Preserve
   upstream provenance for copied organization, algorithms, tests, and prose.
10. Do not fold Cathedral policy into a generic port.  Cathedral adapters and
    authority attenuation are later, separately reviewed integration work.

## Common definition of done for a translation slice

- Every upstream file in the claimed slice is mapped to a translated file,
  deliberately omitted with a reason, or recorded as blocked.
- Public constants, representations, and operations have an inventory check;
  silently missing symbols fail the slice.
- Layout-sensitive types have expected size, alignment, field-offset, enum,
  flag, and GUID/status vectors as applicable.
- Pure algorithms carry normal, boundary, malformed-input, and overflow tests
  derived from upstream tests and the governing specification.
- The port does not manufacture authority or hide raw access behind a harmless-
  looking helper.
- `PORT.md` distinguishes source review, transcription, compilation, testing,
  hardware execution, and Cathedral integration.
- Existing affected Cathedral canaries still pass.  If the new source is not
  yet in a build, say so explicitly instead of claiming compiler coverage.
- The task checkbox and port status are updated in the same commit as the work.

## Phase 0 — establish the licensed-port lane

- [x] **PORT-000 — Align the repository policy.** Update
  `wiki/architecture/prior_art_and_hardware_facts.md`,
  `wiki/architecture/repository_layout.md`, and ADR 0001 where they currently
  say committed third-party-derived ports do not exist.  Preserve the useful
  split between primary-source facts, licensed derivative translations, and
  clean Cathedral integration code.
- [x] **PORT-001 — Add licensing records.** Add `THIRD_PARTY_NOTICES.md` and the
  necessary MIT and Apache-2.0 license texts.  List every pinned project and its
  chosen/preserved licensing.  Document how a later pin update is audited.
- [x] **PORT-002 — Add the `PORT.md` template.** It must contain upstream pin,
  license, source map, status, translated tests, Omega blockers, deliberate
  deviations, integration status, and verification commands.
- [x] **PORT-003 — Add an inventory checker.** Provide host-side tooling that
  compares a checked-in symbol/source manifest with the claimed upstream slice.
  It may inspect `reference_code/`, but must fail clearly when that optional
  checkout is absent rather than silently passing.
- [x] **PORT-004 — Add layout-vector plumbing.** Define a checked-in,
  deterministic format for upstream size/alignment/offset/value vectors and a
  way to compare those vectors with Omega inspection output once supported.
  Landing vectors before their Omega consumer is acceptable; label that state.

Phase 0 evidence: [port tooling and commands](tools/ports/README.md),
[retained licenses](THIRD_PARTY_NOTICES.md), and the checked header source/vector
fixtures. Host inventory and comparator tests pass; expected-vector validation
is not Omega ABI comparison. The current Omega observation adapter remains
unimplemented as explicitly permitted by PORT-004.

## Phase 1 — complete the raw UEFI contract

The immediate target is the `uefi-raw` crate within `uefi-rs`.  Raw UEFI ABI
belongs under `source/contracts/uefi/`.  Higher-level safe wrappers and
Cathedral's authority-bearing boot policy do not belong in this phase.

- [x] **UEFI-000 — Reconcile the existing subset.** Map current
  `uefi.omg` and `boot_services.omg` declarations to the pinned `uefi-raw`
  source and the UEFI specification.  Record collisions, missing fields, local
  policy, and ABI deviations before adding declarations.
- [x] **UEFI-001 — Port scalar foundations.** Status values, GUIDs, handles,
  revisions, time, capsules, memory types/attributes, common enums, and base
  aliases.  Add exact-value and representation vectors.
- [x] **UEFI-002 — Port table foundations.** Table header, system table,
  configuration table entries, and the complete boot-services and runtime-
  services table shapes in specification order.  Include function-slot and
  field-offset vectors; calling a slot remains a boundary concern.
- [x] **UEFI-003 — Port console and image-loading protocols.** Text input/output,
  serial I/O, loaded image, device path, load-file, and shell parameter shapes.
- [x] **UEFI-004 — Port storage protocols.** Simple file system, file, block I/O,
  disk I/O, ATA, SCSI, NVMe, firmware volume/storage, and firmware-management
  raw representations.
- [x] **UEFI-005 — Port display, bus, and machine protocols.** Graphics output,
  PCI/root bridge, USB, IOMMU, RNG, ACPI, memory-protection, and miscellaneous
  protocol shapes and GUIDs.
- [x] **UEFI-006 — Port network protocols.** SNP, PXE, DHCPv4, IPv4/config,
  TCPv4, HTTP, and TLS representations.  This is ABI transcription only, not a
  Cathedral network stack.
- [x] **UEFI-007 — Port HII protocols.** HII database, forms/IFR, strings,
  fonts, images, popup, browser, and configuration representations.  Split this
  into multiple commits if needed, retaining a complete source map.
- [x] **UEFI-008 — Port measured-boot protocols.** TCG v1/v2 raw structures,
  event/log shapes, constants, and GUIDs.  Do not interpret their presence as
  trusted boot or mint Cathedral attestation authority.
- [x] **UEFI-009 — Complete raw-contract conformance.** Close every mapped
  `uefi-raw/src` omission, run all available compile/layout/value checks, and
  produce an explicit list of tests still blocked by Omega.
- [ ] **UEFI-010 — Add producer/consumer fixtures.** Build inert table images
  and mock service functions usable by both an Omega-authored UEFI producer and
  a Cathedral consumer.  Tests must demonstrate identical raw layouts; they do
  not grant service authority or perform `ExitBootServices` integration.

Phase 1 evidence: the isolated [raw corpus](source/contracts/uefi/raw/PORT.md)
records status per slice. Scalar helper tests execute in Omega's semantic
evaluator with a failing assertion control. These checks do not establish
native execution, firmware behavior, or emitted foreign-layout agreement.
The [whole-crate audit](source/contracts/uefi/raw/CONFORMANCE.md) covers all 65
pinned raw source files and 4,194 expected measurements. Combined source and
semantic checks pass on a fresh build of Omega `eaa7993`; exact reflection,
projection, union/tail and native-boundary limitations remain recorded there.

## Phase 2 — small, high-yield hardware packages

- [ ] **PCI-000 — Port `pci_types`.** Land PCI configuration headers, BDFs,
  command/status flags, BAR encodings, bridge headers, capability walking, and
  extended-capability facts.  Keep byte parsing pure; config-space access is an
  unresolved authority seam.  Add malformed-list, alignment, and overflow
  tests.
- [ ] **UART-000 — Audit `uart_16550` against existing facts.** Do not duplicate
  `source/drivers/facts/uart_16550.omg`.  Add omissions and upstream tests, then
  map constructor/read/write behavior into pure plans plus explicit port-I/O
  boundaries.
- [x] **PIC-000 — Audit `pic8259` against existing facts and plans.** Reconcile
  offsets, masks, initialization order, EOI behavior, and existing Cathedral
  tests.  Preserve Cathedral's explicit `PortIo` authority model.
- [ ] **X86-000 — Inventory `x86_64` against Cathedral.** Classify every module
  as already represented, generic fact/layout work, instruction boundary,
  policy-bearing Cathedral work, or deliberate rejection.
- [ ] **X86-001 — Port missing pure x86 representations.** Addresses, page-table
  encodings, descriptor tables, selectors, registers, flags, MSRs, and
  instruction operands.  Extend existing files rather than introduce parallel
  types.
- [ ] **X86-002 — Translate pure x86 algorithms and tests.** Canonical-address
  checks, index extraction, frame/page arithmetic, descriptor construction, and
  table walking.  Actual register access and instructions remain boundaries.

## Phase 3 — VirtIO protocol corpus

- [ ] **VIRTIO-000 — Port transport-neutral specification types.** Device IDs,
  status bits, feature negotiation, common configuration, notification, ISR,
  and device-specific configuration structures.
- [ ] **VIRTIO-001 — Port split virtqueue representations.** Descriptor,
  available, and used rings; event-index arithmetic; validation; wraparound;
  and upstream tests.  DMA ownership is deliberately absent.
- [ ] **VIRTIO-002 — Port packed virtqueue representations.** Descriptor/event
  structures, wrap counters, notification data, validation, and tests.
- [ ] **VIRTIO-003 — Port transport shapes.** PCI capability and MMIO transport
  representations.  Leave discovery, MMIO access, DMA grants, interrupts, and
  queue activation behind named seams.
- [ ] **VIRTIO-004 — Add a pure queue simulator.** Exercise negotiation and ring
  transitions without hardware or DMA.  This is the unit-test target for later
  driver integration.

## Phase 4 — ACPI tables, then AML

- [ ] **ACPI-000 — Inventory and partition `acpi`.** Separate byte/table facts,
  pure table discovery/parsing, platform-topology results, AML parsing, AML
  execution, handler callbacks, and allocator/concurrency assumptions.
- [ ] **ACPI-001 — Port table headers and checksums.** RSDP, RSDT/XSDT, SDT
  headers, signatures, lengths, revisions, checksums, and strict bounded-input
  validation.
- [ ] **ACPI-002 — Port fixed table parsers.** At minimum FADT, MADT, MCFG, HPET,
  and the tables consumed by the first Cathedral hardware-discovery path.  Add
  valid and malformed byte fixtures.
- [ ] **ACPI-003 — Port topology extraction.** Produce inert typed descriptions
  of CPUs, interrupt controllers, timers, and PCI configuration regions.
  Discovery conveys facts, not MMIO or interrupt authority.
- [ ] **ACPI-004 — Port AML syntax and namespace construction.** Keep parsing and
  namespace building deterministic and bounded.  Do not attach live operation-
  region handlers yet.
- [ ] **ACPI-005 — Port the AML interpreter core.** Arithmetic, values, methods,
  control flow, packages, fields, and upstream semantic tests.  Represent
  operation-region access as an explicit unresolved service boundary.
- [ ] **ACPI-006 — Add AML resource limits.** Bound input, namespace growth,
  recursion, method work, and returned data.  Fail closed on unsupported or
  exhausted behavior.
- [ ] **ACPI-007 — Define the later Cathedral adapter.** Specify—but do not yet
  integrate—the attenuation from discovered regions to separately granted
  physical/MMIO/I/O capabilities.

## Phase 5 — only after the corpus above is healthy

- [ ] Port selected pure pieces and tests from APIC, xHCI, USB, PS/2, VGA, and
  comparable rust-osdev projects, one independently reviewable package at a
  time.
- [ ] Port higher-level `uefi` conveniences only where they survive Omega's
  ownership and authority model.  Do not preserve Rust ergonomics as an API
  requirement.
- [ ] Build an Omega-authored UEFI implementation against the same raw contracts
  and producer fixtures used by the Cathedral consumer.
- [ ] Run the same Cathedral EFI image and conformance suite against OVMF and
  the Omega-authored UEFI implementation.
- [ ] Integrate PCI discovery, ACPI topology, VirtIO, and real device access only
  through separately reviewed Cathedral grants and lifecycle policy.

## Explicit non-targets

- Do not port `bootloader` wholesale.  Cathedral is UEFI-first, and that crate's
  Rust-kernel boot contract is not Cathedral's contract.  Extract individual
  tests or algorithms only when a named Cathedral task needs them.
- Do not port `volatile`, allocators, spin locks, or memory barriers as ambient
  primitives.  They are replaced by Omega layout/access plans, owned extents,
  concurrency semantics, and checked instruction boundaries.
- Do not claim that source transcription proves ABI correctness, that unit tests
  prove hardware behavior, or that either one completes Cathedral integration.
- Do not modify Omega merely to make a port compile as part of this queue.
  Record a minimal, general Omega blocker for the language team instead.

## Suggested first handoff

Give one agent **PORT-000 through PORT-004**, then assign **UEFI-000 through
UEFI-002** in small commits.  That establishes the rules and produces the most
valuable shared contract before parallelizing the remaining independent UEFI
protocol groups.
