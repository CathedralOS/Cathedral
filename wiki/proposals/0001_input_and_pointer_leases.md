# Input sources and scoped pointer leases

The accepted [input foundation](../spec/human_surface/input.md),
[channel contract](../spec/human_surface/input_channels.md) and
[focus contract](../spec/human_surface/focus_routes.md) own delivery, registration,
exclusive group control, channel association and recursive local selection.
They supersede conflicting seat or focus assumptions below. This proposal now
explores remaining attribution, protected indicators and protocol details.
The [presentation contract](../spec/human_surface/rendering.md) now also owns
accepted-layout hit-testing and pointer-target cursor preference/suppression.

The platform should route input through logical pointers that can be driven by
authorized sources. A hardware mouse is one source. A Matrix can delegate a
pointer within its own subtree without handing an app the operator's mouse or
keyboard focus.

| Status | Scope |
| --- | --- |
| Proposed | Candidate architecture, not an accepted Cathedral contract |
| Specification owners | Partial [input](../spec/human_surface/input.md) and [compositor/seat](../spec/human_surface/compositor_and_seat.md) contracts; this proposal's remaining choices are not accepted |
| Implementation evidence | [Rust pointer lab](../../source-rs/distribution/lab/compositor/POINTERS.md) exercises one physical and one scoped synthetic pointer; production seats, multiplexing and recursive leases are absent |

## Separate the objects

The [windowing design](../design/part_6_human_surface/00_windowing_and_compositor.md)
already anticipates multiple labeled cursors, agents and remote collaboration.
The accepted contracts separate motion, visual presence, keyboard selection
and permission to act; a cursor does not require a mandatory full seat.

| Object | Responsibility |
| --- | --- |
| Input source | Produces normalized events from a device, agent, remote participant or replay |
| Source binding | Grants a source permission to drive a particular logical pointer; retains trusted provenance |
| Logical pointer | Tracks identity, position, button state and capture within an authorized coordinate space |
| Cursor visual | Presents the pointer's sprite, hotspot, visibility and attribution |
| Pointer lease | Delegates specified control or observation rights within a Matrix subtree and lifetime |
| Focus group | Accepted local association of channels and selection; see the [focus contract](../spec/human_surface/focus_routes.md) |

A pointer's identifier is not a capability. Knowing it does not grant motion,
button injection, observation or capture. A visual-only collaboration marker
needs no authority to activate controls. Permission to move an actionable
pointer need not include keyboard focus or the right to drive another pointer.

```mermaid
flowchart LR
    hardware[Physical input provider] --> bindings[Platform source bindings]
    virtual[Agent or remote source] --> bindings
    bindings --> pointer[Logical pointer state]
    pointer --> routing[Scoped input routing]
    routing --> app[Authorized Matrix or app]
    pointer --> visual[Compositor cursor and attribution]
    assets[App cursor resource] --> visual
```

The platform owns binding, routing and attribution rules. A distribution supplies
the default device profile and presentation choices within those rules. The
kernel supplies isolation, device access and IPC mechanisms; it need not know
cursor names, pointer acceleration or which child is under a cursor.

## Default pointer and multiplexing

Startup policy can create an operator pointer and permit local mouse devices to
drive it. Another profile can give each device its own pointer. A remote session
or app can receive a separate lease without replacing that default binding.

