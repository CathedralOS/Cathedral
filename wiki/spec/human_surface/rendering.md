# Placement, rendering demand and presentation

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-10-23 |

## Scope and ownership

The owner accepted these boundaries on 2026-10-23. They establish presentation
policy and backend independence, not a production drawing vocabulary or GPU ABI.
The [focus](focus_routes.md), [input](input.md) and
[IPC custody](../communication/ipc.md) contracts remain their respective owners.

A program's execution, its presentations and their placement are distinct.
A presentation is content arranged within an owner's environment. A view is an
arrangement of presentations; it need not currently be on a physical display.
Another desktop, offscreen workspace or nested layout can supply placement.
This does not require a desktop abstraction in every distribution.

The enclosing matrix owns placement policy. Its layout manager or chrome can
implement docking, tabs, desktops or floating windows over authorized
presentations. The compositor enforces granted geometry and output authority.
These responsibilities need not be separate processes.

Apps produce content for assigned local dimensions and may report layout
preferences. Global desktop position, sibling layouts and physical display
topology are not required app knowledge. Moving a presentation between
arrangements does not reparent its process or change its storage/sandbox authority.

## Owner-managed placement

Each presentation has a placement owner; other participants request changes
through granted interfaces. Apps must not need updates for each layout feature.
Owner-provided decorations can initiate movement directly. App-drawn decorations
may request owner-managed movement/resizing tied to an authorized interaction.
The owner interprets that intent as floating, docking, reordering, moving between
arrangements or refusal under its policy.

Selecting an input route does not implicitly raise, reveal, resize or move a
presentation. Chrome can combine separately authorized operations into familiar
actions. Agents follow the same rules; shared presentation changes affect others.

Additional consumers of an image do not create independent app state. A second
independently sized or navigable presentation depends on app support; it is not
mandatory and does not imply independent document state. An agent's private
focus route or offscreen view does not isolate changes to a shared app.

## Rendering demand and execution

Minimize/restore are layout or distribution operations, not mandatory universal
program states. Hiding a presentation may remove display demand while retaining
placement. Inactive tabs, offscreen arrangements and mobile navigation can
likewise change demand without implementing a desktop minimize operation.

The producer is informed of resulting demand for its presentation. No new frames
requested allows it to stop unnecessary rendering; it does not suspend execution,
withdraw input authority or prohibit publishing input acceptance and performing
authorized work. Execution remains budgeted. Losing focus alone does not remove
rendering demand. This does not grant click-through access to hidden content.

Authorized previews, capture or other visible presentations can still require
pixels. Demand need not be binary; consumers may need different update rates.
Negotiation, scheduling and supported sizes remain implementation work, without
unlimited production or simultaneous multi-size guarantees.

A scoped read-only shared status mapping is a supported option for dimensions,
current configuration and rendering demand, with change notifications. An app
sees its own authorized state, not a fullscreen PID or global sibling registry.
Publication must be coherent; notification alone is not the state. Status
visibility does not establish image completion or custody. The compositor may
bound or decline unwanted submissions; a hint cannot stop a program spending
its own execution budget.

## Capture and cursor presentations

An authorized consumer may request fresh pixels without inherently restoring,
raising or relocating a presentation. This creates bounded rendering demand,
subject to provider support and grants, not immediate completion. Capture remains
within authorized output. Capture-mask and privacy protocols are not selected here.

Agents may use accepted semantic operations or supplied input routes. Ordinary
visible pointer hit-testing still respects occlusion; screenshot access does not
grant permission to click through another app. Coordinate actions need a defined
relationship to layout state; exact stale-layout handling remains open.

Cursor renderers can contribute bounded surfaces/assets, positions and hotspots
through scoped compositor interfaces. Drawing a cursor does not require writable
access to the entire output or confer input authority. Placement, clipping and
stacking remain enforced by the owner; overlays above unrelated presentations
need authority. Recursive visual handoff, attribution and hidden-target indicators
remain separate choices.

## Scoped fullscreen

Fullscreen requests the presentation owner's available content area. The owner
may grant it locally and optionally request fullscreen for its own presentation
from its parent. Each ancestor applies its policy. If propagation stops, the child
is fullscreen only within the granted area. Forwarding through a sibling compositor
service requires supplied authority; process-tree adjacency grants nothing.

The owner can hide/exclude competing presentations and update their individual
demand. Independent capture or other consumers can still require frames.
Fullscreen does not expose competing identities, transfer unrestricted buffer
or device access, or defeat enclosing supervision and protected recovery.
The owner may withdraw it or present authorized overlays.

## Image forwarding and backend execution

The architecture must permit authorized image/scene references with placement and
clipping through participating matrices without requiring a pixel image at every
ancestor. This is not a universal zero-copy guarantee. Custom hosts may intercept
or render their subtree; effects or incompatible representations may require
intermediates. Forwarding cannot bypass host authority.

Software composition, GPU composition, hardware display planes and direct scanout
are execution choices beneath the presentation contract. Fullscreen permits
considering direct scanout; it does not guarantee compatible formats, effects,
capture behavior or device support. Cursor presentations need not each consume a
hardware cursor plane; general composition remains a fallback.

Compatible image sharing can avoid transport copies while final composition still
reads and writes pixels. The protocol must not require CPU readback or a blocking
CPU completion wait at every ancestor. Authorized completion dependencies can
accompany references; concrete device synchronization remains unspecified.

Submission, render completion, presentation and safe reuse are distinct. Published
images require enforceable custody covering writable aliases and device access.
Backing remains alive until readers and device users finish. A status flag or
cooperative fence promise cannot contain a hostile writer. Spatial isolation,
including whole-page leases, does not establish temporal custody.

## Evidence and limits

The [Rust session experiment](../../../source-rs/distribution/lab/compositor/SESSION.md)
exercises root-resolved placement/clipping across processes without ancestor
pixel buffers. The [rendering investigation](../../../source-rs/distribution/lab/rendering/README.md)
compares flattened leaves and ancestor copies. These are partial evidence, not
general views, demand negotiation, capture, fullscreen propagation or GPU support.

This acceptance adds no runtime changes or measurements. Drawing/text placement,
resource residency/sponsorship, direct-writer admission, scene commits, capture
masks, exact view APIs and device-failure recovery remain outside this contract.
Alternatives remain in the [rendering proposal](../../proposals/0000_rendering_and_composition.md).
