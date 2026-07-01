# Chapter 06: Audio

> Mixing many streams into one device and fanning one capture device out to many clients, with playback as a held capability, the microphone gated like the camera, and real-time latency as the single hard guarantee.

## The Legacy Model

Audio stacks sprawl because they bundle six concerns that each grew their own layer. On Linux that is ALSA for raw device access, PulseAudio or PipeWire for per-app mixing and routing, and JACK for low-latency professional audio, frequently stacked on top of one another. macOS CoreAudio and Windows WASAPI are more unified but expose the same surface area. The recurring problems are consistent across all of them. Playback tends to be ambient, where any process that can open the device can emit sound and historically could read the microphone. Routing across speakers, headphones, HDMI, Bluetooth, and USB is stateful and fiddly. Per-app volume and ducking are special-cased. Low latency needs a privileged real-time path that fights the general scheduler. And the microphone, the most sensitive input on the machine, is too often reachable without the user knowing it is live.

## The Cathedral Model

Audio is the compositor for sound. A privileged **audio server** mixes per-app temporal streams into the output device the way the compositor mixes spatial surfaces into the framebuffer, and the same primitives carry it. A stream is a capability-scoped shared-memory ring ([[ipc_and_service_invocation]]): the app writes samples, the server reads them, with no kernel in the hot path. Playback is therefore a held capability, so an app that was never granted output cannot make a sound. Output devices are driver components ([[driver_model]]); the server holds their capabilities, mixes the active streams with per-stream gain, and writes the device ring. Capture is the mirror image and is gated like the camera: the microphone is a capability with an OS-drawn live indicator on the trusted path ([[human_permission_ux]], [[windowing_and_compositor]]), default-deny, revocable, and visible in the authority graph. The one irreducibly hard part is timing, which is a scheduling guarantee rather than an audio-architecture problem.

## The decided mechanism

The model above composes the IPC-ring, capability, and compositor machinery; the open layer is timing, exclusivity, and fan-out, and each resolves against work already done.

**Real-time latency is a scheduler reservation, and the floor is hardware.** The mixer runs in the RT class with a **WCET-bounded mix, dispatched every device-clock period** (parked on the device clock), plus a **frequency-floor** ([[power_management]]) so DVFS throttling cannot blow the deadline. The genuine cost of a *userspace* mixer is not the mixing (summing streams is cheap) but that it inserts **a buffering stage — about one period of added latency — and a second scheduling hop**: the app *and* the mixer must each hit a sub-millisecond deadline every period, doubling the jitter risk at tiny buffers. Both are handled the way JACK/PipeWire already prove reachable (<3 ms round trip, userspace): **RT reservations** make each stage punctual, and the graph (app → effects → mixer → device) is **processed synchronously within one period** rather than buffered per hop, so no per-node latency accrues. The floor below that is **hardware and firmware** — interrupt latency, the DMA buffer, and SMM/firmware stealing unbounded CPU (the same [[power_management]] hazard, fatal to an audio deadline). The reservation reaches that floor; the floor itself is a hardware fact.

**Effects and resampling are filter components; only the mandatory final stage runs in the server.** Reverb, EQ, per-app resampling, and spatialization are **processing nodes in a stream graph** — a pro-audio plugin chain is an ordinary component graph — out of the server so they compose and swap. Only the **device-rate resampling and the reserved-channel mix** run inside the server's RT cycle, to avoid an extra hop per period, the temporal twin of the compositor's per-surface-effects-vs-final-composite split.

**Exclusive device access is a leased capability handoff — the audio twin of fullscreen direct-scanout.** Normally the server holds the device capability and multiplexes; a client wanting the lowest latency requests **exclusive mode**, and the server **leases it the device capability**, stepping out of the path (`app → device` directly, zero mixer-stage latency). It is the same *forward-the-real-device-vs-synthesize-a-shared-view* choice a Matrix mediator makes, applied to a device — the direct analog of a compositor handing a fullscreen surface straight to scanout. It is gestural/policy-gated because, audio being additive, taking the device **silences** every other stream (a stronger takeover than visual fullscreen, which merely occludes), and it is reclaimed on release.

**Capture fans out from one device read; the mic is shared by default.** The capture server **reads the microphone once and fans the stream to each authorized listener** (each holding a capture capability) — the mirror of the compositor fanning capture out. A call app and a transcriber both receive it; a **denied listener gets silence** (blank capture — indistinguishable from a quiet room, [[human_permission_ux]]); **exclusive** capture is an unusual gestural takeover.

**Audio focus is OS-assigned from the foreground, and the reserved channel is a held capability.** Audio focus tracks input/window focus (the foreground surface owns it), assigned by the OS — an app **cannot grab** it. Ducking (a call lowers media) is a **server policy** over focus + stream class. The **reserved OS channel** (alarms, accessibility, system sounds) is **capability-gated**: only the OS and specifically-authorized components hold its capability, so it is **un-suppressible** (apps cannot mute it) *and* **un-abusable** (apps cannot emit on it) — a held capability, not an ambient channel, the audio twin of the trusted path. Spatial audio is a **filter component consuming a listener-model capability** (head/room context as a sensor cap), app-side or system-side, deferred with the other sensor-model work. An app-audio capture mix **excludes the reserved channel and any no-capture stream by default**, and its watched indicator stays distinct from the mic-live indicator (different meanings) while sharing the being-observed chrome.

## Concerns & Design Space

