# Captured recursive mapper routes and translation

Status: tested as detached numeric algorithms. This slice implements
the remaining-pure audit's recursive-routes and recursive-translation families.
It changes no existing source, production root or canonical representation.
[Topology coordinates and constructor observations](mapper-topology.PORT.md)
and recursive cleanup have separate ownership and evidence.

Modified algorithms come from x86_64
`cc35c876d3badb57df54a66e22f7768a52be95f2`,
`src/structures/paging/mapper/recursive_page_table.rs`. Retained
[MIT OR Apache-2.0 notices](../../../licenses/rust-osdev/x86_64/) apply. The exact
pinned method bodies, existing capture/leaf helpers and Omega data, machine and
authority contracts were reviewed. Both new modules operate only on detached
numeric inputs; no RecursivePageTable instance or pointer-derived access exists.

## Reused representations and recursive differences

[recursive_routes.omg](recursive_routes.omg) reuses Request, CapturedPath,
AllocationInputs, Edits, Outcome and RoutePlan from the ordinary captured routes,
and ChildPlan/LeafPlan from the tested leaf algorithms. Page/frame geometry,
entry words and all five leaf decisions reuse their existing implementations.
Private orchestration follows the existing route shape with the precise
recursive parent rules. `prepare_child` exposes the distinct recursive child
component for reuse; `plan` composes all selected parents and the leaf.

New links force PRESENT|WRITABLE even when parent flags are zero. Allocation is
attempted before a new-link HUGE flag produces the explicit InvalidHugeInput
outcome corresponding to pinned set_frame assertion failure. That failure leaves
the entry unchanged and requests no clearing. Existing links receive the pinned
flag OR before checking HUGE; any preceding write remains in the returned plan.
They are not required to have PRESENT. Only newly created children request a
complete table clear. Their next selected word becomes zero and their snapshot
ID is replaced with the resulting parent address; stale captured values are
ignored at that point.

Unmap's ancestors require PRESENT and then reject HUGE, preserving frame() error
order. Update, set-parent and size-specific translate instead reject only an
entirely zero ancestor word. Nonzero non-present or HUGE ancestors therefore
continue in this numeric model. This source behavior does not demonstrate that
such a hierarchy could safely be traversed on hardware. Set-parent rejects a
level at or below the requested leaf before inspecting a selected word.

Map, unmap, flag update and size-specific translation share their exact leaf
branches with the ordinary mapper: whole-word occupancy, size-specific HUGE
handling, alignment errors, requested-frame AlreadyMapped payload, and flags
that may clear PRESENT or introduce address bits are preserved. Later failures
do not roll back prior parent proposals. Assignment masks record even redundant
source setters; allocation attempts include an unavailable observation.

[recursive_translation.omg](recursive_translation.omg) implements generic
size-discovering translation. Every level first checks whole-word zero, without
a parent PRESENT test. A P4 HUGE word yields the canonical InvalidRootHugePage
result; a P1 HUGE word yields the new InvalidLeafHugePage case, corresponding to
the two pinned panics. Intermediate huge entries truncate frame addresses to
1 GiB/2 MiB boundaries and return their leaf flags and virtual offset. A nonzero
non-present PT word can return a numeric mapping. Invalid virtual addresses are
explicitly rejected before any selected word is interpreted.

RecursiveTranslation wraps the existing Translation sum, adding only the P1
failure absent from the ordinary mapper. The default physical mask already
aligns a PT address, so the source's subsequent 4 KiB InvalidFrameAddress branch
is unreachable in this selected profile; no invented reachable case is claimed.
No sum layout is asserted to be a hardware ABI.

## Captures, effects and authority

Route input words are the selected P4/P3/P2/P1 entries at the page's four indices.
Numeric physical table IDs must match the preceding link when an existing child
is traversed. CaptureMismatch is an added consistency failure, preserving earlier
writes; equality itself establishes no snapshot provenance or freshness. New
zero requests update the next ID from the resulting link, including source raw
flag address contamination. Unvisited values remain untouched input snapshots,
not additional proposed effects. The generic translation helper takes selected
words directly and does not independently establish table identity.

