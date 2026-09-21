# Explicit-profile leaf and mapper routes

Status: tested through actual Omega checked-interpreter execution. This slice composes
memory-encryption state with mapped and recursive leaf/child/route algorithms.
The default-profile APIs and all canonical request, capture, effect and result
representations remain unchanged. Generic translation, cleanup, frame ranges and
register observations have separate profile-composition evidence.

Modified algorithms come from x86_64
`cc35c876d3badb57df54a66e22f7768a52be95f2`, specifically the mapped and recursive
mapper methods and their private child/walker branches. Exact source hashes,
feature-dependent PTE/address operations, Omega data/copy/machine contracts and
the existing route capture policies were reviewed. Retained
[MIT OR Apache-2.0 notices](../../../licenses/rust-osdev/x86_64/) apply.

## Profile and algorithms

[encrypted_mapping_plans.omg](encrypted_mapping_plans.omg) reuses FrameSupply,
ChildPlan and LeafPlan. It accepts the existing copyable `memory_encryption::State`
by value. Pinned equivalence applies to states produced by initial followed by
successful configure calls. Editable or forged masks are ordinary caller data,
not an architectural or global-state invariant.

Frame admission uses the **current** encryption bit intersected with physical52,
then the requested alignment. PTE address/flag interpretation uses the separately
**accumulated** address mask. After configuring bit47 then bit48, bit47 is again
admissible in a newly supplied physical frame, but is still removed when reading
an entry address. The distinction also applies to an allocated child: its newly
captured identity follows the resulting entry address, not an assumption that
allocation input and decoded link remain equal.

Existing-child flag insertion tests the profile-decoded flags. An encryption bit
already present causes no redundant setter. A required insertion preserves the
raw OR behavior and occurs before parent classification; failures retain that
write. Newly allocated recursive links force PRESENT|WRITABLE, while mapped
links preserve the exact requested flags. An unavailable or malformed allocation
is handled at its consumption point, retaining earlier effects. HUGE in new
parent flags produces the explicit assertion-failure result before an entry
write. A mapped new link without PRESENT retains the entry and reports the
canonical MissingPresent case corresponding to the pinned panic.

Map leaves preserve raw flags, whole-word occupancy and AlreadyMapped's requested
frame payload. Unmap and size-specific translation use profile-decoded addresses
for alignment and returned frames. Updating leaf or parent flags replaces flags
around the accumulated-mask address; encryption flags disappear unless supplied
again. PRESENT/HUGE and whole-word-zero ordering remain the pinned operation's
ordering. No new result cases or duplicate nominal types are introduced.

[encrypted_mapping_routes.omg](encrypted_mapping_routes.omg) and
[encrypted_recursive_routes.omg](encrypted_recursive_routes.omg) retain their
respective finite parent traversal rules and share the profile-aware leaf/child
helpers. Their private state graphs carry plain mask values. The ordinary Request, CapturedPath, AllocationInputs,
Edits, Outcome and RoutePlan remain canonical. Recursive update/set-parent/
translate ancestors require whole-word nonzero; recursive map checks HUGE after
flag insertion without requiring PRESENT. Unmap and ordinary mapped traversal
retain PRESENT-before-HUGE checks.

Capture IDs use profile-decoded parent addresses. A mismatch preserves previous
writes. Raw flag bits can contaminate address positions: an existing-link change
can cause a mismatch after its setter, and a new link updates the next selected
ID from that changed address before clearing its selected word. Full child-clear
requests, redundant setter masks, allocation attempts and access depth remain
explicit numeric effects. Invalid requested frame/page geometry rejects before
route work; malformed allocation observations reject only when consumed.

These APIs grant no allocator/deallocator custody, actual table access, recursive
alias, current CPU state, live mapping, flush receipt or installation permission.
Applying proposals requires real backing, current observations, alias/lifetime
checks, publication and invalidation settlement. No instruction executes in Omega.

## Reference evidence and limits

The reference runs eleven isolated configurations: disabled; EncryptedBit at
0, 7, 12, 21, 30, 47 and 51; SharedBit47 and SharedBit63; and EncryptedBit47
followed by SharedBit48. Configuration precedes every typed address and table.
Low/flag-bit configurations test the source arithmetic envelope, not CPU admission.

There are 318 route scenarios, split evenly between mapped and recursive forms.
All 159 mapped scenarios also call actual public pinned MappedPageTable methods
over stable, distinct owned initialized tables through a checked numeric registry.
No table is installed as a CPU root. The witness compares actual final words,
allocation calls, complete-clear observations and return/error/panic payloads with
instrumented mirrors of those same source bodies.

The generator extracts all 42 size-specific method bodies, private child/walker
branches and mapped error conversions. Borrowed snapshots replace root and raw
pointer resolution. Instrumented entry methods execute actual pinned PTE methods;
full clearing executes actual PageTable::zero, including non-selected dirty
sentinels. The mirrors replace the unsafe allocator trait with a safe numeric
observation queue. Setter counts and deepest access come from these adapted
mirrors, not direct instrumentation of the public Rust object. No RecursivePageTable
is constructed; recursive evidence is explicitly an adapted private-body mirror.
No raw recursive dereference or hardware instruction runs.

Scenarios cover all three sizes, parent levels, both route rules, flag replacement,
profile-cleared alignment bits, existing encryption flags, full creation, partial
allocation failure, prior writes retained on failure, non-present/HUGE ancestors,
raw flag contamination, and old-bit frame admission after repeated configuration.
Flush values are discarded as inert test results over never-installed snapshots;
they are not Cathedral authority receipts. Result payloads not returned by a Rust
flush, and capture IDs, are additional projections of the observed trace.

Generated Omega fixtures call the actual library bodies after executing initial
and configure. Added policies cover invalid requested and allocated frames,
malformed geometry, capture mismatch after flag OR and newly contaminated child
identity. Body mutations change expected behavior under an unchanged success
contract. The [inventory](encrypted-mapping-inventory.json) binds both complete
mapper files and records the three address/PTE/encryption mask sources separately.
Native Omega execution, ABI measurement and production integration are unmeasured.

The complete [verification record](../../../tools/ports/x86_64-encrypted-mapping/checked-verification.json)
contains 56 passing positive fixture entries: all 318 routes in 54 batches, the
direct leaf/child unit, and five additional policies in one fixture. All ten
changed-body controls return 1 while their unchanged positives return 0, with no
interpreter errors. Exact authored dependency, fixture, generator, reference and
runner-source hashes were unchanged before and after execution. This is checked
interpreter evidence, not a claim that all 54 const-contract batches were rerun.
The direct leaf const fixture also passes; changing its expected frame computes
1 and is rejected by the unchanged success contract. Representative route const
batches passed during development; the final complete source closure is bound by
the checked execution record.

```sh
python3 tools/ports/x86_64-encrypted-mapping/check_interpreted.py
python3 tools/ports/x86_64-encrypted-mapping/check.py --host-only
python3 tools/ports/x86_64-encrypted-mapping/check.py --omega /path/to/omega --batch 0
```

`check.py` retains the complete const-contract route as an optional slower
check; `--batch N`, `--positive-only`, `--controls-only` and `--jobs` select its
bounded work. Both runners retain generator, inventory and fresh Rust checks.
The checked runner reuses the Cathedral ACPI execution harness at exact SHA-256
`e6d0aee6b4dddbbf34a60cffbe8f158643cc5c4d100f9e20f0481f75f52890be`;
its source and lockfile are bound by the record. Compiler: Omega
`eaa7993a23623cd8fabf45350340479c5c9c7879`, binary SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
