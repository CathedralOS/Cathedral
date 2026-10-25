# Derived rendering resources and residency policy

The accepted [resource contract](../../spec/human_surface/rendering_resources.md)
owns image identity/access, local sharing, remote uploads, retirement and safe
reclamation. The [presentation contract](../../spec/human_surface/rendering.md)
owns demand and backend boundaries. This proposal now owns the remaining cache,
acquisition and sponsorship alternatives; it does not replace those accepted rules.

## Demand acquisition alternatives

A registered resource need not immediately decode or pin every byte. A renderer
could asynchronously acquire a supported version, region, tile or resolution
through its supplied provider. A compatible local provider could supply a backing
lease; another could upload a representation. Exact acquisition/residency promises
remain to be chosen.

The accepted [missing-resource rule](../../spec/human_surface/rendering_resources.md#unavailable-resources-and-submission-results)
skips unavailable draws, continues unrelated rendering and reports misses against
the exact submission. Slow providers do not indefinitely block final drawing.
Hosts may explicitly supply authorized fallback content, but the renderer does not
silently substitute it. Active acquisitions retain backing through completed use.
Remote silence is not confirmed death; available local copies retain their own
authority and lifetime.

## Representations and text

Importing compatible backing can avoid a transport copy. It cannot promise that
the renderer never allocates another representation. Encoded images may need
decoding, a device may require another image layout, and text may use a glyph
atlas. Those allocations are visible derived resources.

Static images reuse a registered version. Animated images publish new frame
versions and may use a bounded pool whose slots become writable only after
release. Image/file decoders need not run inside the final target writer.

Font source bytes, shaped glyph runs and rasterized glyphs have different
identities and lifetimes. A glyph cache key includes font version and rendering
parameters that affect its pixels. Stable font identity alone is insufficient.
Positioned runs can be retained independently of glyph-atlas residency.
Changing atlas placement is internal bookkeeping, not an app-visible raw pointer.

A shared platform font cache can have a platform budget. Private fonts and image
caches require authorized sharing and a selected charging policy. The proposal
does not assume every derived byte stays in the producer's allocation, nor that
all deduplicated data is free to retain.

## Bounded demand without tiny universal limits

Budgets cover different sources of pressure:

| Budget | What remains charged |
| --- | --- |
| Backing storage | Physical allocation until actual reclamation |
| Metadata | Registered resources, versions, scene nodes and queued batches |
| Active imports/pins | Storage kept available by a consumer or device lease |
| Derived caches | Decoded images, glyph atlases, GPU conversions and intermediates |
| Work and submissions | Decode/upload/render demand and outstanding asynchronous operations |

Physical backing is charged once to an identified account. Importers also reserve
a bounded allowance for how much backing they can keep pinned; that is a separate
admission limit, not a second physical allocation. Sharing or forwarding through
Matrices cannot erase the backing's payer or the importer's responsibility.
References must not let an exited process reset a still-live allocation's bill.

The proposed policy is negotiated grants with parent-budget conservation. An
editor or game can receive much larger grants than a small status app. Exhaustion
produces backpressure or a defined rejection rather than an unbounded queue.
Admission considers the whole batch before exposing it as accepted work.

Evicting unused derived caches and releasing inactive imports can reduce pressure.
Active readers or device work cannot be discarded to satisfy an accounting
counter. Shared cache costs and sponsor liability after producer failure need the
[shared-cache decision](../../../OWNER_QUESTIONS.md#shared-render-cache-sponsorship-and-stronger-residency-guarantees)
before this becomes a platform guarantee.