- **Streams as shared rings.** Playback and capture are capability-scoped shared-memory rings; holding the stream capability is the permission to emit or record ([[ipc_and_service_invocation]]).
- **The mixer as a broker.** The audio server reads the active streams, applies gain and effects, and writes the device buffer, the temporal twin of the compositor.
- **Real-time latency.** The device consumes a buffer on a hard deadline, and a miss is an audible glitch. The mixer parks on the device clock with the wait primitive and runs in a real-time scheduling class ([[scheduler_and_resources]], [[time_and_clocks]]). Professional-audio round trip is the stress case that decides how good the design is.
- **Microphone as sensitive input.** Capture is a capability with a trusted-path live indicator, default-deny, attributed, and revocable. A blank capture capability returns silence, so a coercive app cannot tell denial from a quiet room ([[human_permission_ux]]).
- **App-audio capture is the observation family.** Recording another app's *output* (screen sharing with sound, OBS) is the temporal twin of screen capture ([[windowing_and_compositor]]): a grant over a node of the mixing tree, same shape as pixels — your own streams free, another app's by grant, the reserved OS channel never — and a stream can declare itself absent or substituted in capture mixes the way a surface declares no-capture, enforced at mix time, never in the consumer. The same OS-drawn watched indicator applies.
- **Routing and devices.** "Default output" and "default input" are names bound in the per-principal resolution environment, rebindable live and per-app, so hot-plug, Bluetooth, and USB devices are driver components appearing and disappearing while the server re-routes.
- **Policy.** Per-stream volume, mute, ducking, and audio focus tied to the foreground are applied by the server as it mixes. Alarms, accessibility, and system sounds use a reserved OS channel that apps cannot suppress, the audio twin of the trusted path.
- **A/V sync.** Audio and video reference one clock ([[time_and_clocks]]); synchronization is presenting a given sample and a given frame at the same wall-clock time.
- **Recursive sub-mixing.** The mixer is an interface any component can implement, so a browser, a game, or a VM sub-mixes its children and submits one stream upward, the audio twin of recursive composition ([[windowing_and_compositor]]).
- **DSP graph.** Effects, resampling, and spatialization are processing nodes connected by streams, so a professional-audio plugin graph is an ordinary component graph.
- **Zero value.** A zero stream is its ZII zero ([[omega_substrate]]): a zero playback stream mixes as silence (valid-empty) and a zero capture stream reads silence as the inert null-object, which is exactly the blank-capture case, so an app handed a zeroed stream emits and records nothing rather than crashing the mixer.

## Key Questions

- **RT guarantee + latency — resolved:** a scheduler reservation (WCET-bounded mix, dispatched every device period, frequency-floor so DVFS can't miss it); userspace-mixer-at-pro-latency is reachable (JACK/PipeWire) via RT reservations + single-period graph processing; the floor below is hardware/firmware (interrupt latency, DMA buffer, SMM).
- **Effects/resampling location — resolved:** separate filter components in a stream graph, except the device-rate resampling + reserved-channel mix, which run in the server's RT cycle to avoid a per-period hop.
- **Exclusive device access — resolved:** a leased device-capability handoff (the app takes the device directly, mixer steps aside) — the audio twin of fullscreen direct-scanout; gestural, because it silences other streams.
- **Capture fan-out — resolved:** the server reads the mic once and fans it out to each authorized listener (shared by default); denied listeners get silence (blank); exclusive capture is a gestural takeover.
- **Audio focus — resolved:** OS-assigned from the foreground (not app-grabbable); ducking is a server policy; the reserved OS channel is a held capability (un-suppressible *and* un-abusable).

## Omega Leverage

- Streams are **capabilities + domains** over shared-memory rings, audited through the same authority-flow report as every other resource ([capabilities chapter](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- The audio server is an explicit **boundary provider**: the audited edge between proved Omega code and the raw audio hardware.
- The mixer is a **machine with states** ([machines](../../../../Omega/wiki/language_guide/chapter_3_machines.md), [states](../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md)): idle, running, draining, woken on the device clock.
- The mixer interface is a **trait** ([traits](../../../../Omega/wiki/language_guide/chapter_13_traits.md)) any component can implement, so sub-mixing is one interface with many implementations, resolved from the client's environment ([modules & imports](../../../../Omega/wiki/language_guide/chapter_14_modules_imports_visibility.md)).
- Buffer formats (sample rate, channel layout, frame size) are **wire data** ([wire protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md)) shared by client and server.
- Omega does not itself provide a real-time latency guarantee; that is a scheduler obligation Cathedral defines on top of the language ([[scheduler_and_resources]]).

## Open Questions

- **Userspace mixer at pro-audio latency — resolved:** reachable (JACK/PipeWire are the existence proof) with RT reservations + single-period graph processing; the lowest-latency clients that can't spare the mixer stage take **exclusive mode** (the fullscreen-bypass analog). The residual is empirical — the exact commodity-hardware floor, set by interrupt latency and SMM, the same limit the scheduler/power work names, not an audio-specific unknown.
- **Spatial audio — resolved-direction:** a filter component consuming a listener-model capability (head/room as a sensor cap), app- or system-side; deferred with the other sensor-model work.
- **Un-suppressible system sounds without abuse — resolved:** the reserved channel is a **held capability** — apps can neither mute it nor emit on it, because it's not an ambient channel.
- **App-audio capture exclusions + indicator — resolved:** excludes the reserved channel and any no-capture stream by default; the watched indicator stays distinct from the mic-live indicator (different meanings) but shares the being-observed chrome.

## Related
- [[media_and_graphics]] — the display and GPU half of media; audio is the symmetric temporal half.
- [[ipc_and_service_invocation]] — the shared-region primitive that carries audio streams.
- [[scheduler_and_resources]] — the real-time class and budgets the mixer depends on.
- [[time_and_clocks]] — the clock behind latency, buffering, and A/V sync.
- [[human_permission_ux]] — microphone consent, the live indicator, and synthetic (blank) capture.
- [[driver_model]] — sound devices as components.
- [[windowing_and_compositor]] — the compositor whose mixing and recursive-composition pattern audio mirrors.
