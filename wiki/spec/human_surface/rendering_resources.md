# Rendering resources and backing lifetime

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-10-25 |

## Identity, authority and backing

The owner accepted these boundaries on 2026-10-23. The
[presentation contract](rendering.md) owns placement and rendering demand; the
[IPC contract](../communication/ipc.md) owns access custody and mapping obligations.

A resource reference identifies an image/version available through an authorized
provider. Identity and access authority are distinct from storage location. A
handle, digest or version number alone neither grants access nor makes mutable
bytes stable. The provider must enforce the publication/access rules for the
particular backing before consumers rely on stable contents.

Producer/leaf-funded backing is a supported local route. The leaf allocates and
funds an image, explicitly shares bounded access, and the renderer acquires it for
use and releases it afterward. The renderer need not keep a duplicate permanent
image store. Sharing a raw pointer into an app heap is not sufficient: backing
custody must survive every actual reader under the IPC contract.

## Local import and remote transfer

The common resource model admits different mechanisms:

| Backing relationship | Supported route |
| --- | --- |
| Compatible local memory | Authorized shared mapping or image import |
| Incompatible local representation | Explicit conversion or staging allocation |
| Remote consumer | Upload into destination-side storage |

Registration and making backing available are distinct from acquiring stable
access and completing use. Protocols describe supported imports/transfers;
remote memory is not presumed to be a locally dereferenceable pointer. The exact
registration API, version encoding, network transport and GPU representation
remain implementation work, not required copies for the compatible local case.

Uploaded or converted copies have destination-side storage and lifetimes. The
producer's death does not physically delete an already received copy. Retention,
retirement and authorized reuse follow the applicable resource/session policy.
Private-resource identity must not grant unrelated consumers import or capture.

### Remote hosting

The owner accepted the remote-hosting boundary on 2026-10-25. A destination-side
viewer/host owns or explicitly sponsors local backing for uploaded resources and
submits through the ordinary local drawing/image interface. It may forward
validated commands to the rendering service; it need not rasterize them itself.
Remote names resolve only to authorized local resource versions, never directly
to local memory authority. Hosting one app or a desktop subtree uses this boundary.

Commands can reuse uploaded images/font resources until their versions change or
their backing is released. Arbitrary changing framebuffers still need pixel
transfer, possibly through a negotiated codec. Network transport, compression and
device import remain engineering choices; there is no network-shared pointer.

## Unavailable resources and submission results

The owner accepted omission rather than implicit fallback on 2026-10-25. If a
resource is unavailable when needed for drawing, skip the draw requiring it and
continue unrelated work. Do not silently substitute a stale version, a reused
identifier's new object, a placeholder or retained old pixels for that draw.
The host may explicitly submit authorized fallback content as ordinary work.
Output generation must not accidentally retain an omitted draw from an older
frame merely because the renderer uses incremental updates.

Report unavailable resources in a result associated with the exact submission;
asynchronous completion is allowed. A global last-error slot is not sufficient.
The host can supply/reacquire resources and submit again. Slow or absent providers
must not indefinitely block unrelated rendering. This is an availability outcome,
not permission to accept invalid commands or unauthorized resource access.

Missing before acquisition differs from disappearance during use. Successful
acquisition retains stable, authorized backing until CPU/device readers finish;
eviction, producer exit and reuse cannot invalidate an active reader. Registration
alone does not promise residency. Results, outstanding requests and resource
lists require bounded accounting along with backing and render work.

## Retirement and physical reclamation

Confirmed death of a leaf retires its live resource registrations for new work.
Queued work that has not acquired backing can be discarded, and future output
can remove the producer's contribution. This does not reclaim backing that a
CPU reader, accepted memory view or device still uses.

Physical reclamation waits for readers to release and device work to complete
or be safely stopped. Process death and IPC disconnection do not themselves prove
that those accesses ended. Retirement bookkeeping and storage custody must remain
valid through the failure; an untrusted producer cannot promise them away.
This governs live registrations, not deletion of durable source files or separately
authorized copies already transferred to another owner.

A remote outage or timeout is not confirmed process death. Locally available,
authorized copies do not become missing solely because the remote source is
silent. Unavailable resources follow the omission rule above. Whether a disconnected
session retires its entire presentation is separate lifecycle policy; this does
not promise distributed death detection. Retaining bytes never extends expired
presentation/capture authority.

## Bounded retention and work

Source allocations, destination storage, imported/pinned backing, registration
metadata and queued transfer/render work require bounded accounting. Leaf-funded
pixels do not fund unlimited compositor descriptors or submissions. Destination
capacity must be admitted even when source memory is already funded.

Charges persist until actual reclamation/use completion; killing or restarting a
producer must not reset the bill for retained storage. Destination hosts fund or
explicitly sponsor their local uploads; source funding is not destination credit.
Shared derived-cache sponsor selection, cache eviction and any stronger residency
guarantees remain separate policy choices. Registration is not a blanket promise
that all decoded resources remain resident forever.

## Evidence and limits

The [Rust page contract](../../../source-rs/kernel/contracts/memory.rs) and
[memory lab](../../../source-rs/distribution/lab/memory/main.rs) exercise private
writes, sealed transferable reads and reader lifetime. They do not implement the
general image-provider service, remote upload or GPU completion contract above.

The Rust [capture mechanisms](../../../source-rs/platform/libraries/matrix/capture/README.md)
now have host-tested bounded lease/result lifetime and masked software output.
The installed Split provider now connects source frame acknowledgment to funded
private raster output and sealed IPC delivery to a separate viewer. The viewer
checks actual redaction and padding under a disposable probe feature. Retained
source images stay sealed and are mapped only during the provider's bounded
raster dispatch. Source/recipient failure tests and ordinary delivery are recorded
in [the handoff](../../../TASKS_RUST.md). Fixed owner-selected masks and one Notes
source do not establish a general privacy metadata protocol, recursive capture
readiness, remote transfer or GPU lifetime behavior.
Derived image/font cache policy, negotiated capacities and precise
publication/transfer schemas remain engineering and policy work. The
[resource proposal](../../proposals/0000_rendering_and_composition/resource_custody.md)
retains alternatives for those unresolved choices.
