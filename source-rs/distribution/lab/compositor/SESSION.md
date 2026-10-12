# Persistent input and hosting experiment

Three isolated apps retain their own desired state. A userspace compositor draws
their scenes, routes input and enforces delegated focus and placement authority.
The distribution supervises the session. This extends the [drawing](README.md)
and [pointer](POINTERS.md) labs; it is not a stable desktop or Matrix contract.

From the repository root:

```text
python tools/boot-harness-rs/run.py --profile tools/boot-harness-rs/profiles/session.json --release --window
python tools/boot-harness-rs/run.py --session-test --timeout 60
python tools/boot-harness-rs/run.py --session-test --release --timeout 60
python tools/boot-harness-rs/run.py --session-test --release --startup-probes --timeout 60
```

Normal startup goes straight to APP ONE and HOST containing a clipped CHILD.
The on-screen instructions identify the controls; `KEYS` names the current focus
recipient, and the status line reports recovery. The white cursor follows the
mouse; the gold cursor belongs to the host. Clicking an input region gives its
app keyboard focus. Right-dragging inside HOST drives the gold cursor.

Destructive startup checks now require `--startup-probes`. They deliberately
change scenes and replace processes, so their flashing output is not part of
ordinary interactive startup. Use `--release` for interactive runs.

Add `--pressure` to the interactive command or `--session-test` to run the
[resource-pressure experiment](PRESSURE.md) alongside this session. It is opt-in;
ordinary startup adds no hostile peer.

| Interaction | Result |
| --- | --- |
| Click ADD / CLEAR | Increment / reset that app's counter |
| Left / Right, then Enter | Select and activate a button in APP ONE or HOST |
| Enter in CHILD | Activate its selected button (initially ADD); the host delegated Enter only, so arrows do nothing |
| Drag below the counter | Move a dark square; capture keeps delivery with its owner outside the panel |
| Click a title or decorative background | No app input target or keyboard focus |
| F4 | Replace the compositor; all three app processes and their counters/drag content survive |
| F5 | Exit and replace the input source; cancel input and focus |
| F6 | Replace HOST; CHILD survives, hides while its host is absent, then is placed and authorized again |
| F7 | Ask HOST to click its own ADD through its synthetic pointer; keyboard focus stays unchanged |
| Escape | Cancel physical capture and keyboard focus |
| Hold left and right in APP ONE, with `--startup-probes` | Opt-in app fault; supervision replaces it |

After cancellation, release held buttons and keys. A neutral mouse report must
arrive before another press is accepted. Replacement apps start fresh; surviving
apps keep state in their own RAM. This is not persistence across power loss.

## Follow execution

The [profile](../../../../tools/boot-harness-rs/profiles/session.json) supplies
separate app, display and input connections. Start at [main.rs](main.rs), then
[supervisor entrance](supervisor/mod.rs), which selects the [desktop session](supervisor/desktop/mod.rs).
That coordinator admits participants, establishes grants, optionally checks
boundaries and idle behavior, then supervises health. [Recovery](supervisor/desktop/recovery.rs)
owns replacement and reconnection. [Chrome](supervisor/desktop/chrome.rs) supplies
instructions, focus labels and recovery notices through bounded retained control
text; the platform owns no distribution wording. [Client session](client/session/mod.rs) waits,
handles control and input, and publishes desired state; its `events` and
`presentation` modules separate behavior from appearance.

Across IPC, the [compositor](../../../platform/services/display/compositor/mod.rs)
dispatches requests, delivers pending waits and observes disconnects. Its
[session dispatch](../../../platform/services/display/compositor/session.rs)
authenticates focus and hosting operations. Follow
[focus](../../../platform/services/display/composition/input/focus/mod.rs) for attenuation and held-key
rules, [hosting](../../../platform/services/display/composition/scene/hosting.rs) for the process
tree and clips, and [scene](../../../platform/services/display/composition/scene/mod.rs) for
snapshot validation, explicit input regions and drawing. The shared
[interaction contract](../../../contracts/interaction.rs) imports no implementation.
The [input source](../../../platform/services/input/pointer.rs) owns PS/2 decoding
and normalized source updates; apps cannot acquire its role by sending an opcode.

## Authority and presentation

Focus grants bind a client connection incarnation, permitted key mask and
delegation right. A host can delegate a subset to its direct child and select
focus within its already active subtree. It cannot take focus from a sibling or
turn its synthetic pointer into physical keyboard authority. Trusted physical
clicks choose among authorized targets. Revoking a parent also invalidates its
descendants. Only the five navigation keys are modeled; the child gets Enter
without delegation authority. The single focus context does not establish seats
or principal attribution.

