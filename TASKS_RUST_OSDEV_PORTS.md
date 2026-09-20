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
  **BLOCKED native leg:** the [tested fixed-image model](source/libraries/uefi/table_images.PORT.md)
  compares all 376 bytes and exercises pure mocks; complete native table
  reflection and private callback field materialization remain Omega gaps.

Phase 1 evidence: the isolated [raw corpus](source/contracts/uefi/raw/PORT.md)
records status per slice. Scalar helper tests execute in Omega's semantic
evaluator with a failing assertion control. These checks do not establish
native execution, firmware behavior, or emitted foreign-layout agreement.
The [whole-crate audit](source/contracts/uefi/raw/CONFORMANCE.md) covers all 65
pinned raw source files and 4,194 expected measurements. Combined source and
semantic checks pass on a fresh build of Omega `eaa7993`; exact reflection,
projection, union/tail and native-boundary limitations remain recorded there.

## Phase 2 — small, high-yield hardware packages

- [x] **PCI-000 — Port `pci_types`.** Land PCI configuration headers, BDFs,
  command/status flags, BAR encodings, bridge headers, capability walking, and
  extended-capability facts.  Keep byte parsing pure; config-space access is an
  unresolved authority seam.  Add malformed-list, alignment, and overflow
  tests.
- [x] **UART-000 — Audit `uart_16550` against existing facts.** Do not duplicate
  `source/drivers/facts/uart_16550.omg`.  Add omissions and upstream tests, then
  map constructor/read/write behavior into pure plans plus explicit port-I/O
  boundaries.
- [x] **PIC-000 — Audit `pic8259` against existing facts and plans.** Reconcile
  offsets, masks, initialization order, EOI behavior, and existing Cathedral
  tests.  Preserve Cathedral's explicit `PortIo` authority model.
- [x] **X86-000 — Inventory `x86_64` against Cathedral.** Classify every module
  as already represented, generic fact/layout work, instruction boundary,
  policy-bearing Cathedral work, or deliberate rejection.
- [ ] **X86-001 — Port missing pure x86 representations.** Addresses, page-table
  encodings, descriptor tables, selectors, registers, flags, MSRs, and
  instruction operands.  Extend existing files rather than introduce parallel
  types.
- [ ] **X86-002 — Translate pure x86 algorithms and tests.** Canonical-address
  checks, index extraction, frame/page arithmetic, descriptor construction, and
  table walking.  Actual register access and instructions remain boundaries.

PCI-000 evidence: [PCI port record](source/libraries/pci/PORT.md), complete
six-file inventory, pinned in-memory Rust witnesses, and ten Omega semantic
behavior groups with ten body-mutating negative controls. Configuration access
remains an explicit authority seam.

UART-000 evidence: [UART audit](source/drivers/uart_16550/PORT.md), additive
existing facts, 168 pinned values/ordinals, 45 Rust tests, translated Omega
behavior and body-mutating controls. The existing fact canary passes; live
port-I/O/MMIO and polling remain explicit owner boundaries.

