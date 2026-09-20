# Prior Art & Hardware Facts — how Cathedral learns from existing OS code

> **Status: porting policy (2026-09-20).** Implements the ownership rules in
> [`repository_layout.md`](repository_layout.md) and the licensed translation
> lane in [`TASKS_RUST_OSDEV_PORTS.md`](../../TASKS_RUST_OSDEV_PORTS.md).
> The method applies to studied OS and driver code; permission to translate is
> established separately for each pinned source and its license.

## Three kinds of work

Cathedral's target implementation is Omega, maintained in this monorepo.
We do not link Rust crates or import their ambient-authority APIs. We do allow
properly licensed derivative Omega translations, including algorithms, data
organization, tests, and quirk knowledge. A language rewrite does not erase
upstream provenance.

Keep these three kinds of work visible:

1. **Primary-source facts.** Register offsets, bit layouts, table formats, and
   ABI values should cite the governing specification edition and section.
   Upstream code is an index and cross-check. Record disagreements; do not
   silently choose a crate's value over the specification. Specification facts
   do not justify copying surrounding prose or code without its provenance.
2. **Licensed derivative translations.** Preserve useful upstream structure,
   algorithms, tests, and comments under the applicable license. Record the
   exact source mapping, modifications, and notices. Substantial translations
   from the queued rust-osdev projects preserve `MIT OR Apache-2.0` unless the
   repository owner chooses a different compatible license. Identify each
   modified file and retain applicable upstream notices.
3. **Clean Cathedral integration.** Separately authored adapters select
   lifecycle policy, attenuate authority, and connect the translated units to
   Cathedral. Keep policy out of the generic port. Any copied integration code
   remains derivative and must retain its provenance too; calling it an adapter
   is not a change of origin.

## `reference_code/` — pinned source, gitignored

Clone studied sources into the gitignored root `reference_code/` reading room:

```
reference_code/
├── rust-osdev/         # uefi-rs, x86_64, uart_16550, acpi, virtio-spec-rs, …
├── edk2/               # optional firmware implementation reference
└── …                   # other sources studied for a named subsystem
```

Do not commit a Rust vendor tree or link it into Cathedral. Host-side inventory
checks and upstream fixture generation may read or build a pinned checkout;
record those commands and distinguish their results from Omega verification.
An absent optional checkout must make such a check fail clearly, never pass.
Committed Omega translations live in their eventual ownership layer alongside
`PORT.md`, source/symbol manifests, and translated tests or inert vectors.
`reference_code/` being ignored does not mean all committed code is original.

## The translation workflow

1. **Fix the scope and origin.** Use the queue's exact upstream pin. Verify its
   license files and notices, record them in `THIRD_PARTY_NOTICES.md`, and start
   a co-located `PORT.md` from the [template](../../tools/ports/PORT.template.md).
   Map every upstream file and public symbol in the claimed slice to a
   translation, a deliberate omission with reason, or a named blocker. A pin
   update is a separate reviewed change with a renewed inventory/license audit.
2. **Read the contracts first.** Consult the destination charter and relevant
   primary specs, then Omega's current normative contracts and implementation
   boundary. Layout geometry describes bits; access policy and authority are
   separate obligations. In particular, consult Omega's
   [layout plans](../../../Omega/wiki/spec/layouts/plans.md),
   [authority](../../../Omega/wiki/spec/resources/authority.md), and
   [device custody](../../../Omega/wiki/spec/resources/device_access.md).
3. **Translate the inert part.** Facts, parsing, validation, encoding, and pure
   state transitions may land ahead of compiler support. Raw UEFI ABI belongs
   in `source/contracts/uefi/`; driver facts in `source/drivers/facts/`; behavior
   follows the usual layer and charter rules. Extend existing representations
   instead of creating competing versions. Layout-sensitive work includes
   size, alignment, offset, discriminant, flag, and GUID/status vectors where
   applicable.
4. **Name unresolved seams.** Treat Rust `unsafe` as an invariant to review,
   not syntax to copy. Prove the invariant in ordinary Omega or record a seam
   with `PORT-BLOCKED[omega:<short-name>]: <required behavior>`. Record the exact
   required language behavior, specification/implementation evidence, affected
   symbols/tests, and a minimal reproducer when possible. Only an actual
   language implementation gap or unsettled design blocks a task; unfinished
   implementation, engineering effort, and missing local setup are work to do.
   Do not invent semantics or modify Omega as part of this queue.
5. **Preserve tests and report evidence.** Translate upstream unit tests beside
   the behavior, including boundary, malformed-input, and overflow cases. If a
   test cannot run, preserve its fixture/vector and label it unexecuted with the
   exact blocker. Source review, inventory checks, transcription, typechecking,
   test execution, layout comparison, hardware execution, and Cathedral
   integration are separate claims. A host-side vector check does not prove an
   Omega ABI, and an upstream Rust test is not an Omega test.
