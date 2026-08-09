# Prior Art & Hardware Facts — how Cathedral learns from existing OS code

> **Status: DRAFT (2026-07-02).** Operationalizes the stance in
> [`repository_layout.md`](repository_layout.md) ("hardware *facts* are
> transcribed as data; hardware *drivers* are ours") into a concrete method.
> The reference case is the rust-osdev crate ecosystem, but the method is
> general: it applies to any existing OS or driver codebase we study
> (Linux, seL4, SerenityOS, a vendor datasheet's example code).

---

## The governing idea

Cathedral is all-in-house and single-language, so **we link nothing external**
(Omega cannot consume Rust crates, and their ambient-authority idioms —
`Port::new(0x60)`, `static mut`, raw `&mut` to MMIO — contradict the capability
model). But refusing to *depend* on prior art is not refusing to *learn* from
it. The value of a mature codebase like rust-osdev's is three things, in
decreasing order of transferability:

1. **Hardware/spec facts** — register offsets, bit layouts, table formats,
   scancode tables, magic numbers. Pure data, not copyrightable as facts.
   *Transcribe.*
2. **Quirk knowledge** — the accumulated "this silicon lies about X, you must do
   Y first, watch for Z," embedded as workarounds and comments. Invisible in the
   specs, priceless, and the primary reason to read the source. *Harvest as
   notes.*
3. **Algorithm shape** — how to structure a page-table walk, a virtio queue, an
   AML interpreter. *Read as reference architecture; rewrite in Omega.*

The stealing is **densest at raw hardware facts** (the same silicon everyone
fights) and **evaporates going up** (drivers become capability-shaped;
allocators/locks/volatile are subsumed by Omega's own model). The reuse benefit
is smallest exactly where Cathedral is most novel — which is why authorship, not
dependency, is the right call.

---

## `reference_code/` — forked prior art, gitignored

Studied codebases are cloned into a top-level **`reference_code/`** directory
that is **gitignored** (see `.gitignore`). Nothing there is linked, built, or
shipped — it exists to be *read* while transcribing facts and harvesting quirks,
the way Dolrus keeps gitignored `dolphin/` and `mgba/` checkouts to compare
behavior against. Populate it as needed:

```
reference_code/
├── rust-osdev/         # uefi-rs, x86_64, uart_16550, acpi, virtio-spec-rs, …
├── edk2/               # the UEFI reference implementation (the spec, as code)
└── …                   # other OSes studied for a specific subsystem
```

Keeping it out of the tree keeps the all-in-house invariant honest: every file
that is *committed* is ours. `reference_code/` is a reading room, not part of the
system.

---

## The four passes

Learning from a studied codebase is four distinct kinds of work:

1. **Facts extraction.** For each device/ABI on the current milestone, write a
   `source/drivers/facts/<device>.omg` (or, for an ABI, a
   `source/contracts/<abi>/…`) that transcribes register maps and struct layouts
   as ordinary schema data plus `LayoutPlan` geometry and, for live registers,
   separate `AccessPlan` policy. Split fields use name-keyed fragments; device
   semantics such as W1C remain package machines over private plan-derived
   access. Cheap, zero-authority, reviewable against the datasheet — this is the
   bulk of "learning from rust-osdev."
2. **Quirk notes.** Read the crate's *comments and special-cases* and record the
   hardware-lies-about-X knowledge alongside the facts. You cannot get this from
   the spec and cannot afford to rediscover it by debugging on real silicon.
3. **Reference port.** For non-trivial algorithms (the memory-map dance, AML,
   virtio queues, xHCI), rewrite in Omega with capabilities, the layout/
   calling-plan machinery, and proofs where they pay. The crate is the
   architecture reference, not the source to copy.
4. **Tooling hookup.** Some prior art is just *used*, not ported — OVMF
   (`ovmf-prebuilt`) is the test firmware we boot under QEMU. That lands in
   `tools/`, never transcribed.

---

## Provenance discipline (matters for a sovereign TCB)

**Transcribe facts from the *primary sources* — the Intel SDM, the UEFI / ACPI /
PCI / VIRTIO specifications, device datasheets — using the rust-osdev crate as
the *index of which facts matter and a cross-check*, not the verbatim thing to
copy.** Two reasons:

- Facts from a datasheet are not copyrightable, so the provenance of code we are
  trying to *prove* trustworthy stays clean — no murky IP in the TCB. This ties
  directly to the trusting-trust / bootstrap-seed ethos: you audit what you can
  account for.
- A crate occasionally encodes a workaround you want to *understand* before
  adopting, not cargo-cult.

Record provenance per facts-file, e.g.:
`// transcribed from Intel SDM vol 3 §4.5; cross-checked against rust-osdev/x86_64`.

---

## The rust-osdev inventory, sorted by the work it implies

| Crate | Kind | What we do | Lands in |
|---|---|---|---|
| **uefi-rs** | ABI + logic | Transcribe EFI struct layouts (SystemTable, BootServices, ConOut, memory descriptor, GUIDs, status codes, protocol vtables); read the memory-map/ExitBootServices dance for quirks. **Milestone-1 goldmine.** | `contracts/` + facts |
| **x86_64** | facts + logic | Transcribe page-table / GDT / IDT / CR / RFLAGS / MSR bit layouts (→ `Bits` stated-plans). Setup logic is reference. | `drivers/facts/`, `core/` |
| **uart_16550** | facts | Serial register map — trivial transcribe. | `drivers/facts/` |
| **pic8259** / **apic** | facts + ritual | Port offsets + init sequences (remap-the-PIC, LAPIC setup). | `drivers/facts/`, `core/` |
| **pci_types** | facts | PCI config-space header, BAR formats, capability lists. | `contracts/`/facts |
| **virtio-spec-rs** | facts + protocol | Virtqueue descriptor/ring layouts + queue protocol — the first real driver's reference. | `drivers/` |
| **acpi** (+ AML) | facts + big logic | ACPI table parsing (facts) + the AML interpreter — reference for the flagged un-owned gap; the biggest port. | `services/` (confined interp) |
| **xhci** / **usb** / **vga** / **ps2-mouse** | driver logic | One exemplary port per class, later, on the count-budget. | `drivers/` |
| **bootloader** | boot logic | Reference for the real-mode→long-mode transition we skip by going UEFI-first — read to understand what UEFI does for us. | (study only) |
| **multiboot2** / **pvh** / **ieee1275** | alt boot ABIs | Deferred — multiboot2 only for a coreboot-payload reference platform; pvh only for a cloud target. | (deferred) |
| **linked-list-allocator** | primitive | Reject the ambient model — `Extent` owns backing-range authority and allocation strategies are ordinary packages over qualified extents. Reference the free-list algorithm only. | (subsumed) |
| **volatile** | primitive | Reject the wrapper — placed views derive sealed field operations from `Extent + LayoutPlan + AccessPlan`; volatile is an observation contract, not a type qualifier. | (subsumed) |
| **spinning_top** / **mem-barrier** | primitive | Subsumed by the concurrency model and checked instruction catalog: atomics/waits are ordinary contracted operations; fences/cache/TLB instructions emit complete target contracts. | (subsumed) |
| **ucs2-rs** / **endian-num** | util | UEFI strings are `u16` arrays; endianness is the layout/format machinery. Trivial/subsumed. | (subsumed) |
| **ovmf-prebuilt** | tooling | Use directly — it *is* the test firmware (OVMF.fd) we boot under QEMU. | `tools/` |
| **bootimage** / **cargo-xbuild** | tooling | Cargo-bound, irrelevant to the `build.omg` toolchain. Steal only the workflow idea (assemble FAT image → `\EFI\BOOT\BOOTX64.EFI` → OVMF). | (idea only) |

---

## Mapped to the first-boot ladder

- **Milestone 1 (UEFI hello):** uefi-rs EFI ABI shapes → `contracts/uefi/`;
  `ovmf-prebuilt` → `tools/` harness.
- **Milestone 3 (alive after firmware):** x86_64 / uart_16550 / pic8259 / apic
  facts.
- **Later:** acpi + AML (the flagged gap), virtio, pci, then one exemplary
  driver per class.
