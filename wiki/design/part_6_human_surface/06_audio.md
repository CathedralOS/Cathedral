# Chapter 06: Audio

> Mixing many streams into one device and fanning one capture device out to many clients, with playback as a held capability, the microphone gated like the camera, and real-time latency as the single hard guarantee.

## The Legacy Model

Audio stacks sprawl because they bundle six concerns that each grew their own layer. On Linux that is ALSA for raw device access, PulseAudio or PipeWire for per-app mixing and routing, and JACK for low-latency professional audio, frequently stacked on top of one another. macOS CoreAudio and Windows WASAPI are more unified but expose the same surface area. The recurring problems are consistent across all of them. Playback tends to be ambient, where any process that can open the device can emit sound and historically could read the microphone. Routing across speakers, headphones, HDMI, Bluetooth, and USB is stateful and fiddly. Per-app volume and ducking are special-cased. Low latency needs a privileged real-time path that fights the general scheduler. And the microphone, the most sensitive input on the machine, is too often reachable without the user knowing it is live.

## The Cathedral Model

Audio is the compositor for sound. A privileged **audio server** mixes per-app temporal streams into the output device the way the compositor mixes spatial surfaces into the framebuffer, and the same primitives carry it. A stream is a capability-scoped shared-memory ring ([[ipc_and_service_invocation]]): the app writes samples, the server reads them, with no kernel in the hot path. Playback is therefore a held capability, so an app that was never granted output cannot make a sound. Output devices are driver components ([[driver_model]]). The server holds their capabilities, mixes the active streams with per-stream gain, and writes the device ring. Capture is the mirror image and is gated like the camera: the microphone is a capability with an OS-drawn live indicator on the trusted path ([[human_permission_ux]], [[windowing_and_compositor]]), default-deny, revocable, and visible in the authority graph. The one irreducibly hard part is timing, which is a scheduling guarantee rather than an audio-architecture problem.

## The decided mechanism

The model above composes the IPC-ring, capability, and compositor machinery. What remains specific to audio is timing, exclusivity, fan-out, and focus, and each builds on work done elsewhere.

Real-time latency is a scheduler reservation, and the floor is hardware. The mixer runs in the RT class with a WCET-bounded mix, dispatched every device-clock period (parked on the device clock), plus a frequency floor ([[power_management]]) so DVFS throttling cannot blow the deadline. The cost of a userspace mixer is not the mixing, since summing streams is cheap. It is that the mixer inserts a buffering stage, about one period of added latency, and a second scheduling hop: the app and the mixer must each hit a sub-millisecond deadline every period, doubling the jitter risk at tiny buffers. Both are handled the way JACK and PipeWire already prove reachable, under 3 ms round trip in userspace. RT reservations make each stage punctual, and the graph (app → effects → mixer → device) is processed synchronously within one period rather than buffered per hop, so no per-node latency accrues. The floor below that is hardware and firmware: interrupt latency, the DMA buffer, and SMM or firmware stealing unbounded CPU, which is the same [[power_management]] hazard and fatal to an audio deadline. The reservation reaches that floor. The floor itself is a hardware fact.

Effects and resampling are filter components, and only the mandatory final stage runs in the server. Reverb, EQ, per-app resampling, and spatialization are processing nodes in a stream graph, so a pro-audio plugin chain is an ordinary component graph. They live outside the server so they compose and swap. Only the device-rate resampling and the reserved-channel mix run inside the server's RT cycle, to avoid an extra hop per period. This is the temporal twin of the compositor's split between per-surface effects and the final composite.

Exclusive device access is a leased capability handoff, the audio twin of fullscreen direct scanout. Normally the server holds the device capability and multiplexes. A client wanting the lowest latency requests **exclusive mode**, and the server leases it the device capability and steps out of the path, so the app writes the device directly with zero mixer-stage latency. It is the same forward-the-real-device-versus-synthesize-a-shared-view choice a Matrix mediator makes, applied to a device, and the direct analog of a compositor handing a fullscreen surface straight to scanout. It is gestural and policy-gated because audio is additive: taking the device silences every other stream, a stronger takeover than visual fullscreen, which merely occludes. The device is reclaimed on release.

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
- **Recursive sub-mixing.** The mixer is an interface any component can implement, so a browser, a game, or a VM sub-mixes its children and submits one stream upward, the audio twin of recursive composition ([[windowing_and_compositor]]).
- **DSP graph.** Effects, resampling, and spatialization are processing nodes connected by streams, so a professional-audio plugin graph is an ordinary component graph.
- **Zero value.** A zero stream is its ZII zero ([[omega_substrate]]): a zero playback stream mixes as silence (valid-empty) and a zero capture stream reads silence as the inert null-object, which is exactly the blank-capture case, so an app handed a zeroed stream emits and records nothing rather than crashing the mixer.

## Key Questions

- How is the real-time guarantee delivered, and what latency is reachable from a userspace mixer? The answer is a scheduler reservation with a WCET-bounded mix per device period and a frequency floor, with pro-audio latency shown reachable by JACK and PipeWire through RT reservations and single-period graph processing.
- Where do effects and resampling run, and which stages must stay inside the server's RT cycle?
- How does a client take exclusive device access, and why is that a gestural act rather than an API call?
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

- What is the commodity-hardware latency floor? A userspace mixer at pro-audio latency is reachable, with JACK and PipeWire as the existence proof, through RT reservations and single-period graph processing, and the lowest-latency clients that cannot spare the mixer stage take exclusive mode, the fullscreen-bypass analog. The residual is empirical: the exact floor is set by interrupt latency and SMM, the same limit the scheduler and power work name, not an audio-specific unknown.
- How is the listener model (head and room context) delivered as a sensor capability for spatial audio? The shape is a filter component consuming that capability, app-side or system-side; the sensor-model work itself is deferred.

## Related
- [[media_and_graphics]] — the display and GPU half of media; audio is the symmetric temporal half.
- [[ipc_and_service_invocation]] — the shared-region primitive that carries audio streams.
- [[scheduler_and_resources]] — the real-time class and budgets the mixer depends on.
- [[time_and_clocks]] — the clock behind latency, buffering, and A/V sync.
- [[human_permission_ux]] — microphone consent, the live indicator, and synthetic (blank) capture.
- [[driver_model]] — sound devices as components.
- [[windowing_and_compositor]] — the compositor whose mixing and recursive-composition pattern audio mirrors.
