# Follow the running distribution

Start at [init/main.rs](init/main.rs), then [session.rs](init/session.rs).
The session starts and readies providers, launches counter and status, then enters
[supervision](init/session/supervision.rs). Provider protocol details live under
[providers](init/session/providers.rs); child cancellation, collection and spawn
live in [child.rs](init/session/child.rs).

[profile.json](profile.json) is the host's executable composition. The harness
builds those packages and supplies their ELF artifacts to
[kernel startup](../kernel/boot/uefi/startup.rs). The kernel sees supplied code
and grants; the distribution gives those children their roles.

| Role | Launch / task slot | Executable entrance | Protocol / implementation |
| --- | --- | --- | --- |
| Init | Initial / 0 | [init/main.rs](init/main.rs) | [App control](init/session/supervision/control.rs), [session protocol](libraries/session-protocol/lib.rs) |
| Display | 0 / 1 | [display/main.rs](../platform/services/display/main.rs) | [Drawing contract](../contracts/display.rs), [surface](../platform/services/display/surface.rs) |
| Input | 1 / 2 | [input/main.rs](../platform/services/input/main.rs) | [Event contract](../contracts/input.rs), [decoder](../platform/services/input/decoder.rs) |
| Application | 2 / 3 | [status/main.rs](applications/status/main.rs) | [Application workflow](applications/status/application.rs) |
| Counter | 4 / 5 | [counter/main.rs](applications/counter/main.rs) | [Private counter workflow](applications/counter/session.rs) |
| Storage | 3 / 4 | [storage/main.rs](../platform/services/storage/main.rs) | [Object contract](../contracts/storage.rs), [service](../platform/services/storage/service.rs), [store](../platform/libraries/object-store/lib.rs) |

The app's [connections](applications/status/application/connection.rs) use a
private init control pair plus three peer links, ordered display, input, storage.
The corresponding names live in `session-protocol::launch` and `::link`; their
indices must match the profile. Follow [input](applications/status/application/input.rs),
[storage](applications/status/application/storage.rs) or
[view](applications/status/application/view.rs) to see the app-side protocol use.
The kernel's [IPC dispatcher](../kernel/core/user_tasks/session/syscalls.rs)
transports these messages; the service implementations interpret them.

For the pixel handoff, descend from `view` into
[buffer.rs](applications/status/application/view/buffer.rs), then follow the
[runtime ownership API](../platform/libraries/user-runtime/memory.rs) and
[display buffer consumer](../platform/services/display/surface/buffer.rs).
The profile grants status four private pages and display four accepted shared
pages. The `SHARED PIXELS` tile uses one page per redraw; the display completion
precedes owner release. The kernel has no knowledge of pixels or screen layout.

`lab/` contains guest test executables selected by the profile's `user_programs`
section for smoke runs. It is distinct from the ordinary `applications/` path.
The minimal example profile also uses the lab's hello executable as its only task.

The [rendering investigation](lab/rendering/README.md) has two entrances:
[bench.rs](lab/rendering/bench.rs) compares native host pixel/storage costs, and
[main.rs](lab/rendering/main.rs) dispatches guest page-backed rendering and bounds
probes. Its shared algorithms live at [lib.rs](lab/rendering/lib.rs); ordinary
applications and platform services do not depend on this research fixture.

Storage accepts status on its first data link (root 0) and counter on its second
(root 1). Counter has only that one data link. These are fixed persistent root
bindings for this profile; reordering storage links on an existing disk is
unsupported. [The catalog service](../platform/services/storage/CHARTER.md)
explains the authority and transaction boundaries. Init checks counter through
[counter supervision](init/session/counter.rs), independently of the status app.

The separate [compositor profile and experiment](lab/compositor/README.md) starts
its own supervisor and two isolated scene clients. Follow its coordinator into
drawing and recovery checks, then the display service's compositor dispatch.
It exercises retained scopes without changing this ordinary status composition.