6. **Integrate separately.** Untypeable packages stay out of production build
   roots. Even compilable raw contracts and pure behavior do not confer live
   authority. Integration must establish Cathedral's grants and lifecycle
   contracts, run affected canaries, and update the port's evidence. Record a
   failed or unavailable check honestly; never use absence from a build as
   compiler coverage.

`PORT.md` reports one current stage: `inventoried`, `transcribed`, `typechecked`,
`tested`, or `integrated`. That stage applies only to its precisely named slice;
record mixed per-file stages and outstanding work rather than promoting an
entire package on the strength of one passing unit. The template defines the
minimum evidence for each stage. Update the task checkbox and port record in
the same commit as the work.

## Authority is never a transcription result

Parsing an address, declaring a service slot, reproducing a register map, or
validating a byte sequence never grants port I/O, MMIO, physical-memory
ownership, firmware-service authority, DMA custody, or instruction execution.
Do not hide raw access inside a harmless-looking helper. Keep pure algorithms
separate from authority-bearing calls and leave unresolved calls behind named
boundaries. Layout compatibility is neither a grant nor external correspondence.

Quirk notes should name their source and the invariant they protect. A firmware
implementation such as OVMF is a host-harness dependency under `tools/`, with
its own provenance; it does not become part of the target implementation merely
because Cathedral boots under it.

---

## The rust-osdev inventory, sorted by the work it implies

| Crate | Kind | What we do | Lands in |
|---|---|---|---|
| **uefi-rs** | ABI + logic | Transcribe EFI struct layouts (SystemTable, BootServices, ConOut, memory descriptor, GUIDs, status codes, protocol vtables); read the memory-map/ExitBootServices dance for quirks. **Milestone-1 goldmine.** | `contracts/` + facts |
| **x86_64** | facts + logic | Transcribe page-table / GDT / IDT / CR / RFLAGS / MSR bit layouts (→ `Bits` stated-plans). Translate pure setup planning; instruction execution remains a boundary. | `drivers/facts/`, `core/` |
| **uart_16550** | facts | Serial register map — trivial transcribe. | `drivers/facts/` |
| **pic8259** / **apic** | facts + ritual | Port offsets + init sequences (remap-the-PIC, LAPIC setup). | `drivers/facts/`, `core/` |
| **pci_types** | facts | PCI config-space header, BAR formats, capability lists. | `contracts/`/facts |
| **virtio-spec-rs** | facts + protocol | Virtqueue descriptor/ring layouts + queue protocol — the first real driver's reference. | `drivers/` |
| **acpi** (+ AML) | facts + big logic | ACPI table parsing (facts) + the AML interpreter — licensed translation with explicit operation-region boundaries. | `services/` (confined interp) |
| **xhci** / **usb** / **vga** / **ps2-mouse** | driver logic | One exemplary port per class, later, on the count-budget. | `drivers/` |
| **bootloader** | boot logic | Reference for the real-mode→long-mode transition we skip by going UEFI-first — read to understand what UEFI does for us. | (study only) |
| **multiboot2** / **pvh** / **ieee1275** | alt boot ABIs | Deferred — multiboot2 only for a coreboot-payload reference platform; pvh only for a cloud target. | (deferred) |
| **linked-list-allocator** | primitive | Reject the ambient model — `Extent` owns backing-range authority and allocation strategies are ordinary packages over qualified extents. Reference the free-list algorithm only. | (subsumed) |
| **volatile** | primitive | Reject the wrapper — placed views derive sealed field operations from `Extent + LayoutPlan + AccessPlan`; volatile is an observation contract, not a type qualifier. | (subsumed) |
| **spinning_top** / **mem-barrier** | primitive | Subsumed by the concurrency model and checked instruction catalog: atomics/waits are ordinary contracted operations; fences/cache/TLB instructions emit complete target contracts. | (subsumed) |
| **ucs2-rs** / **endian-num** | util | UEFI strings are `u16` arrays; endianness is the layout/format machinery. Trivial/subsumed. | (subsumed) |
| **ovmf-prebuilt** | tooling | Use directly — it *is* the test firmware (OVMF.fd) we boot under QEMU. | `tools/` |
| **bootimage** / **cargo-xbuild** | tooling | Cargo-bound, irrelevant to the `build.omg` toolchain. Reuse only the workflow idea (assemble FAT image → `\EFI\BOOT\BOOTX64.EFI` → OVMF). | (idea only) |

---

## Mapped to the first-boot ladder

- **Milestone 1 (UEFI hello):** uefi-rs EFI ABI shapes → `contracts/uefi/`;
  `ovmf-prebuilt` → `tools/` harness.
- **Milestone 3 (alive after firmware):** x86_64 / uart_16550 / pic8259 / apic
  facts.
- **Later:** acpi + AML (the flagged gap), virtio, pci, then one exemplary
  driver per class.
