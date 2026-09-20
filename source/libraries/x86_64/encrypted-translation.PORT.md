# Explicit-profile captured translation

Status: **tested**. The 18-file source check, all 330 Rust/Omega observations
(55 fixtures, each checking word and captured-path results), seven additional
boundary assertions and nine body-mutating controls pass. This extension composes the tested default
mapped-table walk with the explicit memory-encryption state. It changes no
existing walker or canonical PTE representation.

Modified pure algorithms from x86_64
`cc35c876d3badb57df54a66e22f7768a52be95f2`, specifically
`MappedPageTable::translate` and the encryption-dependent entry masks. Retained
[MIT OR Apache-2.0 notices](../../../licenses/rust-osdev/x86_64/) apply. The
complete pinned method, memory-encryption safety/configuration contract, PTE
accessors, Omega case/borrowing contracts and library charter were consulted.

`encrypted_translation::translate_words` accepts the existing explicit
`memory_encryption::State` and four selected initialized entry words. The
existing `Translation` cases remain canonical. Valid profiles arise from
`initial` followed by successful `configure` calls; arbitrary edited masks are
outside pinned equivalence. Those operations only remove address bits from the
default PTE mask, so zero/PRESENT/HUGE decisions and leaf size selection are
unchanged. The helper reuses those decisions and projects the selected original
leaf with the profile's address and flags masks. Huge-page bases retain the
pin's containing-frame truncation; the 4 KiB nonzero/non-present behavior and
root-HUGE explicit error remain unchanged. Returned flags belong to the leaf.

`walk_path` reuses `CapturedPath` and `CapturedWalk`. Its caller selects entries
by virtual-address indices before calling. Each reached child ID must equal the
profile-decoded parent address. A mismatch is reported before any later leaf
error. Unneeded captures are ignored after an absent or huge parent; a malformed
virtual address rejects before identity checking. The three parent stages are
finite and explicit. Numeric IDs establish no physical provenance or live access.

Repeated configuration removes every previously configured address bit, while
the current encryption polarity/bit remains separate. The same accumulated mask
is used for both child identities and the final frame/flags split. Reconfiguring
a detached profile does not adapt, validate or install existing hardware tables.
The original default-profile modules retain their interfaces and evidence.
Recursive translation and mutation/cleanup under an encryption profile remain
separate composition work; this extension claims only mapped generic translation.

The Rust witness calls the actual public pinned `MappedPageTable::translate`
with `memory_encryption` enabled. Eleven isolated subprocesses configure before
constructing any PTE/physical address, then own distinct stable initialized tables
for read-only translation through a checked ID registry. No CPU root is installed.
The retained 330 observations cover two canonical halves, all three page sizes,
absent and malformed parents, non-present leaves, truncation, all-bit words,
both polarities, low/high configuration positions and repeated configuration.
Low/flag-bit positions test representable arithmetic rather than CPU admission.
Expected root-HUGE panics are captured; any other panic fails the witness.

Omega fixtures call both the word helper and captured-ID helper for every Rust
observation. Separate cases check all three encrypted/raw-ID mismatches, invalid
virtual input and early termination before malformed unused captures. Mutations
change expected leaf flags and child addresses inside executed assertion bodies.
Native Omega artifacts, hardware encryption, live table access and custody are
not exercised. The [inventory](encrypted-translation-inventory.json) retains all
47 anchors in the pinned mapper file, marking only this translate component.

Reproduce with `python3 tools/ports/x86_64-encrypted-translation/check.py`;
`--host-only` checks exact Rust results and generated fixture freshness.

Compiler: clean Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`, SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
The [verification record](../../../tools/ports/x86_64-encrypted-translation/verification.json)
binds the tested source closure, fixture bodies and exact retained Rust results.
`verify_record.py` checks those current hashes without rerunning semantic tests.