On a focus change, undelivered old key events become a cancellation barrier.
Held keys cannot activate the next target until released. Source loss and queue
overflow also cancel input state. An event already published into IPC may still
be consumed; completed effects are not undone. This differs from the old design's
queue-draining prose, recorded in
[the owner queue](../../../../OWNER_QUESTIONS.md#focus-changes-and-undelivered-key-events).
Routing grants remain installed while inactive; switching focus changes the
active recipient rather than minting a new grant on every click.

HOST has a separate placement grant for CHILD. CHILD submits its scene directly
to the root compositor over its own connection. The root translates and clips
it against HOST; no child-to-host pixel buffer copy is needed. HOST's 200x160
child placement at local (100,80) clips to 156x120 inside its 256x200 scope.
Host loss hides CHILD and revokes descendant input, but does not kill CHILD:
the distribution owns both process lifetimes. Reconnection explicitly regrants
authority. Fixed slot order prevents cycles and determines stacking; arbitrary
reparenting, Matrix effects and general recursive hosting remain future work.

Each scene has at most 16 nodes and one 16x16 RGB image. ASCII text scales from
one to four; the interactive controls use scale two. The 1936-byte snapshot
fits one sealed page and is copied into bounded compositor storage. Explicit
INPUT nodes define hit regions independently of painted pixels; ancestor clips
still apply. Legacy scenes without them retain painted-pixel hit testing. The
root draws GOP directly, so this does not establish zero-copy image resources,
atomic frames, vblank scheduling or GPU presentation.

[Rasterization](../../../platform/services/display/composition/scene/raster.rs) paints spans
into a fixed 256-pixel scratch row rather than walking the entire scene for every
pixel. Snapshot comparison bounds repaint damage; unchanged snapshots cause no
scanout writes. No full-frame backing or ancestor image copies are introduced.

## Waiting and recovery

Apps keep one outstanding WAIT and sleep on their control/reply handles. The
compositor retains at most one pending waiter per app and replies when an event
exists. A supervisor WAKE supports configuration rendezvous. Apps serialize
configuration RPCs after consuming their wait reply; arbitrary concurrent RPCs
on one reply queue are not supported.

The kernel's bounded IPC_WAIT_MANY copies and validates one to eight receive
handles, checks every handle's authority before reporting readiness, and returns
an index without consuming a message. The generic server waits on all connected
clients plus control. It still wakes periodically to discover replacement links;
the PS/2 source uses bounded device/control polling. The idle probe establishes
no repeated app event polling or compositor redraws, not zero timer wakeups.

On compositor failure the supervisor reconnects surviving apps, has them publish
their desired state, rebuilds placements and grants, and reconnects the input
source. The surviving source supplies its held-key state so a repeat cannot
become a new press after reconnection. Cursor positions restart; interrupted
capture and focus are cleared.
The old interaction is never resumed merely because its content survived.
If input delivery fails, source health reports `offline` instead of claiming
readiness with a dead connection. Supervision can then replace it. Ordinary
pointer motion no longer writes serial diagnostics for every report or moves the
synthetic cursor; button edges and explicit synthetic actions remain observable.

## Evidence and limits

Host tests cover attenuation, denied widening and sibling focus, parent
revocation, incarnation changes, held-key suppression, overflow, explicit hit
regions and clipped child placement. Guest probes exercise wait-array copy
bounds, complete handle validation, a later ready receiver and non-consumption.
The idle probe compares event, wake, revision and redraw counters.

The [QEMU test](../../../../tools/boot-harness-rs/session_test.py) injects real
emulated input, exercises counters and capture, replaces the compositor during a
drag, replaces HOST without killing CHILD, and replaces the input source while
Enter is held. It checks preserved app identities/state, child-only key rights,
synthetic focus isolation and every scanout pixel at several stages. The final
image is `build/boot-harness-rs/session-test/display.png`.

The test also sends 120 relative motion reports at roughly 100 Hz while dragging,
checks exact final position and release delivery, rejects cancellation/recovery,
and checks keyboard input afterward. Timing goes to `latency.json`. This caught
input loss that the earlier slow, individually acknowledged gestures missed.
On the development host, the old release build lost 22 motion steps in this
workload; the revised build delivered all 120 and its final cursor was visible
about 16-31 ms after the last send in release runs, including the QMP scanout query. These are
workload observations, not per-event latency or a desktop performance guarantee.

With `--startup-probes`, the guest also times eight full 1024x768 repaints using
its 10-ms clock. The observed release run took three ticks in total, including
IPC. That probe repaints the whole screen regardless of damage, so it separates
basic raster throughput from the benefit of small damage rectangles. It measures
neither vblank latency nor scanout atomicity.

This profile has three app slots, two pointers and eight-event queues. It does
not implement text input/IME, USB, multiple seats, trusted prompts, arbitrary
service discovery or a production ABI. F4-F7 are lab controls on the trusted
source connection, not an application global-hotkey contract.
