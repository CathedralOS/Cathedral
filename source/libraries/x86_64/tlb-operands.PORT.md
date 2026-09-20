# Detached TLB instruction operands

## Scope and status

Current stage: **tested** by source checks, Rust witnesses and Omega semantic evaluation.

Implemented 18 pure machines over PCID, INVPCID descriptor bytes, explicit
CPUID-output interpretation, ASID/nested checks, INVLPGB register operands and
one-step range recipes. Four raw records and 12 numeric constants live in
`facts/x86_tlb_operands.omg`. Semantic command, optional selection, outcome and
range alternatives use cases; the fixed hardware descriptor is a separate
record. Existing address/page operations are reused.

No CPU observation, instruction execution, mapping mutation or invalidation
settlement is implied by a numeric result. Live TLB integration remains a
separate authority boundary. The bounded pure operand slice has no open
engineering or language blocker.

## Upstream pin and licensing

x86_64 0.15.5 at `cc35c876d3badb57df54a66e22f7768a52be95f2`, MIT OR Apache-2.0.
Primary source `src/instructions/tlb.rs`; [upstream licenses](../../../licenses/rust-osdev/x86_64/).
Derived files retain SPDX notices. Exact pure Rust fragments are extracted by
`tools/ports/x86_64-tlb/generate_reference.py`; source freshness is checked.

## Source and public-symbol map

[tlb-operands-inventory.json](tlb-operands-inventory.json) audits all eight
instruction modules and 104 lexical anchors: 25 translated and 79 explicitly
omitted in this slice. All 31 TLB anchors have dispositions. Private descriptor,
limit fields, builder selections and implicit command alternatives have
additional mappings. Counts describe source coverage, not independently
executed APIs.

| Family | Existing values and remaining work |
| --- | --- |
| TLB | This slice supplies detached values/recipes. INVLPG, INVPCID, INVLPGB, TLBSYNC and CR3 reload require explicit CPU/invalidation integration. |
| Segmentation | Existing SegmentSelector and FS/GS/KERNEL_GS MSR facts are canonical. Reads, writes, far returns and swapgs require instruction/entry contracts. |
| Tables | Existing descriptor-pointer, selector, GDT/TSS and IDT values remain canonical. Loading/storing live table registers needs backing/lifetime and instruction integration. |
| Port I/O | Numeric ports remain u16. Existing PIC PortIo integration remains canonical. Rust marker wrappers/Copy do not confer read/write authority; generic width providers and optional formatting/equality are ordinary future work. |
| Interrupts | Existing RFLAGS facts cover numeric bits. IF mutation, callbacks with restoration, software interrupts and enable-plus-halt need checked lifecycle integration. |
| SMAP | Existing AC/CR4 facts are reused. A standalone CPUID bit 20 observation helper is ordinary future extraction, while STAC/CLAC and restoration require provider/guard integration. |
| Random | A standalone CPUID bit 30 observation helper is ordinary future extraction. RDRAND success/failure and output require an admitted provider. |
| Instruction module | Module scaffolding and individual halt/no-op/debug/RIP leaves add no independent operand algorithm. |

These omissions are not labelled compiler blockers. Deprecated misspelled
`InvPicdCommand` and Rust formatting APIs are deliberately not reproduced.

## Primary specifications and representation vectors