ZII describes a useful empty state, not an ambient grant. A zero or absent pointer
lease is inert. Creating the default pointer and attaching hardware requires
established authority, consistent with the
[substrate's zero-state rule](../design/part_0_foundations/01_omega_substrate.md).

Multiplexing belongs above normalized device input. Keep source identity and
held-button state until the binding policy combines them. If two sources share
a pointer, one source releasing a button must not erase the other source's held
state. Device removal must retire that source's contribution. The contract must
choose an aggregation or exclusive-control policy and define event ordering.

Hardware origin is a property established by the trusted input path. An app
cannot manufacture it by naming a pointer "operator" or feeding a pointer that
also has a physical binding. Human-only confirmation and the reserved operator
escape must consult trusted origin and binding state, not just pointer identity.
This preserves the existing design's physical-input backstop without assuming
that every event on a mixed pointer was physically generated.

## Delegation inside a Matrix

A host can delegate pointer rights only within authority it holds. A child lease
names its subtree, coordinate space, permitted operations and lifetime. Moving
outside its bounds, reaching siblings or targeting the root's protected prompts
requires authority that the child lease does not supply.

Each hosting level applies its transforms and routing restrictions. Flattening
the scene for drawing does not bypass input interception, clipping or occlusion.
The [accepted-layout rule](../spec/human_surface/rendering.md#hit-testing-and-displayed-state)
permits displayed pixels to lag hit-testing. Each local decision uses coherent
geometry and keeps its target fixed; exact presented-frame matching and strict
screenshot-context rejection are not required.

Capture belongs to a particular pointer, target and grant. A pointer lease alone
does not hide the operator's cursor, capture another pointer or transfer keyboard
focus. Focus-on-click can be an authorized policy; it is not an automatic power
of every cursor visible within a Matrix.

For example, a collaborative editor could host three participant pointers and
one agent pointer. A participant might receive a visual marker only, while the
agent receives bounded motion and activation within the document. The local
operator keeps an independent pointer and the reserved escape path.

## Cursor attribution and indicators

The [presentation contract](../spec/human_surface/rendering.md#cursor-presentations)
now owns bounded cursor surfaces, per-pointer target preferences, no-cursor
requests and recursive owner-controlled handoff. Hiding does not imply pointer
lock or capture, and keyboard selection does not determine cursor preference.

A possible additional policy is an owner-authenticated participant label or
activity indicator. App-supplied text cannot itself establish authenticated
identity. Whether such indicators are required, which identity they expose,
and what appears for hidden targets remain in the
[owner question](../../OWNER_QUESTIONS.md#recursive-cursor-presentation-and-attribution).
A mandatory protected fallback is a proposal, not implied by ordinary cursors.

Cursor assets follow the accepted [resource lifetime](../spec/human_surface/rendering_resources.md).
Decorative pixels confer no input authority or right to cover protected output.

## Lifetime and bounded work

Pointer state and queued events must distinguish lease incarnations. Reusing an
identifier after app death must not deliver old motion or clicks to its successor.
Revocation, source loss, target loss and host death need defined capture release
and cancellation behavior. Clearing a drag must not synthesize a successful click
on a different target.

The current focus chapter says already-queued events drain. Pointer revocation
needs that reconciled with cancellation and the
[pending-IPC question](../../OWNER_QUESTIONS.md#endpoint-transport-and-revocation-of-pending-ipc).
This proposal does not decide that revocation undoes committed input. The final
contract must identify the commitment point and how held state is cleared on
both sides without authorizing new actions.

Bound pointer count, event queues, update work and cursor-resource residency
separately. A large shared bitmap is not permission to create unlimited cursors.
Motion can be coalesced only where the chosen event mode permits it; button and
capture transitions need ordered delivery or an explicit reset protocol. Reserve
enough platform capacity for operator recovery under a flooding source. Exact
ceilings and fair scheduling are implementation choices within that guarantee.

## Accepted topology and remaining presentation choices

The [focus contract](../spec/human_surface/focus_routes.md) replaces the earlier
mandatory-seat alternatives with explicit channel association and privately
controlled groups. The [channel contract](../spec/human_surface/input_channels.md)
already defines channel-tagged delivery and application-owned role assignment.
Cursor handoff and attribution remain in the
[owner question](../../OWNER_QUESTIONS.md#recursive-cursor-presentation-and-attribution).
Compatibility adapters must preserve the accepted authority and channel boundaries.

## Acceptance evidence

Before promoting this proposal to a contract, establish:

- Two physical sources sharing a pointer, and two independent pointers, with
  ordered button handling across disconnect and source rebinding.
- A visual-only lease that cannot activate controls, and an actionable child
  lease that cannot reach siblings, ancestors or protected prompts.
- Nested transforms, clipping, occlusion and scene changes with matching visible
  cursor position and routed targets.
- Capture cancellation, source/target/host death and stale queued events without
  stuck state, inherited authority or a click on a replacement target.
- App-provided cursor assets with platform attribution, bounded resources and a
  failure fallback that retains the required identity cue.
- Synthetic input on a pointer with a physical binding that cannot pass a
  physical-origin gate or suppress operator recovery.
- Explicit seat/focus and compatibility-adapter rules, with budgets that keep
  an abusive source from monopolizing routing and cursor updates.

The [Rust pointer lab](../../source-rs/distribution/lab/compositor/POINTERS.md)
supplies bounded evidence for separate physical/synthetic identities, clipping,
capture, cancellation and app/source failure. Its two app scopes, fixed cursor
visuals and boot-approved source connection do not establish production seat
semantics, multiple physical devices, protected prompts or recursive delegation.
The [session lab](../../source-rs/distribution/lab/compositor/SESSION.md) adds
separate host/child processes, explicit input regions, event waits and attenuated
keyboard-focus grants. Its child receives only Enter; synthetic clicks cannot
acquire keyboard focus, and parent loss revokes descendant input. Focus-change
queue cancellation supplies partial evidence for the accepted
[focus transitions](../spec/human_surface/focus_routes.md#selection-changes-and-outstanding-input),
not a general multi-group implementation.
The list above remains the acceptance bar. Acceptance updates the compositor/seat specification,
affected source contracts and the conflicting seat and agent design prose.