X86-001/002 partial evidence: [address arithmetic](source/libraries/x86_64/addresses.PORT.md)
passes 110 Omega numeric cases, 102 actual pinned Rust witnesses, finite stepping
relations and three body-mutating controls. The [register slice](source/drivers/facts/x86_registers.PORT.md)
adds 177 observed constants and 31 tested helpers, including checked STAR
underflow handling. The [descriptor slice](source/drivers/facts/x86_descriptors.PORT.md)
adds semantic descriptor cases, complete byte codecs, GDT append and bitmap
plans, four Rust tests and four body mutations; native imported-layout limits
are recorded. The full task checkboxes stay open for remaining instruction
and mapper surfaces. The [PTE codec slice](source/libraries/x86_64/page-entries.PORT.md)
reuses the canonical entry schema: 151 Omega scenarios, 143 actual Rust
witnesses and three body mutations pass. The modernized layout canary demands
all fourteen fields through the existing bit policy; native ABI is not measured.
The [detached table slice](source/libraries/x86_64/tables.PORT.md) adds full-array
operations and captured translation: 42 actual Rust mapper calls, 43 Omega
translation scenarios, six table/address fixtures and six body mutations pass.
Complete map/unmap/update routes are covered by the later slice below; live
ownership/invalidation integration remains open.
The [page/frame slice](source/libraries/x86_64/pages.PORT.md) passes 267 numeric
scenarios, 253 actual Rust witnesses, 15 upstream Rust tests, range/overflow
extras and three body mutations. Checked canonical-gap range deviations are explicit.
The [mapping-decision slice](source/libraries/x86_64/mapping-plans.PORT.md) passes
112 actual Rust child/leaf operations and matching Omega decisions, nine extra
assertions and four body mutations. Partial writes on failure are retained;
the later route slice completes this captured mapper orchestration; live
invalidation settlement remains separate.
The [interrupt slice](source/drivers/facts/x86_interrupts.PORT.md) passes 73 Rust
observations/target assertions, four Rust tests, Omega option/frame/gate checks,
complete 4096-byte table encode/decode/rejection fixtures and six body mutations.
Canonical gate fields and placements are preserved; imported generated-field
privacy remains an explicitly reproduced layout-consumer limitation.
The [TLB operand slice](source/libraries/x86_64/tlb-operands.PORT.md) passes
43 exact-source Rust observations, six actual upstream target assertions, four
Rust tests and Omega semantic checks with four body mutations. Pinned and
AMD-defined range-count recipes are separate; no TLB instruction or invalidation
settlement is claimed.
The [CPUID predicates](source/libraries/x86_64/instruction-observations.PORT.md)
complete the remaining RDRAND/SMAP observation helpers: 68 Rust/Omega bit cases
and two body mutations pass. These functions consume supplied register values.
The [cleanup branch slice](source/libraries/x86_64/cleanup-branch.PORT.md) passes
112 actual Rust/Omega singleton-range scenarios, five additional fixtures and
three body mutations. It retains non-present nonzero entries, checks all 512
entries for sibling occupancy and records deepest-first retirement requests.
The [cleanup range cursor](source/libraries/x86_64/cleanup-ranges.PORT.md) adds
bounded, resumable inclusive-range traversal: 13 whole-range Rust witnesses,
18 Omega steps, six additional fixtures and four body mutations pass, including
high-bit and maximum budgets. Exhaustion
retains the exact next page; actual custody, invalidation and reclamation remain
separate integration work.

The [complete route slice](source/libraries/x86_64/mapping-routes.PORT.md) passes
208 actual Rust/Omega scenarios, four additional fixture groups and four body
mutations, including partial edits, allocation failures and capture mismatches.
[Mapper conveniences](source/libraries/x86_64/mapper-conveniences.PORT.md) add
actual trait-default witnesses, three Omega fixtures and four body mutations.
The [remaining pure-work audit](source/libraries/x86_64/remaining-pure.RECONCILIATION.md)
identifies unfinished recursive mapper algorithms; these are implementation work. The later register slices complete the
remaining operand recipes and raw word transport.
[Owned GDT storage](source/libraries/x86_64/gdt-storage.PORT.md) now passes 16
actual Rust/Omega scenarios, malformed-input/reset checks and four body mutations,
including full 8192-word import and failed system appends without partial writes.
[Register operand recipes](source/libraries/x86_64/register-operands.PORT.md) add
1,323 exact pure-body Rust observations, 14 Omega batches, four rejection cases
and seven body mutations, covering reserved bits, XCR0 validation, CR3/CR8 and
STAR/CET/APIC composition. [MSR word transport](source/libraries/x86_64/msr-words.PORT.md)
adds 136 Rust-derived observations, 130 Omega round trips and two body mutations.
No live register methods or instructions execute.
[Numeric mapper topology](source/libraries/x86_64/mapper-topology.PORT.md) adds
3,131 Rust numeric witnesses, 3,139 Omega assertions in 22 fixtures and 22 body
controls for offset addition, recursive coordinates and constructor observations.
All 512 recursive indices are covered; pointer/custody operations stay external.
[Explicit memory-encryption state](source/libraries/x86_64/memory-encryption.PORT.md)
passes 133 Omega profiles against 1,315 actual Rust observations, an additional
reconfiguration regression and four body mutations. Repeated-mask accumulation
and both bit polarities are preserved; existing captured walkers remain explicitly
on their default physical profile.