AMD publication 24594, Volume 3, revision 3.36 (March 2024), printed page 388 / PDF
page 430 defines ECX[15:0] as an additional-page count, so zero still includes the
addressed page. CPUID Fn8000_0008 EDX[15:0] gives the maximum encoded count;
printed page 630 covers that field and NASID. The complete primary manual was
retrieved from this [Stanford-hosted AMD PDF](https://www.scs.stanford.edu/~zyedidia/docs/x86/amd-manual-v3.pdf)
after official AMD endpoints returned 404. PDF SHA-256:
`4d8eff047a237895dfa4433111511088a80b40dde9e2012f9b652113ee58a00e`.

[Intel SDM Volume 2A](https://cdrdv2-public.intel.com/812383/253666-sdm-vol-2a.pdf)
defines the 128-bit INVPCID descriptor: 12-bit PCID, reserved zeros and 64-bit
linear address. This pin uses canonical 48-bit addresses, not a generic LA57
profile. PCID accepts 0..4095; the upstream comment mentioning 4096 does not
change the constructor's strict bound.

[tlb-operands.vectors.json](tlb-operands.vectors.json) records 43 host Rust
observations from exact extracted pure bodies: PCID boundaries, descriptor
geometry/bytes, chunk decisions and broadcast registers. The original private
repr(C) descriptor is mirrored without changing fields; it measures 16 bytes,
alignment 8 and offsets 0/8. This is a private-source mirror measurement, not a
claim to access that private upstream type directly. Six UEFI-x64 compile-time
assertions independently check the actual upstream public Pcid type and
constructor boundaries. Descriptor geometry is not claimed as observed native
Omega layout.

## Translated tests and fixtures

Four Rust tests cover all 65,536 PCID inputs, all four INVPCID commands,
64 broadcast combinations, zero-count behavior, empty ranges and canonical-half
stepping. The host is not x86_64, so the upstream instruction module is gated
out; exact source extraction supplies its pure PCID/descriptor/composition
bodies. Real upstream VirtAddr/Page/Step operations remain dependencies.
Private page stepping calls are adapted to the public Step trait with the same
implementation. Assembly and CPU observation tails never execute.

Omega's const-evaluator fixture executes 14 PCID boundaries, 65 broadcast cases,
15 pinned/architectural range pairs, all INVPCID command cases, complete 16-byte
codec checks, CPUID-output interpretation and numeric rejection payloads.
Four body mutations alter an expected descriptor byte, command kind, range
count and broadcast ASID bits; each must evaluate to 1 and fail the unchanged
`requires value == 0` assertion.

## Deliberate deviations and pinned discrepancy

The pinned broadcast loop caps remaining pages by the encoded count maximum,
passes that count unchanged to ECX, then advances by max(count,1). Under AMD's
operand definition, positive count targets one more address than that advance.
`pinned_range_step` preserves this behavior; `architectural_range_step` caps
addressed pages by maximum+1, encodes pages-1 and advances by addressed pages.
Both limit chunk decisions at the canonical-half boundary using existing Step
arithmetic. One-page requests, maximum 0, maximum 65535, 2MiB strides and the
canonical gap have explicit expectations for both recipes. No physical CPU
result or stronger invalidation granularity is claimed: the architecture
permits invalidating additional entries.

Broadcast setters build detached selections. `broadcast_prepare` performs the
pinned builder's ASID/nested constraints plus explicit stride/address/count
validation before producing registers. CR4.PCIDE, EFER.SVME, actual feature
availability, current CPU/mode and observation provenance remain execution
obligations. Supplied numeric observations do not prove hardware support.
Raw descriptor decoding preserves every bit; only preparation from a bounded
Pcid/canonical address establishes this slice's operand validity.

## Omega blockers and boundary seams

The producer and semantic fixture pass. The requested 16-byte/alignment 8
InvpcidDescriptor Layout has an independent successful local-equivalent
consumer. The real imported type reproduces the existing generated-field
visibility diagnostic (`selects private data`), retained in
`layout_imported_probe.omg`. Thus plan normalization and local source access
are checked, while imported generated access and native ABI remain unproved.
This does not block the complete explicit descriptor byte codec.

## Verification commands and results

```sh
python3 tools/ports/x86_64-tlb/check.py --omega /tmp/cathedral-omega-eaa7993/release/omega
```

Rust uses `nightly-2026-09-04` for the pinned Page Step API and the installed
`x86_64-unknown-uefi` standard-library target. Omega revision:
`eaa7993a23623cd8fabf45350340479c5c9c7879`; compiler SHA-256:
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
The runner checks extracted-source/generated-fixture freshness, inventory,
43 Rust observations, six actual upstream target assertions, four Rust tests,
production source, semantic evaluation, local layout access, four body controls
and the exact imported-layout diagnostic. Expected failure probes are not
counted as successful compilation. No previous slice, core, mapper or PTE
source is modified by this slice.
