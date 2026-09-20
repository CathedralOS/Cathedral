# Recursive cleanup orchestration

Status: **tested**. The 16-file source check, 20 pinned whole-tree witnesses,
40 actual Omega fixtures and 40 body-mutating controls pass. The verification
record binds the selected source closure, compiler and every fixture.

This slice translates recursive cleanup into bounded detached cursor steps. It
composes the existing `cleanup_ranges` and `cleanup_branch` numeric plans and
excludes the recursive P4 slot before inspecting any supplied capture. It creates
no recursive table reference, writes no page-table entry and releases no frame.

The source is [rust-osdev/x86_64 cc35c876d3badb57df54a66e22f7768a52be95f2](https://github.com/rust-osdev/x86_64/blob/cc35c876d3badb57df54a66e22f7768a52be95f2/src/structures/paging/mapper/recursive_page_table.rs),
crate 0.15.5, MIT OR Apache-2.0. Exact licenses and contributor provenance remain
in `THIRD_PARTY_NOTICES.md` and `licenses/rust-osdev/x86_64/`. This is a modified
numeric extraction. `recursive-cleanup-inventory.json` binds every anchor in the
full recursive mapper source; its translated rows denote only this detached part
of `clean_up`/`clean_up_addr_range`, not the original live trait interfaces.

## Cursor and snapshot contract

`begin(first, last, budget, recursive_index)` accepts canonical, aligned 4 KiB
page starts and `recursive_index < 512`. Invalid inputs return `InvalidRange`.
A reversed valid range is already `Done`; a zero budget otherwise gives
`Exhausted`. The budget accepts every u64 value. `resume` replenishes only an
exhausted cursor and preserves its recursive index. Every active step revalidates
its range; a publicly constructed record therefore cannot bypass ordinary input
checks. The index is retained data, not an unforgeable authority token.

For an ordinary P4 slot, `step` forwards the selected words, numeric table IDs and
other-entry occupancy observations to the existing range helper. It returns its
bottom-up branch effects, capture mismatch and next cursor. Occupancy must include
the recursive self-link in `other_nonzero[0]`; forgetting it could incorrectly
claim the entire root is empty and skip later ordinary slots. The caller must
supply consistent detached snapshots, apply/retain proposed effects, and capture
the next selected branch from the resulting snapshot. Numeric IDs establish no
live table identity or ownership. A valid rooted hierarchy and the established
self-link are caller obligations; this module does not rediscover topology.

For the recursive P4 slot, the step produces `has_plan = false`, consumes one
budget unit and skips the remaining 512 GiB slot coverage. Supplied child captures
are ignored, including malformed captures. If the range ends within that coverage,
the cursor is done without forming a successor; otherwise dense 48-bit arithmetic
forms the next canonical page, crossing the canonical gap when needed. Final-slot
coverage cannot wrap the address space. The skipped step neither traverses nor
proposes retiring the root. A default branch value when `has_plan = false` is not
a cleanup result and must not be applied.

## Validation and source adaptation

The tests compile an exact extracted private `clean_up` body against the pinned
crate. Recursive pointer resolution alone is replaced by a stable registry of
owned initialized tables, with an added registry argument. The private address
step helper is reached through public `Step::forward_checked`, whose exact pin
body delegates directly to it. These are checked textual substitutions; the
reference is explicitly a local mirror, not an actual public recursive mapper.
The root is absent from the registry, so traversing the self-link fails. Retire
callbacks record order while keeping allocations alive for final observation.

Whole-tree cases include recursive indices 0, 255, 256 and 511, skipped interior
and final coverage, the canonical gap, ordinary branches on both sides of a
self-link, nonpresent/huge entries, occupied leaf tables, and bounded resumption.
Every final nonzero word and retirement callback sequence is compared. Generated
Omega fixtures check each actual cursor/plan step and separate fixtures exercise
resume chains and checked errors. Each fixture has a body-mutating negative
control that must compute failure 1. These are semantic checks, not native ABI
or live hardware evidence. See the tools [README](../../../tools/ports/x86_64-recursive-cleanup/README.md)
and [verification record](../../../tools/ports/x86_64-recursive-cleanup/verification.json).

The u64MAX regression initially failed because the current evaluator misclassified
a direct record-field ordered comparison. The skip helper now stages its budget
as a scalar parameter before comparing/subtracting; the shared range helper uses
the corresponding scalar staging. This is a source-form workaround, not a reduced
budget policy. Tests retain both maximum-budget paths and a top-bit budget case.

On a pin update, audit all file anchors and license changes, regenerate and review
the private extraction substitutions and the public/private Step equivalence,
review recursive-slot and occupancy rules, regenerate the finite corpus, and run
all Rust/Omega checks and body controls. Live hierarchy access, publication,
synchronization, TLB invalidation and allocator ownership remain separate work.
