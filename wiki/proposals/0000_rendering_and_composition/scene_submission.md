# Scene submission and nested Matrices

A retained scene describes drawing work and references resources. It does not
require a private pixel image for each scene node or Matrix.

Status: proposed behavior under [rendering and composition](../0000_rendering_and_composition.md).
This page owns submission and nesting. Resource lifetime belongs to
[resource custody](resource_custody.md); target execution belongs to
[execution and presentation](execution_and_presentation.md).

## Drawing vocabulary

A small drawing vocabulary can serve ordinary UI without owning its interaction
model. Vello's [scene API](https://docs.rs/vello/latest/vello/struct.Scene.html)
provides a useful comparison: paths, glyphs, images, clips and layers.

| Candidate operation | Meaning and limits to specify |
| --- | --- |
| Fill rectangle or rounded rectangle | Geometry, brush, corner rules and antialiasing |
| Fill or stroke path | Bounded curves, winding rule, stroke geometry and supported brushes |
| Draw positioned glyph run | Font/version reference, glyph indices, positions and paint |
| Draw retained image | Resource/version reference, source rectangle, transform, sampling and color interpretation |
| Place child scene | Authorized scene reference, transform, clip and ordering |
| Group content | Defined opacity/blending; intermediate storage only when semantics or backend require it |

This is a candidate vocabulary, not an opcode list. Initial scope can omit
gradients, arbitrary path clips or effects while defining a compatible way to
report unsupported operations. Full SVG parsing, scripting, arbitrary shaders
and every blend/filter mode are separate commitments.

Layout, styling, widgets, text editing and input behavior remain outside the
drawing vocabulary. A copied control message or a shared command batch can carry
the same description. IPC transport selection does not change drawing authority.

## Retention and updates

Retained descriptions and retained bitmaps have separate lifetimes. A scene may
hold a resource identity without keeping its decoded pixels resident. A retained
path can avoid repeated command submission while still needing raster work when
the output changes.

The proposed update unit is a batch against a named scene revision. The receiver
validates authority, geometry, resource generations and budget before publishing
the next accepted revision. A failed batch leaves the prior accepted revision
intact. A stale base revision produces an explicit failure rather than silently
applying changes to different state. Names here describe behavior, not wire fields.

Scene acceptance, resource readiness, rendering completion and presentation are
distinct events. A metadata acknowledgement does not mean pixels are visible or
that the producer may reuse backing. The resource and execution pages own those
later transitions.

Backpressure may coalesce superseded complete snapshots before execution.
Dependent deltas cannot be dropped as if they were independent frames. Unchanged
scenes do not require resubmission each refresh. Damage tracking and occlusion
can reduce work, but neither is a guarantee that a small message renders cheaply.

## Authority through the host chain

Each Matrix controls its delegated subtree. A child cannot name a sibling's
resource, change an ancestor's clip, or promote itself into a protected layer.
The root derives identity from authority, never from a caller-supplied label.
Resource identifiers are not access grants; importing a resource requires the
appropriate delegation described in [resource custody](resource_custody.md).

For flattening, hosts forward authorized scene/resource references together with
placement constraints. The platform accumulates transforms and intersects clips
before generating drawing work. A host may restrict the child further. A deeper
host cannot enlarge the authority granted above it. Finite numeric values,
transform overflow, scene cycles, depth and node counts need bounded validation.

For example, a game inside a launcher inside a desktop Matrix can contribute one
image reference. The launcher adds a border and clip. The desktop places that
subtree. The renderer can sample the game's image directly into the final target
alongside both sets of decorations. Neither ancestor needs its own subtree image
merely to position it.

Flattening requires the host's participation. A custom host may intercept,
replace, hide or render its subtree to an image before forwarding it. A foreign
guest may expose only a framebuffer. An effect over an entire subtree may need
an intermediate result. Those are reasons to materialize content; nesting alone
is not. Forwarding does not give a child a new route around its host.

The existing design gives hosts observation of their children's output. A
flattened route needs to preserve that authority without granting unrelated
output access. Readback or subtree capture may do additional work when exercised;
continuous eager copying is not required just to keep the authority available.

## Trusted callers and shared targets

Trusted sub-compositors can use the same submission interface as untrusted ones.
The final renderer can remain the sole writer to its target. A trusted caller
does not need a writable target mapping merely to contribute layout or drawing.

If a selected renderer implementation delegates direct writes internally, its
scope and enforcement remain visible in the trust model. Rust row borrows provide
checked access for cooperating code. A process name or package location does not
turn those borrows into native-code containment.

## Text commands and legibility

A glyph command draws already selected and positioned glyphs. Text shaping,
paragraph layout, fallback selection and locale handling can remain in reusable
libraries or separately chosen providers. [HarfBuzz](https://harfbuzz.github.io/what-is-harfbuzz.html)
illustrates the separation between shaping and rasterization.

The linked [font-rendering experiment](https://mccloskeybr.com/articles/font_rendering.html)
explores TrueType parsing and an SDF atlas for Latin alphanumerics. It does not
establish complete text shaping, fallback, emoji or small-size quality requirements
for Cathedral. CPU and GPU backends need a tested text-quality target before a
rasterization technique is selected.

Drawing data and accessibility annotations remain different authorities. A glyph
run does not automatically grant another client the original string, offscreen
document or hit-testing model. Existing annotation policy stays with
[[windowing_and_compositor]] and [[media_and_graphics]]. Whether the platform
accepts glyph commands at all is an open owner choice, linked from the
[evidence page](evidence_and_acceptance.md#design-conflicts).
