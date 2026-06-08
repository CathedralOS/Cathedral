# Chapter 06: Audio

> Mixing many streams into one device and fanning one capture device out to many clients, with playback as a held capability, the microphone gated like the camera, and real-time latency as the single hard guarantee.

## The Legacy Model

Audio stacks sprawl because they bundle six concerns that each grew their own layer. On Linux that is ALSA for raw device access, PulseAudio or PipeWire for per-app mixing and routing, and JACK for low-latency professional audio, frequently stacked on top of one another. macOS CoreAudio and Windows WASAPI are more unified but expose the same surface area. The recurring problems are consistent across all of them. Playback tends to be ambient, where any process that can open the device can emit sound and historically could read the microphone. Routing across speakers, headphones, HDMI, Bluetooth, and USB is stateful and fiddly. Per-app volume and ducking are special-cased. Low latency needs a privileged real-time path that fights the general scheduler. And the microphone, the most sensitive input on the machine, is too often reachable without the user knowing it is live.

## The Cathedral Model

Audio is the compositor for sound. A privileged **audio server** mixes per-app temporal streams into the output device the way the compositor mixes spatial surfaces into the framebuffer, and the same primitives carry it. A stream is a capability-scoped shared-memory ring ([[ipc_and_service_invocation]]): the app writes samples, the server reads them, with no kernel in the hot path. Playback is therefore a held capability, so an app that was never granted output cannot make a sound. Output devices are driver components ([[driver_model]]); the server holds their capabilities, mixes the active streams with per-stream gain, and writes the device ring. Capture is the mirror image and is gated like the camera: the microphone is a capability with an OS-drawn live indicator on the trusted path ([[human_permission_ux]], [[windowing_and_compositor]]), default-deny, revocable, and visible in the authority graph. The one irreducibly hard part is timing, which is a scheduling guarantee rather than an audio-architecture problem.

## Concerns & Design Space

- **Streams as shared rings.** Playback and capture are capability-scoped shared-memory rings; holding the stream capability is the permission to emit or record ([[ipc_and_service_invocation]]).
- **The mixer as a broker.** The audio server reads the active streams, applies gain and effects, and writes the device buffer, the temporal twin of the compositor.
- **Real-time latency.** The device consumes a buffer on a hard deadline, and a miss is an audible glitch. The mixer parks on the device clock with the wait primitive and runs in a real-time scheduling class ([[scheduler_and_resources]], [[time_and_clocks]]). Professional-audio round trip is the stress case that decides how good the design is.
- **Microphone as sensitive input.** Capture is a capability with a trusted-path live indicator, default-deny, attributed, and revocable. A blank capture capability returns silence, so a coercive app cannot tell denial from a quiet room ([[human_permission_ux]]).
- **Routing and devices.** "Default output" and "default input" are names bound in the per-principal resolution environment, rebindable live and per-app, so hot-plug, Bluetooth, and USB devices are driver components appearing and disappearing while the server re-routes.
- **Policy.** Per-stream volume, mute, ducking, and audio focus tied to the foreground are applied by the server as it mixes. Alarms, accessibility, and system sounds use a reserved OS channel that apps cannot suppress, the audio twin of the trusted path.
- **A/V sync.** Audio and video reference one clock ([[time_and_clocks]]); synchronization is presenting a given sample and a given frame at the same wall-clock time.
- **Recursive sub-mixing.** The mixer is an interface any component can implement, so a browser, a game, or a VM sub-mixes its children and submits one stream upward, the audio twin of recursive composition ([[windowing_and_compositor]]).
- **DSP graph.** Effects, resampling, and spatialization are processing nodes connected by streams, so a professional-audio plugin graph is an ordinary component graph.
- **Zero value.** A zero stream is its ZII zero ([[omega_substrate]]): a zero playback stream mixes as silence (valid-empty) and a zero capture stream reads silence as the inert null-object, which is exactly the blank-capture case, so an app handed a zeroed stream emits and records nothing rather than crashing the mixer.

## Key Questions

- What real-time scheduling guarantee does the mixer need, and how low can round-trip latency go with a userspace mixer and isolated clients?
- Where do shared effects and resampling run: inside the server, or as separate filter components in a stream graph?
- How is exclusive device access, for a professional-audio app that wants the raw device, reconciled with the shared mixer?
- How does capture fan out to multiple authorized listeners, and when is the microphone exclusive versus shared?
- How is audio focus decided so it follows the foreground without an app being able to grab it coercively?

## Omega Leverage

- Streams are **capabilities + domains** over shared-memory rings, audited through the same authority-flow report as every other resource ([capabilities chapter](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- The audio server is an explicit **boundary provider**: the audited edge between proved Omega code and the raw audio hardware.
- The mixer is a **machine with states** ([machines](../../../../Omega/wiki/language_guide/chapter_3_machines.md), [states](../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md)): idle, running, draining, woken on the device clock.
- The mixer interface is a **trait** ([traits](../../../../Omega/wiki/language_guide/chapter_13_traits.md)) any component can implement, so sub-mixing is one interface with many implementations, resolved from the client's environment ([modules & imports](../../../../Omega/wiki/language_guide/chapter_14_modules_imports_visibility.md)).
- Buffer formats (sample rate, channel layout, frame size) are **wire data** ([wire protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md)) shared by client and server.
- Omega does not itself provide a real-time latency guarantee; that is a scheduler obligation Cathedral defines on top of the language ([[scheduler_and_resources]]).

## Open Questions

- Can a userspace mixer reach professional-audio latencies on commodity hardware, or is a tighter path needed for the lowest-latency clients?
- Should spatial audio and room modeling live in the server, in a system filter graph, or in the app, and who owns the listener model?
- How are system sounds and alarms made un-suppressible without becoming a channel that apps learn to abuse?

## Related
- [[media_and_graphics]] — the display and GPU half of media; audio is the symmetric temporal half.
- [[ipc_and_service_invocation]] — the shared-region primitive that carries audio streams.
- [[scheduler_and_resources]] — the real-time class and budgets the mixer depends on.
- [[time_and_clocks]] — the clock behind latency, buffering, and A/V sync.
- [[human_permission_ux]] — microphone consent, the live indicator, and synthetic (blank) capture.
- [[driver_model]] — sound devices as components.
- [[windowing_and_compositor]] — the compositor whose mixing and recursive-composition pattern audio mirrors.
