# Numeric mapper topology

Status: **tested**. All 22 Omega fixtures (3,139 assertions), 22 body mutations
and 3,131 Rust numeric witnesses pass; the source check passes 14 files.
[Verification record](../../../tools/ports/x86_64-mapper-topology/verification.json)
binds the selected source closure, compiler and every fixture.

This slice implements the `offset-address`, `recursive-coordinates` and `recursive-constructor` families identified by `remaining-pure.RECONCILIATION.md`. It reuses the accepted address, page/frame, index and page-entry helpers. Numeric results grant no raw pointer, recursive table reference, hierarchy access or mapping custody. No existing production import is changed.

The exact source is [rust-osdev/x86_64 cc35c876d3badb57df54a66e22f7768a52be95f2](https://github.com/rust-osdev/x86_64/tree/cc35c876d3badb57df54a66e22f7768a52be95f2), crate 0.15.5, under MIT OR Apache-2.0. The upstream contributors and exact license texts remain in `THIRD_PARTY_NOTICES.md` and `licenses/rust-osdev/x86_64/`. This is a modified numeric extraction, with checked ordinary inputs and semantic result cases. `mapper-topology-inventory.json` binds the complete two relevant mapper source files; constructor and pointer-method mappings explicitly cover only their numeric components.

## Operations and input policy

`offset_frame_address(offset, frame_start)` validates a 4 KiB physical frame start and computes the pin's ordinary virtual-address addition through `addresses::virtual_add`. Noncanonical input/output, the canonical hole and integer overflow reject. Addition does not skip the hole. The final raw-pointer conversion from `PhysOffset::frame_to_pointer` is excluded. The selected physical profile is the existing default 52-bit envelope, without a configured encryption mask.

The recursive functions return 4 KiB virtual page starts formed by `pages::from_indices`:

| Function | Permitted source sizes | Output indices |
| --- | --- | --- |
| `recursive_p3` | 4 KiB, 2 MiB, 1 GiB | `(r, r, r, p4)` |
| `recursive_p2` | 4 KiB, 2 MiB | `(r, r, p4, p3)` |
| `recursive_p1` | 4 KiB | `(r, p4, p3, p2)` |

Inputs must be canonical, aligned page starts and `r < 512`; invalid ordinary inputs return `NumberResult::Rejected`. The source's `PageSize`/`NotGiantPageSize` restrictions become these explicit checks. All 512 recursive indices are numerically representable. The pin's index-511 warning concerns forming references at the end of the address space; numerical representability here establishes no such access.

`observe_recursive(table_address, observed_cr3_frame, recursive_entry_word)` extracts only the constructor's observations. It validates canonicality, takes the containing 4 KiB page, obtains P4, and compares with the page formed by repeating that index four times. `NotRecursive` precedes any active-frame comparison. A repeated address then requires an aligned default-profile observed frame; an invalid supplied frame returns the added `InvalidObservedFrame` case. For valid observations, the supplied entry's `frame()` classification preserves PRESENT-before-HUGE ordering. Both failures produce `NotActive` with an explicit diagnostic reason, as does a differing physical frame. Matching values produce `Observed { recursive_index, physical_frame }`.

The containing-page calculation ignores byte offsets, as the extracted constructor body does. The original `&mut PageTable` separately supplies alignment, validity and lifetime guarantees. This ordinary-number API supplies none of those guarantees, even when its observations agree. The caller must establish that the supplied entry was observed at the corresponding recursive index and that the CR3 observation has the required freshness. No CR3 instruction or table dereference occurs here.

## Reference and semantic evidence

`tools/ports/x86_64-mapper-topology/generate_reference.py` binds three exact private coordinate function bodies to the pinned source. They are compiled as local mirrors, **not called as public upstream APIs**. The constructor mirror replaces pointer acquisition, CR3 reading and table indexing with explicit observed inputs, and returns the index instead of a borrowed mapper. Its ordered checks otherwise come from the pinned body. The offset mirror retains the exact numeric addition expression and excludes `as_mut_ptr`.

The host witness uses actual public `VirtAddr`, `Page`, `PhysFrame`, `PageTableEntry` and flag operations. It records 3,131 numeric results: 3,072 coordinate comparisons across all recursive indices and six permitted size/level combinations, 11 offset boundaries, and 48 constructor/error-order observations. Source-page indices cycle through zero, low-half maxima and high-half cases. This is finite coverage, not an exhaustive proof over every source page.

Generated Omega fixtures evaluate the actual new machines against those observed values, plus eight invalid-input policies. Every fixture has a mutation inside its expected behavior; the changed body must compute failure 1. These checks establish semantic behavior, not native ABI agreement, live mapper execution or pointer validity. The checker hashes the exact selected source dependencies so concurrent unrelated modules do not invalidate or masquerade as this evidence.

## Remaining boundaries and pin updates

The other offset mapper methods delegate to the existing mapped routes, translation and cleanup; they are not newly implemented algorithms here. Recursive route, translation and cleanup work belongs to its separate slices. Unsafe traits, live constructors, allocator ownership, table access, activation and invalidation remain outside this module.

On a pin change, review both full file hashes and all anchors, regenerate the exact private bodies and inspect every constructor substitution, review licenses and default physical-mask semantics, then regenerate reference values and rerun Omega behavior/control checks. Do not promote the constructor's numeric mapping into a claim that its borrowed live API is translated. Narrow pending inventory rows are not a global status override for algorithms completed in other slices.
