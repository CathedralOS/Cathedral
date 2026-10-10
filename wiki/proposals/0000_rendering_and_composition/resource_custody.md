# Resource backing, leases and accounting

The compositor can retain a resource's identity and read authority without
owning a duplicate of its backing. Storage, accounting and active use need
separate lifetimes.

Status: proposed behavior under [rendering and composition](../0000_rendering_and_composition.md).
This page owns resource identity, acquisition, release and charging. Scene
revision semantics belong to [scene submission](scene_submission.md).

## What ownership means

| Concept | Proposed responsibility |
| --- | --- |
| Resource identity | Opaque identity plus incarnation/version, kind and validated metadata |
| Access capability | Which principal may publish, import, read or delegate this resource |
| Backing custody | Shared allocation object or provider keeps actual storage alive |
| Account | App, Matrix sponsor or platform budget pays for backing and work |
| Read lease | Renderer holds access to a specific stable version until completion |
| Derived representation | Decoder/renderer owns an evictable representation with an identified payer |

An app's persistent realm may contain an encoded image or font. Its decoded RAM
allocation and GPU representation are distinct resources. Referring to all three
as "app storage" hides the decode, upload and residency costs.

App-funded backing should be a shareable allocation object with a lifetime
independent of a raw pointer into the app's heap. The kernel or resource provider
maintains its custody while readers remain. The compositor obtains only the
necessary read/import rights. A content hash can name immutable data, but knowing
the hash is not permission to read a private resource.

## Registration and demand acquisition

Registration advertises a resource version and enough metadata to validate its
use. It does not require immediate decoding or pinning of all registered bytes.
The receiver bounds registration metadata even when no backing is resident.

A ready resource can be offered with a read capability at submission. For a
nonresident resource, the renderer can asynchronously request the required
version, visible region or resolution through a granted provider endpoint.
That request grants no access to unrelated app storage. The provider can respond
with a shared backing lease rather than sending all pixel bytes in the reply.

The proposal keeps resource waits outside the final drawing pass. A slow or dead
provider must not block unrelated presentation. Exact fallback policy is an owner
choice: preserve a prior complete frame where still authorized, omit content, or
show a host-selected placeholder. A hide, lock or revoked presentation grant
cannot be ignored merely because old pixels are available.

Large resources may be acquired by tile or mip level when their provider supports
it. A large memory grant does not imply that an entire document, animation or
image pyramid must be resident. Prefetch and reuse are scheduling decisions.

## Version and lease lifetime

The candidate lifecycle separates publication from release:

| Phase | Consequence |
| --- | --- |
| Produce | Writer owns mutable backing; no reader consumes incomplete content |
| Publish | A specific stable version becomes available under an access grant |
| Acquire | Renderer obtains a lease and reserves its pin/import budget |
| Use | CPU reads or queued GPU work reference that version |
| Complete | All accesses represented by this lease have finished |
| Release | Read authority and pin reservation end; reclamation can proceed when other owners/users are gone |

A producer updates content by publishing another version or by regaining
exclusive write ownership after readers finish. A readiness signal alone does
not enforce immutability against a hostile CPU writer. Enforcement needs sealed
pages, validated access, ownership transfer or another admitted mechanism.
The GPU route additionally needs the completion/custody obligations in
[execution and presentation](execution_and_presentation.md).

Replacing a scene reference does not release a resource still used by an older
in-flight frame. Dropping an app-side handle does not reclaim backing held by an
accepted reader. Process exit does not imply GPU completion. Retained scene
references, active read leases and cache references need separate accounting.

Revocation stops future access according to the eventual authority contract.
Withdrawing a live mapping while safe-language references or device work still
use it is not a valid implementation of cancellation. The existing owner question
on [accepted-lease revocation](../../../OWNER_QUESTIONS.md#endpoint-transport-and-revocation-of-pending-ipc)
must settle the production rule. This proposal does not turn the Rust lab's
reader-pinning behavior into that rule.

The release ledger must survive an app's failure. Renderer failure also needs
recovery custody for in-flight device references. Final reclamation follows
completion or an established device-stop/reset guarantee, not merely loss of a
userspace process. Persistent storage objects follow their storage contract;
dropping a rendering lease does not delete the app's original asset.

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
[owner decision](../../../OWNER_QUESTIONS.md#render-resource-residency-and-sponsor-liability)
before this becomes a platform guarantee.
