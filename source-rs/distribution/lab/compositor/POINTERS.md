# Logical pointer experiment

The pointer profile adds a mouse provider and two root-rendered cursors to the
[retained compositor lab](README.md). The kernel transports tagged controller
bytes. The platform owns decoding, pointer state, targeting and capture. The
distribution chooses placement, grants the synthetic pointer and restarts failed
apps. This supplies experimental evidence without accepting a seat contract.
The newer [session profile](SESSION.md) builds on this with attenuated keyboard
focus, explicit input regions, a hosted child and live compositor recovery.

From the repository root:

```text
python tools/boot-harness-rs/run.py --pointer-test
python tools/boot-harness-rs/run.py --pointer-test --release
python tools/boot-harness-rs/run.py --profile tools/boot-harness-rs/profiles/pointer.json --window
```

Wait for the two overlapping panels after the startup probes. White cursor `1`
follows QEMU's mouse; gold cursor `2` belongs to app two. Both are drawn by the
compositor. Dragging paints a small dark square at the last delivered location.

| Interaction | Result |
| --- | --- |
| Left drag in either app | That app requests capture; moving over its neighbor keeps delivery with the drag owner |
| Right drag in app two | App two drives cursor `2`, offset by 40 pixels and clamped to its own scope |
| Escape | Input provider cancels the physical drag; release held buttons before starting another |
| Left and right held in app one | Deliberate app fault during capture; the supervisor replaces it |
| Middle press in app one | Deliberate IPC flood; a health deadline causes replacement |
| F5 | Deliberate mouse-provider exit and replacement; active physical capture is cancelled |

The cursor can continue moving while button delivery is suppressed after a
failure. A neutral mouse report clears suppression. This prevents a held button
from becoming a new press on a replacement app. These failure gestures belong
only to this profile; the standard status distribution keeps its keyboard demo.

## Follow the path

The [profile](../../../../tools/boot-harness-rs/profiles/pointer.json) gives the
input process exclusive controller access and its own connection to display.
Apps have separate connections and no device grant.

1. Kernel [controller transport](../../../kernel/core/user_tasks/session/syscalls/keyboard.rs)
   preserves the auxiliary-port tag. Existing timer polling drains mouse bytes;
   this experiment does not add IRQ12 handling. The historical `keyboard` grant
   and syscall names now cover both ports of this single PS/2 controller.
2. Input [startup](../../../platform/services/input/main.rs) selects
   [mouse mode](../../../platform/services/input/pointer.rs). The
   [controller](../../../platform/services/input/controller.rs) selects standard
   three-byte stream packets, decoded by [mouse.rs](../../../platform/services/input/decoder/mouse.rs).
3. Display [input dispatch](../../../platform/services/display/compositor/input.rs)
   authenticates source and app roles from accepted connections. The experimental
   [wire contract](../../../contracts/pointer.rs) imports no implementation.
4. [Pointer state](../../../platform/services/display/composition/input.rs) owns leases and
   lifetime. [Routing](../../../platform/services/display/composition/input/routing.rs)
   selects targets; [cursor rendering](../../../platform/services/display/composition/input/cursor.rs)
   overlays bounded root-owned pixels after app drawing.
5. Distribution [supervision](supervisor/pointer.rs) grants and recovers the lab;
   [interactive clients](client/interactive.rs) consume local events and update
   their retained content.

## Experimental rules

This profile uses two pointers and two app event queues. Each queue holds eight
events. Adjacent motion for the same pointer coalesces; button edges do not.
Overflow replaces that app's pending events with cancellation barriers, releases
its captures and suppresses held presses until buttons are up. Other clients
retain their state. The service rotates control, both apps and the mouse source;
a full reply queue cannot block another connection.

Hit-testing selects the topmost painted pixel in the same accepted scene used
for rendering. Ancestor clips and app placement apply to both paths. Transparent
text gaps can target content underneath: this is a lab hit policy, not a decision
about production input regions. Events retain the signed wire fields but coordinates are clamped to the granted
app-local viewport. Cancellation carries zero coordinates. Capture preserves
delivery when the physical cursor leaves the app without disclosing its outside
position; without capture, held motion stays
inside the current target, while button edges return to the original press owner.
Releasing all buttons ends capture. Moving an app cancels its active drags.

Only the designated source connection can update physical pointer `1`. An app
cannot acquire that role by naming a pointer. Control can grant one synthetic
lease to a visible app; its token is bound to the app connection incarnation.
Local coordinates clamp to that scope. Synthetic hit-testing cannot reach a
neighbor, and its cursor is clipped by scope and covering apps. A revoked token
cannot drive its replacement. The wire uses IDs `0` and `1`; displayed tags are
`1` and `2`. Neither ID carries authority by itself.

Source loss cancels physical capture. App disconnect clears its queue and
revokes its synthetic lease. Cancellation discards that pointer's undelivered
events and queues a cancel barrier where needed, including when a release was
queued after a delivered press. Already delivered effects are not undone.
This timing is an experiment; production pending-IPC revocation remains open.

Pointer position and cursor damage are processed with the compositor's accepted
scene. Small old/new cursor regions redraw from retained content; app submission
redraws cannot erase a cursor. Writes still go directly to GOP and may tear.
Physical and synthetic tags distinguish the two lab identities, not verified
human principals or protected UI attribution.

## Evidence and remaining scope

Host tests cover signed packet decoding, source tags and loss, nested clips,
overlap, capture, queue overflow, queued-release revocation, independent pointer
state and covered synthetic cursors. QEMU injects real emulated mouse reports and
checks local delivery, cross-app dragging, Escape, stale/wrong-owner lease denial,
source replacement, app fault and flood recovery. The root cursor progresses
during the flood. The host independently reconstructs all 786432 final pixels,
including cursor overlays, retained app content and exposed backgrounds.

There is one physical device and one synthetic source, not source aggregation,
USB, hotplug, scrolling or acceleration. There are no seats, keyboard focus,
recursive Matrix delegation, visual-only leases, custom cursor assets or trusted
prompts. Provider reconnect after compositor failure remains the base lab's
pre-input probe; this interactive profile supervises app and mouse-provider loss,
not live compositor replacement. Source health and live compositor replacement
are exercised by the separate session profile. The [pointer proposal](../../../../wiki/proposals/0001_input_and_pointer_leases.md)
and [owner question](../../../../OWNER_QUESTIONS.md#pointer-leases-seats-and-action-authority)
retain those production choices.
