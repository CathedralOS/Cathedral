# Owned matrix surfaces

Start at [the request coordinator](../../service/requests/surfaces.rs): authenticate
through the child's installed connection, validate its allocation request, acquire
upstream backing, then publish a fresh local name. [The world model](../surface.rs)
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
16 KiB user stack. There is no per-frame heap allocation.

## Evidence and limits

[Host tests](tests.rs) cover conservation, exhaustion, independent name sequences,
stale handles, clipping, retained moves, teardown and three-level stacking ranges.
The desktop [app lifecycle](../../../../../distribution/lab/matrix/desktop/app/surfaces.rs)
fills its allowance, checks exhaustion, releases and reuses objects, and rejects
stale names. [Desktop probes](../../../../../distribution/lab/matrix/desktop/probes.rs)
check real pixels after removing one surface, restoring it, moving a retained
subtree, and overlapping it with an outside sibling. The same checks run under
CPU pressure and after repeated dynamic subtree replacement.

This is a rectangle-backed surface-object lab, not a complete compositor API.
There are still two direct children per matrix, sixteen root primitives, and eight
process accounts. Text/images, shared bitmap storage, active quota redistribution,
transactional multi-object presentation and root-provider recovery remain separate
work. The prior fixed six path keys and two-level rendering encoding are gone;
recursion is limited by supplied budgets and the lab's process arena.
