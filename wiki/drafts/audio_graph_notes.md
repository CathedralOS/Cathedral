# Audio hierarchy and shared resources

Matrix nesting need not require a new sample buffer or mixing stage at every
ancestor. This investigation separates routing authority from sample transport
and records questions for a future audio prototype.

Status: shared-buffer playback and policy-only forwarding are now accepted in
[the playback contract](../spec/human_surface/audio.md). Retained sample assets,
general DSP graphs and capture choices below remain exploratory. The permanent
design owner is [Audio](../design/part_6_human_surface/06_audio.md).
These notes are not implementation evidence.

## Resource analogy

The [rendering proposal](../proposals/0000_rendering_and_composition.md) separates
scene descriptions from image backing. Audio can make the same distinction:

| Rendering object | Audio counterpart |
| --- | --- |
| Retained image resource | Retained sample resource, such as a decoded chime |
| Image instance and placement | Playback instance with start time, gain and channel placement |
| Updating image producer | Streaming producer with a bounded pool or ring of sample blocks |
| Group transform or opacity | Group routing, gain or mute policy |
| Intermediate rendered image | Materialized submix for an effect or downstream consumer |

Resource identity, backing custody, read access and accounting remain separate.
A playback instance need not own another copy of its sample asset. Streaming
adds ownership transitions as blocks are filled, published, consumed and reused.
Those transitions must prevent producer mutation during an immutable read lease;
shared memory alone does not provide that guarantee against native code.

## Authority tree and processing graph

An app publishes a stream through its host's authority while the eventual mixer
reads authorized backing directly. The accepted baseline permits ancestors to
contribute routing and controls without touching samples. This does not bypass
ancestor mute, revocation or separately authorized observation rights. Flattening
preserves the effective controls and their lifetimes; it is not blanket authority
to map every descendant's audio.

Introduce an actual processing node where signal semantics require one. Nested
scalar gains can often combine, subject to automation and numeric behavior.
Compression of a combined bus cannot generally be replaced by compression of
each source separately. An intermediate submix can also reduce repeated work
when many sources share an effect. Flattening every node is not the objective.

Playback and observation need distinct authority. Permission to route a stream
to speakers does not automatically authorize another app to map its samples or
record the resulting mix. Capture exclusions must be applied before an authorized
recording consumer receives samples, including when processing graphs are shared.

## Existing precedents

- [PipeWire graph scheduling](https://docs.pipewire.org/page_scheduling.html)
  separates configuration from real-time processing and establishes buffers and
  coordination state in shared memory before execution. Synchronous nodes run
  according to dependencies; asynchronous links add a graph cycle of latency.
- [XAudio2 voices](https://learn.microsoft.com/en-us/windows/win32/xaudio2/xaudio2-voices)
  separate sample sources, intermediate submixes and device output. Submixes
  support effects over combined sources and shared format conversion.
- [Web Audio buffer sources](https://www.w3.org/TR/webaudio-1.0/#AudioBufferSourceNode)
  separate reusable audio data from playback nodes, a precedent for retained
  sample resources and independent playback instances.

These are mechanism precedents. They do not establish Cathedral's capability
model, isolation guarantees or a latency target on its hardware.

## Timing and evidence still needed

The Rust lab now has [reusable shared backing](../../source-rs/platform/libraries/user-runtime/memory/shared/README.md)
and service connections. Those establish transport building blocks, not an audio
driver, streaming protocol, graph scheduler or real-time service guarantee.

A future experiment should separate transport copies, signal-processing work and
buffering delay. Measure the same sources under increasing Matrix depth, with
both policy-only ancestors and genuine effects. Report device period, clock,
buffer occupancy and underruns alongside CPU work. A smaller copy count does not
establish lower latency or punctual execution.

Define late-producer behavior, clock drift handling, buffer reuse, graph-update
commit points and cleanup on producer/consumer failure before promising seamless
playback. The device-facing path needs bounded work and ready data; an app that
misses its deadline cannot hold every other stream indefinitely. Silence or
another explicit concealment policy is a choice to test, not an implicit replay
of stale samples.

Exclusive output remains a [separate deferred decision](../../OWNER_QUESTIONS.md#exclusive-audio-and-the-reserved-system-channel).
Shared playback retains final service mixing; no raw handoff is selected.
A future proposal must distinguish single-stream service policy from device
delegation and resolve reserved-channel enforcement. Neither hypothetical handoff
nor another platform's latency result establishes a Cathedral guarantee.
