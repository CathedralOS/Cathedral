# Owned matrix surfaces

Start at [surface.rs](surface.rs): conserve rank ranges, validate local names,
clip retained primitives, then retire revoked objects. The stock distribution's
[request coordinator](../../../../../distribution/matrix/service/requests/surfaces.rs)
authenticates the installed connection, acquires upstream backing, then publishes
a fresh local name. Its
[root adapter](../../../../../distribution/matrix/service/parent/rendering.rs) maps its private ranks into the
existing compositor's bounded [tile array](../../../../services/display/composition/scene/tiles.rs).
No ancestor image buffer or process identity is exposed to a child.

## Capacity and ownership

The current compositor has 64 backed primitive slots. Owner-only `SURFACE_LIMIT` subdivides
that allowance between direct children; their total cannot exceed the parent's.
A nested matrix learns only its own allowance through `POLL`, and applies the same
subdivision rule. Counts do not mint extra capacity at each level. A changed
allowance is refused while that child has live objects; revocation retires them
and returns its allowance. Revoked/replaced descendants require explicit regrant.

`SURFACE_CREATE(epoch, rank)` asks for one local stacking rank inside that allowance.
The provider validates the rank and reserves an upstream object before returning
a fresh child-local surface handle. Failed upstream allocation installs no local
object. A rank identifies a requested order, not an authority or global tile.
`SURFACE_INFO` reports only that child's allowance and allocated count. The same
integer handle in two children can name two unrelated objects.

`PAINT(epoch, handle, x, y, width, height, rgb)` replaces the object's retained
rectangle. Every matrix validates the binding and name, clips to the local extent,
and translates exactly one level. The app supplies local content; its matrix
controls placement, clip and the subtree's stacking range. Content replacement
does not raise a surface above another subtree. The shared `surface_range` API
accepts an owner-selected disjoint range; `surface_limit` finds free space when
allocation order suffices. Neither relocates live sibling ranks. The stock matrix
places first-child ranks at the bottom and second-child ranks at the top, regardless
of launch order. That distribution policy keeps every descendant below the later
outer sibling. Placement snapshots preserve the chosen range on replacement.

`ERASE(epoch, handle)` releases that object and its upstream backing. A hidden or
zero-sized rectangle remains allocated until explicitly released. Names never
repeat within a child slot, including connection replacement. Old handles cannot
address the next object reusing a rank. Upstream handles and rank offsets are
never returned to the child.

## Placement and lifetime

A same-extent move replays retained local geometry without asking the app to repaint.
A changed viewport/key binding invalidates its surface objects; the app reads the
new epoch and recreates them. Revocation, disconnect and subtree cancellation
retire the affected objects. Retirement is flushed through the same upstream
protocol before the next service wait. A parent that already revoked an object
may reject its subsequent cleanup request; other transport failures remain errors.
Retained state lives in process-private static storage; replay and retirement
process one object at a time. This keeps debug builds within the existing guarded
bounded user stack. There is no per-frame heap allocation.

## Retained bitmaps

`SURFACE_IMAGE(epoch, handle, x, y, width, height, stride)` carries one sealed
memory attachment. [Image requests](../../../../../distribution/matrix/service/requests/images.rs) validate
local authority and pixel bounds, preview clipping, forward to the parent, then
commit only after success. Pixels are little-endian RGBX u32; the top byte is
ignored. Positive-coordinate clipping preserves the original stride. Each matrix
retains an unmapped read right for later replay in [its parent adapter](../../../../../distribution/matrix/service/parent/images.rs).
Only [the root compositor](../../../../services/display/compositor/matrix.rs) maps the pixels.

All levels share the [same backing object](../../../../../kernel/core/user_tasks/memory/README.md);
there is no intermediate pixel image or child-to-parent pixel copy. Final software
composition still samples the bitmap and writes scanout. Replacing content or
erasing a surface releases its old read right. Subtree stop flushes ancestor
retirement before returning the child's funding. Failed validation preserves the
previous image. Transport failure is still an ambiguous commit, not a transaction.

