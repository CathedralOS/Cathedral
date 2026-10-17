# Follow the running distribution

Start at [init/main.rs](init/main.rs), then [session entrance](init/session/session.rs).
The session starts and readies providers, launches counter and status, then enters
[supervision](init/session/supervision/supervision.rs). Provider protocol details live under
[providers](init/session/providers/providers.rs); child cancellation, collection and spawn
live in [child.rs](init/session/child.rs).

[profile.json](profile.json) is the host's executable composition. The harness
builds those packages and supplies their ELF artifacts to
[kernel startup](../kernel/boot/uefi/startup.rs). The kernel sees supplied code
and grants; the distribution gives those children their roles.

| Role | Launch / task slot | Executable entrance | Protocol / implementation |
| --- | --- | --- | --- |
| Init | Initial / 0 | [init/main.rs](init/main.rs) | [App control](init/session/supervision/control.rs), [session protocol](libraries/session-protocol/lib.rs) |
| Display | 0 / 1 | [display/main.rs](../platform/services/display/main.rs) | [Drawing contract](../platform/contracts/display/display.rs), [direct service](../platform/services/display/direct/direct.rs), [surface](../platform/services/display/surface/surface.rs) |
| Input | 1 / 2 | [input/main.rs](../platform/services/input/main.rs) | [Event contract](../platform/contracts/input.rs), [keyboard service](../platform/services/input/keyboard/keyboard.rs), [decoder](../platform/services/input/decoder/decoder.rs) |
| Application | 2 / 3 | [status/main.rs](applications/status/main.rs) | [Application workflow](applications/status/application/application.rs) |
| Counter | 4 / 5 | [counter/main.rs](applications/counter/main.rs) | [Private counter workflow](applications/counter/session/session.rs) |
| Storage | 3 / 4 | [storage/main.rs](../platform/services/storage/main.rs) | [Object contract](../platform/contracts/storage.rs), [service](../platform/services/storage/service/service.rs), [store](../platform/libraries/object-store/lib.rs) |

The app's [connections](applications/status/application/connection.rs) use a
private init control pair plus three peer links, ordered display, input, storage.
The corresponding names live in `session-protocol::launch` and `::link`; their
indices must match the profile. Follow [input](applications/status/application/input.rs),
[storage](applications/status/application/storage.rs) or
[view](applications/status/application/view/view.rs) to see the app-side protocol use.
The kernel's [IPC dispatcher](../kernel/core/user_tasks/syscalls/syscalls.rs)
transports these messages; the service implementations interpret them.

For the pixel handoff, descend from `view` into
[buffer.rs](applications/status/application/view/buffer.rs), then follow the
[runtime ownership API](../platform/libraries/user-runtime/memory/memory.rs) and
[display buffer consumer](../platform/services/display/surface/buffer.rs).
The profile grants status four private pages and display four accepted shared
pages. The `SHARED PIXELS` tile uses one page per redraw; the display completion
precedes owner release. The kernel has no knowledge of pixels or screen layout.

`lab/` contains guest test executables selected by the profile's `user_programs`
section for smoke runs. It is distinct from the ordinary `applications/` path.
The minimal example profile also uses the lab's hello executable as its only task.

The [rendering investigation](lab/rendering/README.md) has two entrances:
[bench entrance](lab/rendering/bench/main.rs) compares native host pixel/storage costs, and
[main entrance](lab/rendering/main.rs) dispatches guest page-backed rendering and bounds
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
Its [persistent session profile](lab/compositor/SESSION.md) adds a third app
hosted inside another app's scope, attenuated focus and live compositor recovery.
The distribution still owns all process lifetimes; display enforces scene and
input grants independently of those lifecycle rights.

The [matrix boundary lab](lab/matrix/README.md) now lends launch budgets to the
root matrix, which admits its own children. The inner matrix admits its leaf;
the fixture holds only root control and has no descendant process handles.
The [visible matrix policy](lab/matrix/desktop/desktop.rs) uses that same platform tree
for local drawing, physical pointer input and focus through the existing
compositor. See the [implementation map](../STATUS.md) for commands and limits.

The [device lifecycle lab](lab/devices/README.md) pairs a userspace supervisor
with an interrupt-driven serial driver. The kernel grants only the child access
to COM2/IRQ3, and the supervisor replaces it after faults and blocked waits.

The [shared-memory lab](lab/shared-memory/README.md) forwards one sealed object
through three processes on both CPUs. The matrix desktop uses the same ordinary
IPC mechanism for retained bitmaps, with explicit child memory budgets.

The [ordinary channel lab](lab/channels/README.md) remains the focused proof for
runtime endpoint moves, startup environments and recursive launch-loan lifetime.
Its [scenario](lab/channels/scenario.rs) descends into each of those mechanisms.

The [private content lab](lab/objects/README.md) composes the experimental database
provider with two explicitly connected storage clients. Its supervisor owns process
lifetimes and crash-test milestones; the platform provider owns content publication
and the disk. This is separate from the default desktop catalog composition.

The separate [notes application](applications/notes/README.md) starts at
[main.rs](applications/notes/main.rs) and [session.rs](applications/notes/session/session.rs).
The `notes.json` composition selects it beneath ordinary root/nested matrix services;
it does not import the distribution lab or any service implementation.
