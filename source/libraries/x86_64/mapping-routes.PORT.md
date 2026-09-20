# Complete captured mapper routes

Status: **tested** by pinned Rust witnesses and Omega semantic evaluation. This X86-001/002
slice composes the already tested [child/leaf decisions](mapping-plans.PORT.md)
across the complete selected hierarchy. It modifies algorithms from pinned
x86_64 `cc35c876d3badb57df54a66e22f7768a52be95f2`,
`src/structures/paging/mapper/{mapped_page_table,mod}.rs`, under the retained
[MIT OR Apache-2.0 notices](../../../licenses/rust-osdev/x86_64/).

The pinned method bodies, Omega's data/case and ownership contracts, and the
existing address/page/PTE helpers were reviewed before implementation. Requests
and outcomes use semantic cases, while an ordinary record carries detached
numeric edits. No raw union or live mapper object is introduced.

## Inputs and retained effects

`CapturedPath` supplies four selected entry words and the numeric identities of
their containing tables, ordered P4/P3/P2/P1. `plan` validates the selected page
start and 4 KiB/2 MiB/1 GiB size, computes its four indices, and validates a map's
supplied frame geometry before doing any work. A caller is responsible for
obtaining the selected initialized words from the named snapshots. Numeric ID
equality does not establish snapshot provenance, physical backing, unique
ownership, freshness, an alias-safe tree or a hardware mapping.

Map traverses one, two or three parents, consuming the next explicit allocation
observation only for an unused word. Allocation attempts include a returned
Unavailable value. Existing parents receive the pinned OR operation before
PRESENT and HUGE_PAGE classification. Each completed assignment is retained in
the word array and `write_mask`, even if a subsequent check fails. Parent
classification errors wrap the exact tested `ChildPlan`; failures do not imply
rollback.

A newly created child contributes a full-table zero request to `zero_mask` and
replaces the next selected word with zero. Its supplied stale capture is ignored;
the next table ID becomes the resulting parent address. An existing child must
match the captured ID before its selected word is used. CaptureMismatch retains
preceding parent edits. Flags containing address bits follow the pinned numeric
operation; the matching capture must describe the resulting target.

Unmap, flag update and size-specific translation require each traversed parent
to be PRESENT and non-huge, in that order. They reuse the complete tested leaf
decisions, including the distinct leaf PRESENT requirements, huge-role errors,
frame alignment errors and requested-frame payload on AlreadyMapped. `visited`
records the number of selected words examined before the outcome.

SetParent selects P4/P3/P2, and returns ParentHugePage immediately when the
requested table level is at or below the selected huge leaf, matching the pinned
methods. The selected parent entry only has to be nonzero; updating it can clear
PRESENT or introduce HUGE_PAGE/address bits. Its ancestors still require normal
table traversal. These numeric rules are not an architectural admission policy.

The masks refer to P4/P3/P2/P1 positions. Zero requests concern complete child
tables; writes concern their selected words and may follow a zero request on
the same table. Both are ordered from root toward leaf. Unvisited array values
remain input snapshots and are not additional effects. Outcomes and effect
records are ordinary copyable proposals, including success values.

## Authority and invalidation

No table is mutated by `plan`, no allocator/deallocator is called and no frame
or reference is constructed. AllocationInputs are observations supplied for
simulation. A later owner must separately establish disjoint new-frame custody,
tree consistency, source snapshots and mutation authority before applying any
proposal. The captured model must not replay allocation observations as grants.

Successful map/unmap/update requires the same later invalidation obligations as
its live operation; parent flag changes may require broader invalidation. This
API deliberately returns no flush receipt, ignore capability or completed
settlement. Failure after a proposed parent change also does not erase that
owner's consistency or invalidation obligations. Translation is read-only.

## Reference evidence and limits

The harness runs actual public pinned Mapper operations on stable owned
PageTable allocations. UnsafeCell permits the mapper's interior writes; the
registry contains distinct checked IDs and generated inputs supply only
previously unlinked child tables. There are no cycles, concurrent aliases or
installed translation roots. Only the isolated reference harness ignores flush
tokens because its tables have never been installed.

The 208 original scenarios cover all three sizes, all four leaf operations,
parent PRESENT/HUGE failures at every depth, allocation exhaustion, partial
new-table suffixes, flag changes, leaf failure after parent writes and all nine
page-size/parent-level selections. Expected final words, allocation attempts and
full-table clearing are checked against actual Rust observations, including
state retained after caught pinned panics. A dirty last entry demonstrates
clearing beyond the selected slot. Assignment masks and visitation order also
follow the reviewed branches; ordinary Rust memory inspection cannot distinguish
redundant writes from no write.

Omega evaluates the actual composed machines in bounded groups. Additional
malformed-input/capture cases cover invalid page/frame/size/parent level,
capture mismatch after a write, malformed allocation and replacement of a stale
capture by a newly cleared table. All 208 Omega scenarios, four additional fixture groups and four body mutations
pass. Body controls change expected edits or result cases while retaining the
success contract. The producer source check passes 16 files. Native Omega execution, live
hierarchy mutation, hardware instructions and production integration are outside
this evidence.

[The source inventory](mapping-routes-inventory.json) preserves pending cleanup,
convenience wrappers and live interfaces. It does not claim the complete mapper
package is finished. Previously translated size-discovering translation remains
in [its separate slice](tables.PORT.md).

Compiler: Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`, binary SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.

```sh
python3 tools/ports/x86_64-mapping-routes/check.py --omega /path/to/omega
python3 tools/ports/x86_64-mapping-routes/check.py --host-only
```