Distribution policy explicitly stages private/reader quotas with owner-only
`RUN_PAGES` before dynamic admission. No executable kind implicitly grants memory.
The kernel reserves a reader slot and pages before queuing each attachment, so
both retained rights and an in-flight replacement need capacity. The desktop
uses one-page images and headroom for replacement; general image eviction,
streaming and active quota changes are not implemented.

## Bounded cursor presentation

`SURFACE_CURSOR(epoch, handle, x, y, width, height, rgb)` draws a built-in arrow
through an already-owned surface. Width/height clip its fixed 7-by-12 shape;
they cannot scale it into an arbitrary image. The empty pixels leave underlying
surfaces visible. Each matrix applies the same clipping, local handle validation
and translation as for a rectangle. The root retains a small primitive, without
allocating a bitmap or a full-screen cursor buffer. Changed position damages the
old and new bounds; unchanged content needs no repaint.

This is a presentation primitive, not an input grant or authenticated participant
badge. It neither selects a focus group nor bypasses ancestor stacking/visibility.
A routing owner must still authorize, fund and place its cursor presenter. Multiple
cursor surfaces consume the same conserving allowance as other content. A hidden
subtree hides its cursor surfaces, and disconnect/revocation retires them.
Same-extent movement replays retained cursor geometry without a child repaint.
The staged presentation protocol can atomically replace or move cursor surfaces;
the compact rectangle-only packet cannot encode one.

Host proofs check recursive clipping, rejected stale/oversized requests,
transparency, occlusion, damage and teardown. The integrated matrix desktop probe
checks actual pixels through two matrix hops, staging/commit, retained movement
and edge clipping. Default F5 still uses the existing physical cursor; a visible
synthetic participant and per-pointer cursor suppression/attribution remain work.

## Evidence and limits

[Host tests](tests.rs) cover conservation, exhaustion, independent name sequences,
stale handles, clipping, retained moves, teardown and three-level stacking ranges.
The desktop [app lifecycle](../../../../../distribution/applications/colors/session/surfaces.rs)
fills its allowance, checks exhaustion, releases and reuses objects, and rejects
stale names. [Desktop probes](../../../../../distribution/desktop/lab/probes.rs)
check real pixels after removing one surface, restoring it, moving a retained
subtree, and overlapping it with an outside sibling. The same checks run under
CPU pressure and after repeated dynamic subtree replacement.

This is a bounded rectangle/immutable-image/cursor lab, not a complete compositor API.
The stock executable has five direct child slots (chrome, three apps, background);
the shared `World<N>` selects its capacity independently. The lab has 64 root
primitives and 31 process accounts. Rich text, streaming image updates, active quota redistribution
remain separate work. The desktop now tests explicit root-tree recovery after provider loss. Bounded atomic presentation is
implemented through BEGIN/stage/COMMIT and the compact rectangle operation below.
The prior fixed six path keys and two-level rendering encoding are gone;
recursion is limited by supplied budgets and the lab's process arena.

## Publication scheduling

[Request validation](../../../../../distribution/matrix/service/requests/presentation.rs) previews every update
before [the parent publisher](../../../../../distribution/matrix/service/parent/presentation.rs) starts work.
The service resumes that publisher after each reply, answering other children's
local queries while dependent events wait in [bounded stream slots](../../../../../distribution/matrix/service/deferred.rs).
Placement, revocation and disconnect processing follow publication, so the validated
geometry cannot change halfway through a commit. Transport loss retires the matrix;
this is not a durable cross-provider commit/retry protocol.

`PRESENT_RECTS` uses one 64-byte packet for one or two existing rectangles. Names
remain full-width caller-local handles; coordinates use checked u16 fields and RGB
uses checked 24-bit color. Every hop clips and remaps names normally. The entire
packet is checked before staging; an invalid second rectangle cannot publish the
first. It shares the connection/view revision history with ordinary transactions.
Compact rejection leaves no active transaction. Sealed images, cursors, erases
and up to four-surface batches continue using the original staged protocol. Owner placement
replay is still individual updates, and scene atomicity does not promise vblank.
