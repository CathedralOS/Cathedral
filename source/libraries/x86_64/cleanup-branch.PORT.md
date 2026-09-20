# Detached single-page cleanup branch

Current stage: **tested**. All 112 Rust/Omega branch scenarios, five additional
fixtures and three body mutations pass. This is a
bounded component of X86-002, not completion of arbitrary-range cleanup.

Modified translation of x86_64 `cc35c876d3badb57df54a66e22f7768a52be95f2`,
`src/structures/paging/mapper/mapped_page_table.rs::CleanUp`, under the retained
[MIT OR Apache-2.0 notices](../../../licenses/rust-osdev/x86_64/). The exact
recursive method and its full-table `is_unused` test were reviewed, together
with Omega ownership/data contracts and the existing page/PTE implementations.

The input describes one canonical aligned 4 KiB page's P4/P3/P2/P1 entries,
the numeric identities of their containing snapshots and whether any other
entry in each snapshot is nonzero. `has_other_entries` computes that observation
over all 512 initialized entries while excluding exactly the selected index.
It reuses the canonical PTE schema and codec.

Traversal follows PRESENT/non-huge parents and checks each next snapshot's
identity. A missing, non-present or huge link stops descent, as the pinned
cleanup ignores next-table errors. The reached table is empty only when both
its selected word and every other word are zero. Non-present software data
still prevents reclamation. A huge mapping is never traversed or retired.

An empty child causes a zero-word request for its parent and appends the child's
frame identity to `frames_to_retire`. The algorithm repeats upward while the
remaining parent entries are all zero. Retirement order is deepest child first;
the P4 root is never retired. A nonempty sibling stops upward retirement after
any already selected child removal. The word array retains untouched input
values, while `write_mask` selects actual proposed entry changes.

Invalid page geometry produces no effects. CaptureMismatch reports expected
and supplied numeric IDs; checking the whole descent before unwinding means
this profile has no earlier detach requests on that failure. Snapshot IDs and
emptiness booleans are ordinary observations, not proof of provider identity,
exclusive custody, consistent snapshots or an alias-safe tree.

This machine does not modify a table, call a deallocator, complete invalidation
or release ownership. A later owner must separately validate its live snapshots
and backing hierarchy, apply parent clears in the specified order, retire all
loans and complete the required CPU/IOMMU lifecycle before reusing any proposed
frame. An empty numeric snapshot cannot authorize reclamation.

## Verification

The 112 original cases execute actual pinned `clean_up_addr_range` calls for a
singleton page on stable owned detached tables. They cover all stopping depths,
zero/non-present/huge entries, all combinations of sibling occupancy and exact
bottom-up deallocator callback order. Non-selected nonzero entries at index 511
prove that checking only the selected slot is insufficient. Backing allocations
remain owned by the test so final cleared links can be observed after callbacks;
no table is installed or returned to a live allocator.

The same Omega plans are evaluated in bounded groups. Additional fixtures cover
invalid page starts, deep capture mismatch and three complete 512-entry scans:
all empty, only the excluded final slot nonzero, and a non-present nonzero final
neighbor. Body mutations change retirement count, retirement order and the
full-array scan result while retaining the unchanged success requirement.
No native Omega execution, layout measurement or production integration is
claimed. Existing canonical facts are unchanged.

The [source inventory](cleanup-branch-inventory.json) keeps full range cleanup
pending and links this tested component explicitly. It does not mark the entire
Rust method translated merely because a singleton range is supported.

Compiler: Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`; binary SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.

```sh
python3 tools/ports/x86_64-cleanup-branch/check.py --omega /path/to/omega
python3 tools/ports/x86_64-cleanup-branch/check.py --host-only
```
