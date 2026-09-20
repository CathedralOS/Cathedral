# Detached Mapper conveniences

## Scope and status

Current stage: **tested** by pinned Rust witnesses and Omega semantic evaluation.

This bounded slice implements 11 pure machines for default map requests,
identity mapping, MappedFrame size views and translation conveniences. Existing
`mapping_routes::RoutePlan`, `translation::Translation`, `NumberResult` and
page/address helpers remain canonical. There is no live Mapper trait facade,
flush promise, allocator grant, backing object or custody constructor.

Pinned source: x86_64 `cc35c876d3badb57df54a66e22f7768a52be95f2`,
`src/structures/paging/mapper/mod.rs`, MIT OR Apache-2.0. See
[retained upstream licenses](../../../licenses/rust-osdev/x86_64/).
The existing Omega data/case and machine contracts were consulted; frame-size
alternatives use semantic cases and have no requested native layout.

## Preserved behavior

`default_map_request` uses the existing PRESENT/WRITABLE/USER_ACCESSIBLE subset
helper. It preserves all original leaf flag bits. `default_map` passes that
request to the existing route planner and returns its outcome and edits intact,
including failures after partial proposed parent edits. No extra rollback,
allocation, frame validation policy or leaf implementation is substituted.

`identity_page` checks physical frame geometry and strict virtual canonicality
before `identity_map` delegates to default mapping. At this pin,
`VirtAddr::try_new` requires the original address to equal its sign extension.
Thus a zero-extended bit-47 physical frame causes Rust's default identity map to
panic before invoking the underlying mapping method. An obsolete upstream
error comment still describes automatic sign extension; the implementation and
actual witnesses take precedence. Omega returns InvalidInput with no edits for
this raw-input rejection. No new mapping is inferred from numeric equality.

`MappedFrame` is a numeric view with 4 KiB, 2 MiB and 1 GiB cases. The checked
constructor reuses existing physical frame geometry. Start/size projections
match the upstream methods; their numbers are not allocated frames. A raw case
can be constructed independently, so projections are views rather than
validation or custody operations. Conversion from existing Translation checks
the numeric frame/size before creating a view.

The public Rust TranslateResult permits an offset outside its frame size.
`Translate::translate_addr` simply adds that offset to a valid typed frame.
`translated_address_pinned` preserves this behavior over the existing numeric
Translation shape, validates representable typed-frame geometry, and returns
Rejected on physical or arithmetic overflow. It does not establish that the
result belongs to the mapped frame. Existing `translation::translated_address`
keeps its stricter within-frame offset check. The captured-word wrapper reuses
that helper because captured translation produces offsets within the selected
frame. No second Translation enum or generic InvalidFrameAddress payload is
introduced; nonmapped outcomes project to Rejected.

`translate_page` delegates the size-specific captured route with no allocation
observations. The shared planner retains all parent/leaf errors and validates
captured table IDs. Numeric ID equality still does not prove snapshot provenance,
physical storage, freshness or unique ownership.

## Source inventory and authority boundaries

[mapper-conveniences-inventory.json](mapper-conveniences-inventory.json) records
41 lexical anchors in the complete source file: 10 translated in this bounded
overlay and 31 deliberately outside it. Underlying map/unmap/update/translate
work remains in the existing route/plan modules; cleanup belongs to its own
slice. MappedFrame's three implicit variants are recorded explicitly.

MapperFlush/MapperFlushAll construction, ignore and live flush remain excluded
from the Omega API. A route's numeric success or error does not settle a later
owner's invalidation obligation. This slice changes no route, cleanup, core,
PTE or prior-port implementation.

## Actual pinned witnesses

The Rust recording Mapper executes the original trait defaults, not copied
method bodies. It observes 54 map/identity delegations across three sizes and
nine flag sets, plus three identity failures before delegation. The recording
implementation returns an error after capturing arguments and never creates a
flush token or calls an allocator.

Actual MappedFrame getters supply six projection assertions. A recording
Translate implementation executes 15 original translate_addr calls: offsets
0, size-1, size and size+1 for every size, two unmapped/error outcomes and a
physical-overflow panic. The out-of-frame cases distinguish the exact default
from the existing stricter captured-result helper.

Four additional witnesses execute actual MappedPageTable operations using the
existing reviewed test setup: stable, distinct owned PageTables with UnsafeCell,
registered numeric IDs and no installed translation root. They cover three
sizes of default/identity mapping and one size-specific translation. Only this
isolated reference harness discards Rust flush tokens. Its observations include
all selected words, allocation calls and cleared child-table sentinels.

## Omega checks

The scalar fixture executes 27 default requests, three frame/identity groups,
12 translation offsets, malformed inputs and captured-word projections. Two
separate route fixtures check all three mapping sizes, selected page indices,
parent/leaf words, write/zero masks, allocation counts, read-only translation
and identity rejection without effects. Four body mutations change the expected
parent mask, out-of-frame physical sum, identity rejection and route write mask;
the unchanged success contract must reject the evaluated failure value.

No native layout or hardware execution claim is needed for these semantic
wrappers. The tests use Omega revision
`eaa7993a23623cd8fabf45350340479c5c9c7879`, binary SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.

```sh
python3 tools/ports/x86_64-mapper-conveniences/check.py --omega /path/to/omega
python3 tools/ports/x86_64-mapper-conveniences/check.py --host-only
```

`--controls-only` runs the reference/source audits and body controls when the
positive producer and three fixtures have already passed separately. Route
checks are kept in small groups because their proof/evaluation is more costly
than the value-only fixture. This is test organization, not a port blocker.

Observed results: producer 17 sources and all three positive fixtures 18 sources
each pass; the controls-only runner passes all four body mutations and actual
Rust witnesses. Source inventory/generator freshness and whitespace checks pass.