X86-000 evidence: [full source reconciliation](source/libraries/x86_64/PORT.md)
classifies all 41 Rust files, 1,307 lexical anchors, 124 supplemental anchors
and 85 test/proof scenarios. Pending rows are reviewed implementation work;
confirmed instruction-catalog gaps are limited to their exact live operations.

## Phase 3 — VirtIO protocol corpus

- [x] **VIRTIO-000 — Port transport-neutral specification types.** Device IDs,
  status bits, feature negotiation, common configuration, notification, ISR,
  and device-specific configuration structures.
- [x] **VIRTIO-001 — Port split virtqueue representations.** Descriptor,
  available, and used rings; event-index arithmetic; validation; wraparound;
  and upstream tests.  DMA ownership is deliberately absent.
- [x] **VIRTIO-002 — Port packed virtqueue representations.** Descriptor/event
  structures, wrap counters, notification data, validation, and tests.
- [x] **VIRTIO-003 — Port transport shapes.** PCI capability and MMIO transport
  representations.  Leave discovery, MMIO access, DMA grants, interrupts, and
  queue activation behind named seams.
- [x] **VIRTIO-004 — Add a pure queue simulator.** Exercise negotiation and ring
  transitions without hardware or DMA.  This is the unit-test target for later
  driver integration.

VIRTIO-000/001 evidence: [protocol and split-queue port](source/libraries/virtio/PORT.md)
retains 343 expected vectors, 255 actual Rust UEFI-target measurements, 20 upstream
allocation cases, and eight Omega behavior groups with eight body-mutating
controls. All 24 policy declarations check separately; combined fixture/layout
import exposes a recorded compiler diagnostic. No DMA or native ABI claim.

VIRTIO-002 evidence: [packed queues](source/libraries/virtio/packed.PORT.md)
passes five semantic groups/five body-mutating controls, actual Rust bitfield
checks and 14 Rust UEFI-target layout measurements. Four policy declarations
check independently; wrap/event predicates confer no DMA or ordering authority.

VIRTIO-003 evidence: [transport shapes](source/libraries/virtio/transport.PORT.md)
retains PCI capabilities, all 30 MMIO and 23 PCI common field descriptions,
and inert ordered access plans. Seven semantic groups and seven body mutations
pass; 28 Rust target geometry measurements agree. Five layout declarations
check independently. Discovery, access and activation remain named owner seams.

VIRTIO-004 evidence: [pure queue simulator](source/libraries/virtio/simulator.PORT.md)
passes six lifecycle groups and six body-mutating controls for negotiation,
split/packed submission, completion/reuse, wrap and event suppression. Its
eight-slot direct-buffer profile is explicit; asynchronous DMA is not modeled.

## Phase 4 — ACPI tables, then AML

- [x] **ACPI-000 — Inventory and partition `acpi`.** Separate byte/table facts,
  pure table discovery/parsing, platform-topology results, AML parsing, AML
  execution, handler callbacks, and allocator/concurrency assumptions.
- [x] **ACPI-001 — Port table headers and checksums.** RSDP, RSDT/XSDT, SDT
  headers, signatures, lengths, revisions, checksums, and strict bounded-input
  validation.
- [x] **ACPI-002 — Port fixed table parsers.** At minimum FADT, MADT, MCFG, HPET,
  and the tables consumed by the first Cathedral hardware-discovery path.  Add
  valid and malformed byte fixtures.
