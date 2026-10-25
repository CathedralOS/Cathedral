# Chapter 06: Audio

> Shared sample backing and one final output mixer; authority nesting does not require a processing hop at every ancestor.

## The Legacy Model

Audio stacks sprawl because they bundle six concerns that each grew their own layer. On Linux that is ALSA for raw device access, PulseAudio or PipeWire for per-app mixing and routing, and JACK for low-latency professional audio, frequently stacked on top of one another. macOS CoreAudio and Windows WASAPI are more unified but expose the same surface area. The recurring problems are consistent across all of them. Playback tends to be ambient, where any process that can open the device can emit sound and historically could read the microphone. Routing across speakers, headphones, HDMI, Bluetooth, and USB is stateful and fiddly. Per-app volume and ducking are special-cased. Low latency needs a privileged real-time path that fights the general scheduler. And the microphone, the most sensitive input on the machine, is too often reachable without the user knowing it is live.

## The Cathedral Model

The [shared-buffer playback contract](../../spec/human_surface/audio.md) now ratifies
caller-funded sample transport, preserved ancestor controls and final service-owned
mixing. The [audio hierarchy investigation](../../drafts/audio_graph_notes.md)
retains further graph candidates. Capture, focus policy and exclusive mode below
remain design proposals, not accepted additions to that partial contract.
No Rust audio driver or graph implementation establishes a latency guarantee.

Playback is a held capability over caller-funded shared sample backing. The
platform audio service reads authorized streams, performs final mixing and submits
output through an audio driver. Parents can pass backing grants and policy without
copying every sample; the kernel enforces mappings, funding and notifications.
Driver buffers and DMA confinement are distinct from IPC transport, and an IOMMU
does not perform audio mixing. Capture is a separate proposed boundary, not an
automatic consequence of permission to play. Timing requires evidence across the
whole service/driver path, not merely a scheduling reservation.

## Mechanisms under consideration

The model above composes the IPC-ring, capability, and compositor machinery. What remains specific to audio is timing, exclusivity, fan-out, and focus, and each builds on work done elsewhere.

Real-time scheduling and resource admission support punctual execution, but do not
alone prove data readiness, bounded signal-processing work or device latency.
Measure period, buffering, DSP work, underruns and driver behavior on the admitted
path. Shared transport avoids a mandatory buffering/scheduling hop per policy-only
ancestor; it does not make mixing free or prove a fixed round-trip time.
No numeric latency claim from another audio stack is a Cathedral guarantee.

Effects and resampling are actual processing work. They may be explicit components
or supported service processing; exact graph placement is not selected here.
Parents that only attenuate authority or contribute controls need no submix.
Parents performing real bus effects may need one. Preserve signal semantics rather
than flattening every transformation or requiring every parent to touch samples.

Exclusive output is deferred to a separate owner decision. A service accepting
one stream while retaining device control differs from raw device-capability
handoff. Neither is the assumed implementation of low latency. A future proposal
must say which output controls are surrendered and how reserved audio, failure
and reclamation behave; shared-buffer playback does not depend on that proposal.

Capture fans out from one device read, and the mic is shared by default. The capture server reads the microphone once and fans the stream to each authorized listener, each holding a capture capability. This mirrors the compositor fanning capture out. A call app and a transcriber both receive it. A denied listener gets silence, a blank capture indistinguishable from a quiet room ([[human_permission_ux]]). Exclusive capture is an unusual gestural takeover.

Audio focus is OS-assigned from the foreground, and the reserved channel is a held capability. Audio focus tracks input and window focus, so the foreground surface owns it. It is assigned by the OS, and an app cannot grab it. Ducking (a call lowers media) is a server policy over focus plus stream class. The **reserved OS channel** (alarms, accessibility, system sounds) is capability-gated: only the OS and specifically authorized components hold its capability, so it is un-suppressible (apps cannot mute it) and un-abusable (apps cannot emit on it). It is a held capability, not an ambient channel, the audio twin of the trusted path. Spatial audio is a filter component consuming a listener-model capability, with head and room context as a sensor capability, app-side or system-side, and it is deferred with the other sensor-model work. An app-audio capture mix excludes the reserved channel and any no-capture stream by default. Its watched indicator stays distinct from the mic-live indicator, since the two mean different things, while sharing the being-observed chrome.

## Concerns & Design Space

