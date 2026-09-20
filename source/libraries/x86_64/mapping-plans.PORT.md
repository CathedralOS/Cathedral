# Detached child and leaf mapping decisions

This bounded X86-001/002 slice extracts `create_next_table` decisions and the
leaf components of map, unmap, flag update and size-specific translation from
x86_64 `cc35c876d3badb57df54a66e22f7768a52be95f2`,
`src/structures/paging/mapper/{mapped_page_table,mod}.rs`. Modified derivatives
retain [MIT OR Apache-2.0 notices](../../../licenses/rust-osdev/x86_64/).
Status: tested. All 112 pinned Rust scenarios and matching Omega decisions,
nine additional assertions and four body-mutating controls pass.

The package uses ordinary semantic cases. `FrameSupply` is an explicit numeric
observation, and `ChildPlan`/`LeafPlan` combine a result case with the retained
word and whether the pinned operation would assign it. They are not raw ABI
unions, frame grants, reference constructors or live mapping objects. Omega's
mixed field/case data, ownership and layout contracts and the pinned operation
bodies were consulted before implementation. Existing [PTE codecs](page-entries.PORT.md),
[captured translation](tables.PORT.md) and core admission remain canonical.

## Preserved decisions

`prepare_child` retains the original order. An unused entry first consumes the
supplied allocation outcome. A missing frame reports allocation failure before
the supplied flags are examined. A supplied frame must fit the 52-bit aligned
profile; the extra raw-input check replaces Rust's established PhysFrame input.
HUGE_PAGE on that new entry is rejected before assignment. Otherwise the word
is assigned, then checked for PRESENT/huge state. Missing PRESENT represents the
pin's subsequent panic explicitly, including the word already written.

Existing nonzero entries can have inserted flags ORed into them before a later
huge-parent failure. The result preserves that assignment and the changed word;
it never describes failure as automatically rolled back. Raw inserted flags can
contain address bits, so the pin can also alter the child address numerically.
Ready reports the resulting numeric address and whether a newly supplied child
must be zeroed. The flag is a request, not proof that any storage was cleared.

Leaf mapping requires an aligned supplied frame for the selected 4 KiB, 2 MiB or
1 GiB size. A nonzero old word is AlreadyMapped, even if PRESENT is clear; the
retained error payload is the requested frame, matching Rust. That check precedes
the 4 KiB HUGE_PAGE assertion. Huge-page mapping inserts bit 7, while a 4 KiB map
rejects it. Flags can contain address bits exactly as the pin's raw bitflags do.

Unmapping requires PRESENT and the pin's size-specific huge-bit role, then checks
frame alignment before requesting a zero word. A huge-size word with HUGE_PAGE
clear retains the pin's `ParentEntryHugePage` category despite its counterintuitive
name. Flag update and size-specific translation instead require only a nonzero
leaf; they do not independently require PRESENT. Updates insert HUGE_PAGE for
huge sizes and can remove PRESENT. These are extracted numeric rules, not a new
hardware or reserved-bit admission policy.

`default_parent_flags` preserves the generic Mapper wrapper's PRESENT/WRITABLE/
USER_ACCESSIBLE subset. Other flags are neither inferred nor silently added.
No supplied numeric success demonstrates frame uniqueness, an installed root,
correct physical storage, an alias-safe hierarchy, CPU permissions or access.

## Verification and incomplete orchestration

[mapping-plans-inventory.json](mapping-plans-inventory.json) binds both complete
source files and 88 lexical anchors. One extracted helper is translated, eight
Rust scaffolding anchors are deliberately omitted, and 79 remain pending.
Pending map/unmap/update/translate methods link their tested leaf component
explicitly; **the full methods are not marked complete**. Ordered composition
across all parent levels, allocation custody, full mutation plans, cleanup and
invalidation settlement still require implementation and integration work.

The reference harness executes 99 actual pinned leaf operations through public
Mapper APIs and 13 actual child-creation paths through one-level huge-page maps.
Owned stable PageTable allocations use UnsafeCell for mapper-authorized interior
mutation, distinct registered IDs and no concurrent aliases. The hierarchy is
never installed on a CPU. Only this isolated test discards Rust flush tokens:
there was no active hardware mapping to invalidate. The Omega API exposes no
flush-discard capability and cannot settle a later owner's invalidation duty.

Reference observations include words left behind after caught panics/errors,
allocation-before-flag error ordering, previously non-present parents, address
bit contamination, and actual clearing of a supplied child's dirty last slot.
The model's `write` request is checked from reviewed source branch conditions;
ordinary Rust memory observations do not measure redundant assignment events.

Omega fixtures evaluate those same 112 decisions plus malformed input and parent
flag cases in bounded constant groups. Four mutations change actual expected
behavior: zero-child request, word retained on huge-parent failure, flag-update
write and parent-flag subset. Each must compute failure under the unchanged
success contract. No expected vector is native ABI evidence.

Compiler revision: `eaa7993a23623cd8fabf45350340479c5c9c7879`.
Binary SHA-256: `2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
No native Omega execution, live hierarchy mutation, TLB operation or production
build integration is claimed.

```sh
python3 tools/ports/x86_64-mapping-plans/check.py --omega /path/to/omega
python3 tools/ports/x86_64-mapping-plans/check.py --host-only
```

Pin updates must review error order, partial mutations and default-parent flag
selection before regenerating witnesses or mapping dispositions.
