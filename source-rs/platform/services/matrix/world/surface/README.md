# Owned matrix surfaces

Start at [the request coordinator](../../service/requests/surfaces.rs): authenticate
through the child's installed connection, validate its allocation request, acquire
upstream backing, then publish a fresh local name. [The world model](surface.rs)
owns quotas, names, retained primitives, clipping and retirement. The
[root adapter](../../service/parent/rendering.rs) maps its private ranks into the
existing compositor's bounded [tile array](../../../display/composition/scene/tiles.rs).
No ancestor image buffer or process identity is exposed to a child.

## Capacity and ownership

The root has sixteen backed primitive slots. Owner-only `SURFACE_LIMIT` subdivides
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
does not raise a surface above another subtree. First-child ranks grow from the
bottom of the parent's range; second-child ranks end at its top. Recursive ranges
therefore stay disjoint and every descendant remains below the later outer sibling.

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
memory attachment. [Image requests](../../service/requests/images.rs) validate
local authority and pixel bounds, preview clipping, forward to the parent, then
commit only after success. Pixels are little-endian RGBX u32; the top byte is
ignored. Positive-coordinate clipping preserves the original stride. Each matrix
retains an unmapped read right for later replay in [its parent adapter](../../service/parent/images.rs).
Only [the root compositor](../../../display/compositor/matrix.rs) maps the pixels.

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

## Evidence and limits

[Host tests](tests.rs) cover conservation, exhaustion, independent name sequences,
stale handles, clipping, retained moves, teardown and three-level stacking ranges.
The desktop [app lifecycle](../../../../../distribution/desktop/session/app/surfaces.rs)
fills its allowance, checks exhaustion, releases and reuses objects, and rejects
stale names. [Desktop probes](../../../../../distribution/desktop/session/probes.rs)
check real pixels after removing one surface, restoring it, moving a retained
subtree, and overlapping it with an outside sibling. The same checks run under
CPU pressure and after repeated dynamic subtree replacement.

This is a bounded rectangle/immutable-image lab, not a complete compositor API.
There are still two direct children per matrix, sixteen root primitives, and twelve
process accounts. Text, streaming image updates, active quota redistribution
remain separate work. The desktop now tests explicit root-tree recovery after provider loss. Bounded atomic presentation is
implemented through BEGIN/stage/COMMIT and the compact rectangle operation below.
The prior fixed six path keys and two-level rendering encoding are gone;
recursion is limited by supplied budgets and the lab's process arena.

## Publication scheduling

[Request validation](../../service/requests/presentation.rs) previews every update
before [the parent publisher](../../service/parent/presentation.rs) starts work.
The service resumes that publisher after each reply, answering other children's
local queries while dependent events wait in [bounded stream slots](../../service/deferred.rs).
Placement, revocation and disconnect processing follow publication, so the validated
geometry cannot change halfway through a commit. Transport loss retires the matrix;
this is not a durable cross-provider commit/retry protocol.

`PRESENT_RECTS` uses one 64-byte packet for one or two existing rectangles. Names
remain full-width caller-local handles; coordinates use checked u16 fields and RGB
uses checked 24-bit color. Every hop clips and remaps names normally. The entire
packet is checked before staging; an invalid second rectangle cannot publish the
first. It shares the connection/view revision history with ordinary transactions.
Compact rejection leaves no active transaction. Sealed images, erases and up to
four-surface batches continue using the original staged protocol. Owner placement
replay is still individual updates, and scene atomicity does not promise vblank.
