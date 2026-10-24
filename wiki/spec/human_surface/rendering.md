# Placement, rendering demand and presentation

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-10-24 |

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

## Hit-testing and displayed state

The owner accepted using the latest coherently accepted hit-test layout rather
than requiring exact agreement with the displayed frame. Each routing decision
uses a consistent local layout, including its transforms, bounds and applicable
hit regions. Once targeted, an event cannot silently move to a replacement
target. Apps validate actions against their current state.

Displayed pixels may lag accepted routing state. This is not guaranteed to be
only one frame under load or queued rendering. No display-frame barrier or
cross-matrix atomic scene commit is required by this baseline. Tying a screenshot
action to its captured layout and rejecting stale actions remains an optional
future facility, not an ordinary coordinate-input guarantee.

## Capture

An authorized consumer may request fresh pixels without inherently restoring,
raising or relocating a presentation. This creates bounded rendering demand,
subject to provider support and grants, not immediate completion. Capture is an
explicit capability to the supplied compositor, which serves a buffer containing
only authorized output with applicable masks already applied. A consumer must
not receive the unmasked original and instructions to ignore protected pixels.
The contract does not prescribe copying when backing can safely provide the same
guarantee. Mask format, publication and detailed privacy policy remain unspecified.

Agents may use accepted semantic operations or supplied input routes. Ordinary
visible pointer hit-testing still respects occlusion; screenshot access does not
grant permission to click through another app. Coordinate targeting follows the
accepted-layout rule above, not a guarantee that a prior screenshot is current.

Input acceptance metadata describes permitted interactions; it need not contain
passwords or document contents. Apps and their libraries must not publish values
they intend to keep private through that interface. Pixel masking does not erase
data an app independently exposes. This does not require a second general-purpose
confidentiality system for ordinary interaction descriptions.

## Cursor presentations

Cursor renderers can contribute bounded surfaces/assets, positions and hotspots
through scoped compositor interfaces. Drawing a cursor does not require writable
access to the entire output or confer input authority. Placement, clipping and
stacking remain enforced by the owner; overlays above unrelated presentations
need authority.

Cursor preference follows a particular pointer's hover/capture target, not the
keyboard focus route. That target can request an asset/style or no cursor. A
nested matrix resolves its internal target and propagates the resulting request
through its owner-controlled presentation interface. This is a composed pointer
target path, not a requirement for a separately registered cursor focus group.
Ancestors need not each render an additional cursor.

Requests are scoped to the relevant pointer and target lifetime. Leaving the
target, ending capture or losing the target restores the applicable owner policy;
stale suppression cannot hide unrelated pointers or protected outer UI. No-cursor
requests affect visual presentation only. Relative-motion locking, movement
confinement and drag capture require their own granted operations. Mandatory
attribution, protected indicators and hidden-target styling remain open.

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

The [resource contract](rendering_resources.md) owns authorized image versions,
leaf-funded local sharing, remote upload and retirement/reclamation boundaries.
An image reference does not require the renderer to map producer memory directly.

## Evidence and limits

The [Rust session experiment](../../../source-rs/distribution/lab/compositor/SESSION.md)
exercises root-resolved placement/clipping across processes without ancestor
pixel buffers. The [rendering investigation](../../../source-rs/distribution/lab/rendering/README.md)
compares flattened leaves and ancestor copies. These are partial evidence, not
general views, demand negotiation, capture, fullscreen propagation or GPU support.

The Rust [owned-surface path](../../../source-rs/platform/libraries/matrix/world/surface/README.md#bounded-cursor-presentation)
also has an experimental bounded cursor primitive. It preserves recursive
clipping, normal subtree stacking, visibility and retirement without a full-screen
cursor buffer. This is presentation evidence, not a complete cursor service:
the installed Split matrix demonstrates a scoped producer and owner-funded
cursor. The [pointer-target policy](../../../source-rs/platform/libraries/matrix/world/input/cursor/cursor.rs)
now accepts default/hidden preferences on delivered local pointer aliases.
Stock and Split hosts compose the effective preference up their supplied
interaction connections, independently of keyboard focus. Notes F6 demonstrates
this in the ordinary desktop. The desktop fixtures cover target exit without
further motion, nesting and independent physical/producer suppression; host tests
cover drag targeting, cancellation, geometry changes and stale bindings. Assets,
authenticated attribution, relative lock and confinement remain separate work.

The Rust [demand mechanism](../../../source-rs/platform/libraries/matrix/world/demand.rs)
now carries a child-local no-frames hint through `POLL`, composed across matrices
without replacing input or surface grants. Notes and Colors defer drawing while
continuing state work. The installed [Split demonstration](../../../source-rs/distribution/applications/split/README.md)
can hide its right note while an independent input participant edits it, then
restore its updated content. The desktop fixture checks actual hidden edits,
absence of child drawing submissions and cold-open saved data. This is bounded
display demand, not rate negotiation.

The installed Split capture provider separately adds bounded frame demand to its
Notes source, including while hidden. A supplied recipient requests one fixed
owner-masked crop, receives sealed output and presents it in its own rectangle.
The [capture integration](../../../source-rs/distribution/applications/split/session/capture/capture.rs)
uses the [platform lease/raster mechanisms](../../../source-rs/platform/libraries/matrix/capture/README.md).
This is leaf-source software evidence: it does not implement app-published masks,
recursive source readiness, arbitrary views, remote capture or GPU synchronization.

The accepted policy does not freeze that lab encoding. Drawing/text placement,
detailed cache residency/sponsorship, direct-writer admission, exact scene/mask
publication, view APIs and device-failure recovery remain outside this contract.
Alternatives remain in the [rendering proposal](../../proposals/0000_rendering_and_composition.md).