The caller supplies initialized, consistent snapshots and allocation observations
for simulation. These are ordinary copyable numbers and plans, not allocated
frames, recursive aliases, loaded CR3, usable page tables or current CPU facts.
The coordinate helper does not grant access either. Applying any proposal later
requires actual backing/custody, alias and lifetime checks, instruction authority
and appropriate invalidation settlement. This API constructs no allocator,
deallocator, flush receipt, ignore capability or live mapping reference.

Page/size/frame geometry is checked before route work. Malformed allocation
addresses become InvalidSuppliedFrame instead of relying on upstream typed
construction. The selected profile is canonical48/default physical52. Memory
encryption, LA57 and admission of the recursive topology are separate work.

## Pinned-body reference and test boundaries

The [inventory](recursive-routes-inventory.json) binds the complete 40-anchor
source file: 24 route/child/translation components and 16 outside-slice entries.
It does not mark constructors, pointer wrappers, coordinates or cleanup complete
because this slice does not implement them.

The reference generator copies the 21 size-specific method bodies, private
child-creation body and generic translation body into free functions. Their
branch conditions, setter calls, typed errors and zeroing order remain extracted
from the pin. These are **adapted source-body mirrors**, not invocations through
an actual RecursivePageTable. Specific adaptations are recorded in the manifest:

- Supplied disjoint borrowed tables replace self.p4 and recursive pointer
  resolutions. Coordinate-valued locals become the corresponding supplied
  table reference; coordinate arithmetic is tested in the separate topology slice.
- Instrumented entry/table aliases delegate numeric methods to actual pinned PTE
  APIs and complete clearing to actual PageTable::zero. Counters record setters,
  full-table clears and the deepest accessed selected table. Initialization and
  final inspection do not increment those counters.
- A safe numeric observation queue replaces the unsafe FrameAllocator trait.
  No backing or uniqueness guarantee is fabricated. Explicit type annotation
  restores inference formerly supplied by p1_ptr; imports use the pinned crate.

No raw dereference, RecursivePageTable constructor, CPU read or hardware operation
runs. Public Rust flush values returned by the copied methods are inert test
results over never-installed snapshots; they are not Cathedral receipts.

The reference has 232 original route scenarios and 90 generic translations.
It records all selected words, setter/zero masks, allocation attempts, access
depth, return/error payloads and caught setter/translation panics. Non-selected
nonzero sentinels prove full-table zeroing beyond the selected entry. Cases
cover all three sizes, all parent depths, four leaf operations, all nine
size/parent-level combinations, allocation exhaustion, zero parent flags,
non-present and huge ancestors, partial suffix creation and writes retained on
failure. Translation covers low/high/final addresses, every stopping level,
non-present chains, huge truncation and both panic cases.

Omega evaluates those observations through actual library bodies in 39 route
batches and three translation batches. Added checks cover malformed page and
allocation inputs, capture mismatch after a parent write and a noncanonical
virtual address. Nine body mutations change mandatory flags, retained writes,
non-present/huge traversal, allocation count, translation result and capture
failure effects. Each must compute failure under an unchanged success contract.
ID bookkeeping and local result payloads not returned by a Rust flush value
are explicitly checked projections of the observed entry/clear trace, rather
than claims about physical snapshot provenance.

Compiler: Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`, SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
The complete canonical command passed: route and translation producers checked
16 and 14 sources; all 39 route batches checked 18 sources each, all three
translation batches checked 16 each, and the additional-input fixture checked
20. All nine mutations computed failure (`1 == 0`) under the unchanged success
contract. Fresh Rust observations and all five source/fixture/inventory
generators passed in that same run. Python syntax, JSON, trailing whitespace
and local Markdown targets were checked across all 67 owned files.
No native Omega execution, ABI measurement, live topology or production
integration is claimed.

```sh
python3 tools/ports/x86_64-recursive-routes/check.py --omega /path/to/omega
python3 tools/ports/x86_64-recursive-routes/check.py --host-only
```

`--positive-only`, `--controls-only` and `--route-batch N` select bounded checks
while retaining source-generator/inventory/fresh-reference verification. Review
all source branches, adapter substitutions and capture policies before changing
the pin or regenerating observations.
