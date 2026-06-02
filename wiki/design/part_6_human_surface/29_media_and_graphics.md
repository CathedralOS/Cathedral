# Chapter 29: Media & Graphics

> The low-level pipeline beneath the window manager: display server, GPU, decode,
> and audio — capability-gated, and deliberately kept narrow, because graphics
> stacks are where elegant operating systems go to die.

## The Legacy Contract

The media stack is the least-contained part of a legacy OS. GPU access means a
giant, vendor-specific userspace driver mapped into the app with broad DMA reach;
video decode and audio mixing run through sprawling frameworks
(VA-API/V4L2/PulseAudio/PipeWire/DRM-KMS) with their own ad hoc permission
models. Screen capture is frequently all-or-nothing. DRM/content protection
drags in opaque, attestation-hungry blobs. The result is enormous attack surface,
poor isolation between apps sharing the GPU, and a pipeline so complex that
"clean-slate" OS projects routinely founder trying to reimplement it.

## What Cathedral Wants

Treat every media facility as a **capability-gated `device_io` effect** over an
explicit pipeline, and keep the *first target deliberately narrow* — enough to
play video, render UI, and mix audio on one class of device, not a universal
graphics platform on day one. The display server / compositor pipeline here sits
*beneath* the window management of [[27_windowing_and_compositor]]: this chapter
owns the bytes-and-frames layer, that chapter owns windows and trust. GPU,
decode, and screen capture are all held capabilities, never ambient: an app
renders because it holds a rendering capability, captures the screen because it
holds a (rare, attributed) capture capability.

The discipline is *say no early*. Pick one GPU path, one decode path, one audio
graph for the first wedge; gate each behind a capability; route everything
through the driver model ([[24_driver_model]]) so the untrusted vendor blob is
isolated rather than mapped into apps.

## Concerns & Design Space

- **Display server / compositor pipeline.** The frame-composition and scanout
  layer beneath window management; the boundary to GPU scanout hardware.
- **GPU access.** Capability-gated command submission with isolation between
  principals sharing the device; no ambient DMA reach.
- **Video decode.** A `device_io` capability over a decode session, not a mapped
  library with the run of memory.
- **Audio graph.** A routed graph of capability-held nodes; mixing and capture are
  distinct, separately granted.
- **DRM / content protection.** Quarantine the attestation blob behind a narrow
  boundary; decide *whether* to support it at all in the first target.
- **Screen capture.** A capability ([[03_capability_model]]), rare, attributed,
  and visibly indicated — never silent.
- **Window isolation & input routing.** Defer to [[27_windowing_and_compositor]];
  the pipeline must not leak one principal's frames or input to another.
- **Accessibility & color management.** First-class consumers of the pipeline, not
  afterthoughts; accessibility needs a privileged-but-audited read path.
- **Frame scheduling / GPU budget.** GPU time and frame deadlines are scheduled
  resources ([[10_scheduler_and_resources]]).
- **Low-latency media & power-aware rendering.** Latency and energy as explicit
  budgets, with rendering that backs off under power policy
  ([[14_power_management]]).

## Key Questions

- What is the single narrow GPU + decode + audio target for the first device, and
  what is explicitly out of scope?
- How is the untrusted vendor GPU/decode driver isolated so a bug there does not
  become app or kernel compromise ([[24_driver_model]])?
- How is screen capture surfaced as a capability with an unspoofable in-use
  indicator?
- How do frame deadlines and GPU budget integrate with the general scheduler
  rather than living in a private real-time silo?

## Omega Leverage

- GPU/decode/capture/audio are **capabilities + the `device_io` effect**, audited
  on the same axes as any other boundary
  ([capabilities & effects](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- The pipeline is a set of **boundary providers** wrapping firmware/vendor code;
  the audited edge is explicit.
- Decode/render sessions are **machines with states**
  ([machines](../../../../Omega/wiki/language_guide/chapter_3_machines.md)):
  configured → running → flushing → drained.
- Frame/audio buffers crossing the hardware edge are **wire data**
  ([wire protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md)).
- Omega gives no GPU memory-isolation model for shared devices; that isolation
  guarantee is a Cathedral + hardware obligation.

## Open Questions

- Is a clean capability model achievable over today's monolithic vendor GPU
  stacks, or does the first target need a simpler/software path until drivers
  cooperate?
- Does Cathedral support hardware DRM at all in v1, given the trust it demands?

## Related
- [[27_windowing_and_compositor]] — window management above this pipeline.
- [[24_driver_model]] — isolating the GPU/decode vendor driver.
- [[10_scheduler_and_resources]] — frame scheduling and GPU budget.
- [[14_power_management]] — power-aware rendering.
- [[03_capability_model]] — screen capture and GPU access as held capabilities.
