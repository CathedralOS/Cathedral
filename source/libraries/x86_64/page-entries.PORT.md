# PTE word codecs, indexes and levels

This tested X86-001/002 slice uses the existing `X86PageTableEntry` semantic
schema. It introduces no competing PTE, table owner or bootstrap policy.
`page_entries.omg` translates pure operations from pinned x86_64
`cc35c876d3badb57df54a66e22f7768a52be95f2`, `src/structures/paging/page_table.rs`.
MIT OR Apache-2.0 and the [retained notices](../../../licenses/rust-osdev/x86_64/)
apply to the modified derivatives.

## Semantics and limits

Decode/encode cover all 64 bits, including software fields and the four-bit
protection key. Zero means the whole word is zero, not merely PRESENT clear.
Frame lookup uses semantic `NotPresent`, `HugeFrame` and `Value` cases. It
preserves the pin's error order and its generic interpretation of bit 7 as
HUGE_PAGE; hardware uses that same bit as PAT in PT leaves. This convenience
accessor is therefore not a paging-role admission check.

Address extraction uses the default 52-bit mask. `set_address` checks alignment
and the physical envelope, then retains the pin's raw OR behavior. Arbitrary
`from_bits_retain` flag words can contain address bits; `set_flags` preserves
the original masked address and ORs those supplied bits, including contamination.
Fixtures explicitly test this behavior. No resulting word proves reserved-bit,
CPU-feature, encryption, alignment-for-huge-page or role-policy validity.

Index/offset construction returns checked numeric values; truncation retains
the low 9/12 bits. Index stepping checks both range and extreme counts, and
overflowing stepping returns the original index with a flag. Level transitions
and entry/table spans accept exactly levels 1–4. All counts use u64; no 32-bit
usize profile, LA57 or ambient memory-encryption state is represented here.
Zero construction/reset uses `decode(0)` and ordinary owned-value replacement.

Omega data/case, ownership and layout contracts, the existing core validators
and [source reconciliation](RECONCILIATION.md) were reviewed. Intel's
[SDM Volume 3A](https://cdrdv2-public.intel.com/874249/253668-090-sdm-vol-3a.pdf),
chapters 4–5, supplies canonicality and paging context. Existing core four-level
walk policy remains above this reusable package and does not gain dependencies.

## Layout modernization

The canonical field schema remains in `facts/x86_page_table_entry.omg`, retaining
its legacy flattened import surface. Its old receiver-owned layout buffer failed
fresh source checking because returning `self.entries` moved a non-copy value
out of borrowed storage. The policy now uses a local buffer and explicit Layout
witness in `facts/x86_page_table_layout.omg`. The same fourteen bit placements,
eight-byte size and eight-byte alignment are retained. Separating layout imports
also avoids the known combined semantic/layout fixture `PlacedField` trait-arity
diagnostic. The old plan name remains available from the new explicit module.

The existing layout canary now uses current package wiring and demands every
field projection of a local equivalent schema through the actual policy. Its
source audit binds that local test schema to the canonical field definitions and
checks every requested bit position. Fresh Omega checks all thirteen sources.
This is actual plan/field source checking, not observed native bytes. Imported
plan-laid field access and native ABI evidence are separate compiler frontiers.
The retained `tools/ports/x86_64-page-entries/layout_imported_probe.omg` fails on
fresh eaa7993 with `selects private data
X86PageTableEntryLayout<X86PageTableEntry>::present`; no imported-layout usability
claim is made from the passing local equivalent.
The retired JSON-artifact jq assertion is retained only as historical material.

## Evidence and remaining work

[page-entries-inventory.json](page-entries-inventory.json) audits the complete
85-anchor source file: 55 translated, 14 deliberate omissions and 16 pending
full-table anchors. Detached table construction, mutation and traversal remain
engineering work. Existing core table candidates stay canonical. No language
blocker is asserted for those unfinished algorithms.

All 151 Omega scenarios execute actual bodies in bounded constant fixtures;
143 also execute the actual pinned Rust APIs. Tests cover each individual word
bit, every decoded field, zero/nonzero state, flags and address contamination,
frame errors, index/offset limits, overflowing stepping and level geometry.
The pin's index-overflowing unit test passes on `nightly-2026-09-04`; no pinned
source is patched and no instruction feature is enabled. Three body mutations
change PRESENT, protection-key and overflow expectations; each computes failure
and rejects the unchanged success contract.

Fresh compiler revision: `eaa7993a23623cd8fabf45350340479c5c9c7879`.
Binary SHA-256: `2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
All numeric cases and controls pass. No native execution, pointer validity,
memory mapping, hardware table access or production integration is claimed.

```sh
python3 tools/ports/x86_64-page-entries/check.py --omega /path/to/omega
OMEGA_BIN=/path/to/omega tools/x86-page-table-layout-canary/run.sh
```

Pin updates require reviewing complete source bytes, pending dispositions and
intentional raw-flag semantics before regeneration.
