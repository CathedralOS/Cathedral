# Shared-buffer audio playback

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | None |
| Last reviewed | 2026-10-25 |

## Baseline and ownership

Playback uses caller-funded shared sample buffers admitted by a platform audio
service. Authorized backing can be mapped to the final mixer without copying
samples or materializing a submix at every Matrix ancestor. The service mixes
admitted streams into output backing and submits it through the audio device
driver. This is the baseline, not an exclusive-device handoff.

```text
App A's sample ring ----+
App B's sample ring ----+--> final mixer --> output buffer --> audio driver/device
Authorized system ring-+

Ancestors supply routing, permissions and controls; they need not copy samples.
```

Kernel responsibilities remain mapping/authority enforcement, resource accounting,
scheduling and [notifications](../execution/notifications.md). The platform service
owns stream admission, mixing and output orchestration; drivers own hardware
interaction. An IOMMU, where applicable, constrains device DMA rather than doing
mixing or playback. An application's shared buffer need not be device-mapped.
Driver-specific staging and format conversion are permitted; this contract does
not promise universal zero-copy or a latency bound.

## Authority hierarchy is not a mandatory processing chain

Forwarding authorized backing is distinct from forwarding unrestricted control.
The final service must preserve the effective ancestor routing, attenuation,
volume/mute controls and revocation. A direct mapping is not a bypass around them.
Mapping a buffer alone grants neither playback nor permission to record it.
Access to stream samples is limited to the principals authorized for processing
or observation; a routing intermediary need not map them.

Policy-only ancestors must not require another sample buffer, copy or processing
hop merely because the Matrix tree is deeper. Their controls may be represented
at the final mixer. Actual DSP, conversion or deliberate submixing can introduce
processing nodes and intermediate backing where signal semantics require them.
Flattening must not change those semantics: processing a combined bus is not
generally equivalent to processing every source separately.

Authority is scoped to the admitted stream and controls. An ordinary client
cannot mute, replace or impersonate independently authorized system streams.
This isolation does not promise audibility despite muted hardware, device failure
or missed deadlines. Exclusive output and its interaction with reserved audio
remain a separate decision.

## Buffer protocol, funding and lifetime

Use the existing [shared-memory IPC boundary](../communication/ipc.md), not a new
kernel audio payload queue. The service admits format, actual mapped capacity and
resource bounds. Producer/consumer positions and frame lengths are untrusted
inputs: peer-written headers cannot enlarge a mapping or authorize out-of-bounds
access. Notifications indicate work may be available, not that samples are valid.

The protocol must define when frames are published, consumed and safe to reuse,
with synchronization appropriate to the selected backing. A writable native
mapping does not become immutable because its owner promised not to modify it.
Readers must tolerate permitted concurrent writes or establish enforced stable
ownership/snapshots before relying on immutability. Audio samples must never become
unchecked control metadata, pointers or buffer lengths.

Caller funding covers stream backing and admitted work; service/output backing
also needs an identified funding owner. Forwarding mappings does not silently
multiply payload storage, erase accounting or allow unlimited streams. Backing
stays valid and funded while a consumer or device can still use it. Closure,
revocation, producer death and driver retirement must stop or retire accesses
before reuse; an unmap in one process is not proof that DMA has stopped.

Late or malformed producers cannot make the final service wait indefinitely or
consume unchecked data. The admitted stream protocol must specify bounded
underrun/overrun, failure and cleanup behavior. This decision does not select a
concealment algorithm, ring encoding, sample format set or period size.

## Timing and evidence

Shared transport removes a required per-ancestor payload hop, not the work of
mixing, resampling, real effects or driver buffering. Deadline guarantees require
an admitted workload and evidence for the complete device-facing path. Real-time
scheduling alone does not establish adequate data readiness or bounded DSP work.
No Cathedral round-trip figure or guaranteed absence of glitches is ratified here.

Validation must distinguish policy-only nesting from real processing depth and
measure copies, buffering, deadline misses and work independently. It must include
hostile producer controls, a stalled source alongside a healthy one, and custody
through teardown. Existing generic IPC proofs are reusable mechanisms, not audio
implementation evidence.

## Deferred scope and implementation

Exclusive single-stream service operation and raw device handoff are distinct
possibilities, neither selected nor required for baseline playback. Their authority,
reserved-channel and failure contracts need a separate decision. Do not implement
handoff as the assumed fix for an unmeasured latency problem.

Capture/microphone consent, capture fan-out, focus assignment, routing UX, retained
sample assets and spatial audio remain outside this partial playback contract.
Related design material is not silently ratified by adopting shared buffers.

The [audio outcome](../../../TASKS_RUST.md#shared-buffer-audio-playback) tracks the
service, driver and ordinary consumer path. The
[audio design](../../design/part_6_human_surface/06_audio.md) and
[graph investigation](../../drafts/audio_graph_notes.md) retain rationale and
unselected alternatives. This docs change supplies no runtime implementation.