- [x] **ACPI-003 — Port topology extraction.** Produce inert typed descriptions
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
- [x] **ACPI-007 — Define the later Cathedral adapter.** Specify—but do not yet
  integrate—the attenuation from discovered regions to separately granted
  physical/MMIO/I/O capabilities.

ACPI-000 evidence: [partitioned inventory](source/libraries/acpi/PORT.md)
classifies 52 Rust files, 1,581 anchors, 66 Rust tests and 19 ASL/AML assets.
External fixture provenance remains explicit; no firmware dumps or externally
derived test bodies were copied. Parsing and interpretation remain implementation
work, separate from this completed inventory.

ACPI-001 evidence: [bounded header parser](source/libraries/acpi/headers.PORT.md)
checks RSDP revisions 0/2 and SDT/root entries in a 4096-byte input profile,
with strict lengths and checksums and raw OEM bytes. All 65 semantic scenarios
and three body-mutating controls pass; 88 wire vectors and 65 pinned signatures
are audited. Larger inputs, native layouts and firmware mapping are outside this slice.

ACPI-002 evidence: [fixed parsers](source/libraries/acpi/fixed.PORT.md) cover GAS,
FADT, all 17 pinned MADT entry kinds, MCFG and HPET. All 291 Omega scenarios and
three body mutations pass; 236 compiled Rust declaration layout facts and 30
pinned flag getter observations provide separate reference evidence.

ACPI-003 evidence: [topology extraction](source/libraries/acpi/topology.PORT.md)
passes 77 Omega scenarios and three body mutations. Ordered typed CPU/controller
facts, timer descriptions and checked PCI-region queries retain unknown entries
and separate observed boot identity from table order. ECAM uses bus-0-relative
addressing. NUMA extraction, native execution and hardware activation remain open.

ACPI-007 evidence: [adapter contract](source/libraries/acpi/ADAPTER.md) specifies
discovery snapshots, named source custody, device-scoped requests, conserved
attenuation, complete transfer/page footprints, mediated versus placed lifetimes,
and teardown/rejection obligations. This is a specification milestone; no grants,
provider integration or adapter execution tests are claimed.

ACPI-005 partial evidence: [integer/byte helpers](source/libraries/acpi/interpreter/PORT.md)
pass 150 Omega scenarios and three body controls, including five translated
upstream object-test scenarios. Width-aware arithmetic, BCD, bit copying and
bounded conversions preserve explicit primary-spec corrections. Method/context,
namespace/target and generic object operations were outside that helper slice.
The [integer method executor](source/libraries/acpi/interpreter/execution/PORT.md)
now passes 79 actual Omega checked-interpreter cases and 79 changed-body controls.
It executes integer method bytes, nested calls, existing named/alias targets and
bounded If/Else/While/Break/Continue with shared fuel. Current frame-admission
constant proof and its mutation control also pass; full bytecode evidence is the
distinct checked-interpreter stage. Definition observations preserve original
scope across aliases/rebinding. Automatic loader capture, generic values, fields,
packages, dynamic declarations and multi-unit source management remain pending.
Unresolved region/synchronization/service results grant no live access. ACPI-005/006
stay open.

ACPI-004 partial evidence: [static AML syntax and namespace](source/libraries/acpi/aml/PORT.md)
passes 27 Omega scenarios with 27 body mutations and an 18-file source check.
It covers NameString/package framing, literal/package values, static declarations,
retained method bodies, stable aliases, scoped lazy references and transactional
loader errors under explicit capacities/budgets.
[Field declaration metadata](source/libraries/acpi/aml/fields/PORT.md) adds 18
semantic cases, 18 body mutations and a 19-file source check for Field, IndexField,
BankField and all five FieldList forms. Dynamic BankValue/BufferSize bodies remain
explicitly unparsed or opaque; field namespace installation and full interpreter
behavior remain pending. The separate integer executor does not complete these
syntax/resource-limit milestones. ACPI-004/006 stay open.

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