- **Streams as shared rings.** Playback and capture are capability-scoped shared-memory rings; holding the stream capability is the permission to emit or record ([[ipc_and_service_invocation]]).
- **The mixer as a broker.** The audio server reads the active streams, applies gain and effects, and writes the device buffer, the temporal twin of the compositor.
- **Real-time latency.** The device consumes a buffer on a hard deadline, and a miss is an audible glitch. The mixer parks on the device clock with the wait primitive and runs in a real-time scheduling class ([[scheduler_and_resources]], [[time_and_clocks]]). Professional-audio round trip is the stress case that decides how good the design is.
- **Microphone as sensitive input.** Capture is a capability with a trusted-path live indicator, default-deny, attributed, and revocable. A blank capture capability returns silence, so a coercive app cannot tell denial from a quiet room ([[human_permission_ux]]).
- **App-audio capture is the observation family.** Recording another app's output (screen sharing with sound, OBS) is the temporal twin of screen capture ([[windowing_and_compositor]]): a grant over a node of the mixing tree, the same shape as pixels. Your own streams are free, another app's is by grant, and the reserved OS channel is never capturable. A stream can declare itself absent or substituted in capture mixes the way a surface declares no-capture, enforced at mix time, never in the consumer. The same OS-drawn watched indicator applies.
- **Routing and devices.** "Default output" and "default input" are names bound in the per-principal resolution environment, rebindable live and per-app, so hot-plug, Bluetooth, and USB devices are driver components appearing and disappearing while the server re-routes.
- **Policy.** Per-stream volume, mute, ducking, and audio focus tied to the foreground are applied by the server as it mixes. Alarms, accessibility, and system sounds use a reserved OS channel that apps cannot suppress, the audio twin of the trusted path.
- **A/V sync.** Audio and video reference one clock ([[time_and_clocks]]); synchronization is presenting a given sample and a given frame at the same wall-clock time.
- **Recursive policy without mandatory submixing.** Parents may supply routing and controls while final mixing reads authorized backing directly. A deliberate submix remains appropriate for real effects; nesting alone creates no sample-copy requirement.
- **DSP graph.** Effects, resampling, and spatialization are processing nodes connected by streams, so a professional-audio plugin graph is an ordinary component graph.
- **Zero value.** A zero stream is its ZII zero ([[omega_substrate]]): a zero playback stream mixes as silence (valid-empty) and a zero capture stream reads silence as the inert null-object, which is exactly the blank-capture case, so an app handed a zeroed stream emits and records nothing rather than crashing the mixer.

## Key Questions

- What workload and device measurements justify a latency/deadline promise? Scheduling, buffering, data readiness and processing costs must be established together.
- Where do effects and resampling run, and which stages must stay inside the server's RT cycle?
- Exclusive single-stream output and raw device handoff require a separate owner decision; neither blocks shared playback.
- How does capture fan out to many listeners, and what does a denied listener receive?
- Who assigns audio focus, and how are system sounds un-suppressible without being abusable?
- What does an app-audio capture mix exclude by default, and how does its indicator relate to the mic-live indicator?

## Omega Leverage

- Streams are capabilities plus domains over shared-memory rings, audited through the same authority-flow report as every other resource ([capabilities chapter](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- The audio server is a boundary provider: the audited edge between proved Omega code and the raw audio hardware.
- The mixer is a machine with states ([machines](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_3_machines.md), [states](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_4_states_transitions.md)): idle, running, draining, woken on the device clock.
- The mixer interface is a trait ([traits](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md)) any component can implement, so sub-mixing is one interface with many implementations, resolved from the client's environment ([modules & imports](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_15_modules_imports_visibility.md)).
- Buffer formats (sample rate, channel layout, frame size) are ordinary numbered schemas under a selected wire codec ([wire protocols](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md#wire-protocols)) shared by client and server.
- Omega does not itself provide a real-time latency guarantee; that is a scheduler obligation Cathedral defines on top of the language ([[scheduler_and_resources]]).

## Open Questions

- What latency and workload bounds can the implemented shared playback path sustain? Separate policy-only nesting, real processing depth and device/firmware effects; do not assume exclusive handoff supplies the answer.
- How is the listener model (head and room context) delivered as a sensor capability for spatial audio? The shape is a filter component consuming that capability, app-side or system-side; the sensor-model work itself is deferred.

## Related
- [[media_and_graphics]] — the display and GPU half of media; audio is the symmetric temporal half.
- [[ipc_and_service_invocation]] — the shared-region primitive that carries audio streams.
- [[scheduler_and_resources]] — the real-time class and budgets the mixer depends on.
- [[time_and_clocks]] — the clock behind latency, buffering, and A/V sync.
- [[human_permission_ux]] — microphone consent, the live indicator, and synthetic (blank) capture.
- [[driver_model]] — sound devices as components.
- [[windowing_and_compositor]] — the compositor whose mixing and recursive-composition pattern audio mirrors.
