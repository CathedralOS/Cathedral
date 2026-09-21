# Explicit-profile recursive translation

Status: **semantic constant evaluation tested**. All 330 reference observations
pass in 55 Omega fixtures, plus one invalid-input check and eight changed-body
controls. Producer source check passes 16 files. This extension reuses RecursiveTranslation/Translation and the tested
recursive decision order, then projects the selected leaf with the existing
explicit encryption state. It changes no default-profile implementation.

Modified pure `RecursivePageTable::translate` behavior from x86_64
`cc35c876d3badb57df54a66e22f7768a52be95f2`, under retained
[MIT OR Apache-2.0 notices](../../../licenses/rust-osdev/x86_64/).
The full pinned method, PTE address/flag masks, existing recursive implementation,
encryption configuration contract and Omega case-data rules were consulted.

`translate_words(profile, va, p4, p3, p2, p1)` preserves whole-word zero tests,
non-present ancestor traversal, huge-page truncation and separate root/leaf HUGE
errors. It does not substitute the ordinary mapped walk's PRESENT checks. Valid
profiles never remove those low flag bits from the flag mask, so the original
recursive shape decision remains reusable. The selected original leaf supplies
its profile-masked frame and flags, with existing size/offset geometry.

Only profiles produced by initial/configure fall within pinned equivalence.
Accumulated PTE mask removals survive repeated configuration, independently of
which current bit/polarity describes encryption. The caller supplies initialized
selected words; this API performs no table identity check or pointer resolution.
Numeric outputs establish no recursive topology, physical provenance, live
mapping, backing or access authority. LA57 and native layout remain separate.

The Rust reference copies the complete pinned generic recursive translation
body. Only topology resolution becomes four disjoint initialized table borrows,
and the former pointer call's inferred Page type becomes explicit Size4KiB.
Generator assertions bind each substitution and reject residual raw access.
Actual public PTE/PhysAddr/PhysFrame operations run with memory_encryption enabled.
This is an **adapted private-body reference**, not an actual RecursivePageTable
instance or public recursive mapper call. Caught panic strings distinguish the
two expected HUGE failures; every other panic fails the reference.

Eleven isolated processes configure before constructing physical/PTE values.
The 330 retained observations cover low/high canonical addresses, all sizes,
missing levels, non-present words, all-bit leaves, low/high configuration bits,
both polarities and repeated configuration. Omega fixtures execute the actual
composition bodies. Additional noncanonical input verifies rejection before
malformed root/leaf interpretation. Body controls change leaf flags, the leaf
HUGE outcome and invalid-address classification under unchanged success contracts.

The [inventory](encrypted-recursive-translation-inventory.json) binds all forty
pinned recursive mapper anchors and marks only this generic translate component.
Route mutation, cleanup and constructor profile composition retain their separate
work/evidence. No native Omega, CPU root, hardware encryption or ABI test runs.

Run `python3 tools/ports/x86_64-encrypted-recursive-translation/check.py`;
`--host-only` verifies fresh adapted reference observations and generated fixtures.
Compiler: clean Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`, SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
