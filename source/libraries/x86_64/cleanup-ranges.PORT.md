# Bounded captured cleanup ranges

Current stage: **tested**. Thirteen actual pinned whole-range Rust witnesses,
18 Omega cursor-step fixtures, six additional fixtures and four body-mutating
controls pass. No native Omega or live reclamation execution is claimed.

Modified pure orchestration of x86_64
`cc35c876d3badb57df54a66e22f7768a52be95f2`,
`src/structures/paging/mapper/mapped_page_table.rs::CleanUp`. Retained
[MIT OR Apache-2.0 notices](../../../licenses/rust-osdev/x86_64/) apply. The
upstream recursive range algorithm, canonical page/Step behavior and Omega
data/ownership contracts were consulted. Existing [branch cleanup](cleanup-branch.PORT.md)
and page/address representations are reused.

## Range algorithm

`begin` checks both inclusive endpoints as canonical aligned 4 KiB pages.
Reversed ranges are Done without work; a nonempty range with zero budget is
Exhausted. The default record is InvalidRange. To express the upstream
whole-space `clean_up`, use endpoints zero and `0xfffffffffffff000`.

Each `step` receives the selected page's initialized P4/P3/P2/P1 words, table
IDs and complete other-entry occupancy observations. It applies the tested
branch algorithm to produce detached parent-clear and deepest-first retirement
requests. The caller retains that proposal and obtains the next snapshots after
those proposed edits in its simulation. No callback or actual table mutation
is hidden inside this interface.

Cleanup never removes a mapped leaf. Consequently a reached P1 table's full
emptiness observation settles its entire 2 MiB coverage; inspecting another
leaf in that same unchanged table cannot reclaim anything further. Absent or
huge P3/P4 links settle the corresponding 1 GiB/512 GiB region. An entirely
empty root settles the remaining range immediately. The cursor skips that
settled coverage, clamped against the inclusive last page. It uses dense
canonical coordinates and sign extension to cross the canonical gap without
manufacturing a noncanonical intermediate page or wrapping the last page.

This traversal preserves the pin's final clears and callback order on a
consistent acyclic tree. It visits potentially reclaimable branches in
ascending address order. Early upward retirement only skips subsequent zero
entries: a nonzero entry outside the selected branch prevents that retirement.
Non-present software words and huge mappings remain untouched. The root table
itself is never retired. A nonempty table outside the requested range prevents
its parent from being incorrectly reclaimed.

One successful branch consumes one budget unit. If more range remains, zero
remaining budget returns Exhausted with the exact next page. `resume` accepts a
new budget only for Exhausted and revalidates that remaining interval; it does
not revive an invalid, mismatched or completed cursor. Nonactive steps produce
no branch plan. Publicly constructed Active cursors are revalidated before
use, including their budget, so changing a case tag does not bypass geometry.

CaptureMismatch stops before any branch detach request and retains the failed
page. Its diagnostic attempt does not consume a successful-branch budget unit.
Earlier returned proposals are still the caller's responsibility. Neither
Exhausted nor mismatch is a completion claim or rollback of earlier effects.

## Custody and deviations

These records are ordinary copyable data. Table IDs, occupancy observations,
cursor status and deallocation proposals confer no authority and do not prove
snapshot consistency, table provenance, alias freedom or ownership. A later
owner must separately enforce those invariants, retain the proposed effect
sequence and resolve all live loans, invalidation and reclamation obligations.
Done means the specified **numeric traversal** completed, not that any frame is
safe for reuse. This interface has no allocator, deallocator, flush token,
physical pointer or receipt constructor.

The pin performs recursive callbacks over live borrowed tables. This adaptation
is a bounded resumable cursor over caller-supplied snapshots and emits inert
plans. It checks full-table occupancy through the branch helper, coalesces
already settled address coverage and exposes exhaustion/missing capture.
These are deliberate API and work-accounting changes. The resource budget
counts successful branches, not CPU instructions or compiler evaluation steps.
The budget is staged as a typed scalar before comparison/subtraction: direct
record-field ordering at `u64::MAX` misbehaved in the pinned Omega evaluator.
The high-bit/max regression evaluates the real cursor helper; no compiler source
was changed.

## Evidence

The host harness calls the actual pinned `clean_up_addr_range` on distinct,
stable owned PageTable allocations through a checked ID registry. Generated
links are acyclic and unaliased. Its callback records retirement order while
backing remains owned for final inspection; no root is installed on a CPU.
Thirteen completed scenarios compare **every remaining nonzero entry** in all
five initialized tables and the exact deallocator sequence. Cases include an
empty whole address space, an empty hierarchy, adjacent P1 children, partial
ranges, non-present words, huge parents, high-half addresses, the canonical
gap, the final page and a reversed interval.

Original cursor fixtures check every step of those nonempty scenarios plus a
budget-limited prefix. Six additional Omega fixtures check malformed inputs,
inactive steps, forged Active state, deep snapshot mismatch and an exhausted
two-child traversal resumed to completion, and budgets at `2^63` and `u64::MAX`.
Four body mutations change Done to Exhausted, the next-page skip, the resumed
retirement order and the maximum-budget decrement. Each must
compute failure under the unchanged success requirement.

[The complete source-file inventory](cleanup-ranges-inventory.json) marks its
three cleanup anchors translated at this explicit pure boundary. Other mapper
methods retain their separate slice maps; no whole mapper/driver integration
claim follows. No previous core, PTE or production build is changed.

Compiler: Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`, binary SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.

```sh
python3 tools/ports/x86_64-cleanup-ranges/check.py --omega /path/to/omega
python3 tools/ports/x86_64-cleanup-ranges/check.py --host-only
```
