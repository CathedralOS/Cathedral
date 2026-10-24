# Input delivery and interaction

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-10-23 |

## Scope and ownership

The owner accepted this input foundation on 2026-10-23. The platform defines
common input/interaction contracts and reusable mapping libraries. Drivers decode
devices; matrices supply and attenuate input environments; applications interpret
their permitted inputs and perform work. The kernel supplies isolation, IPC,
scheduling and resource enforcement, not application action meanings.

These are responsibilities, not a required process pipeline. A cursor renderer,
mapping stage or service broker need not be an extra hop in every input delivery.
The [scoped bootstrap](../../../source-rs/platform/contracts/services/README.md)
provides authorized connections; a manifest requests interfaces but creates no
authority. There is no ambient lookup of the physical operator's input devices.

The same interfaces apply across nested matrices. A matrix may supply forwarded,
transformed or synthesized input. A child cannot bypass its owner to obtain an
upstream stream. Identifiers are local to supplied authority and do not reveal
global devices, participants or sibling state merely to support input handling.

The [input channel contract](input_channels.md) defines independently routed logical
streams, stable identities, scoped discovery and compact references. Applications
can receive multiple channels through one acceptance interface without creating
an interface or view per participant. References below to interaction contexts
describe scoped routing/interpretation state, not mandatory app-created sessions.

An input compositor/router owns the channel registry and admits authorized
producer registrations. Receiving apps publish acceptance descriptions, not the
channels they receive. Nested routers may provide independent registries; their
clients receive the supplied service rather than ambient access to an outer one.

## Preserve distinct meanings

The common vocabulary must distinguish:

- Device controls: key/button identity, axes, relative/absolute motion, contact
  lifecycle and source availability, with declared units, ranges and conventions.
- Layout-interpreted keyboard symbols.
- Text composition updates and committed text, including multi-event composition.
- Application actions and semantic operations with declared values and meaning.

Device normalization must not collapse independent controls or discard information
needed by an authorized consumer. Common device profiles need extensible control
descriptions; no finite gamepad profile covers all devices. Labels are descriptive
metadata, not control identity or authority. Physical key identity, spatial layout,
printed legend and interpreted symbol are not interchangeable. The contract does
not assume identical keyboard geometry or infer the ideal movement cluster.

Applications may bind physical controls or interpreted symbols, or consume text.
Text entry must not require every application to reconstruct layout and composition
from key presses. Layout and composition state must be scoped to the relevant
interaction context, not forced into one global keyboard state.

## Buffered delivery and current state

Input collection and delivery are independent of display frames. Consumers may
drain available events, obtain a current-state snapshot and wait for input or
lifecycle changes. Polling a buffered stream is supported; latest-state-only
sampling is not a substitute for retaining observed transitions.

For an admitted stream, the delivery contract must specify ordering, timestamp
meaning and the relationship between snapshots and event history. A timestamp
must not silently change from source-observation time to downstream receipt time.
No total physical-time ordering across independent devices is implied. Filtering
and coordinate transformations must preserve the ordering/meaning they promise.

Queues and retained history consume bounded funded resources. Overflow must be
observable and have a defined cancellation/resynchronization outcome; it cannot
silently overwrite unread button transitions while claiming continuous history.
Any coalescing policy must state what it preserves. Latest absolute position,
accumulated relative displacement and a full stroke history are different data.
Snapshots must not silently consume or replace transition history.

The guarantee starts with events observed by the provider; it cannot reconstruct
transitions absent from device reports. Nor does this contract promise unlimited
retention, lossless hardware sampling, same-frame delivery or a hard latency bound.

An application may establish standing subscriptions to permitted controls and
process them in its own update loop. Momentary gameplay acceptance conditions do
not require advertising a new binding before every input window. No mandatory
render barrier, synchronous callback or per-event broker exchange is imposed.

## Accepted interactions and target lifetime

Ordinary routed input uses an established interaction endpoint and an accepted
description of targets, operations or standing subscriptions. An advertisement
requests delivery within existing grants; it cannot widen them. A broad canvas
or controller subscription is valid. No ordinary matching input is delivered
where no accepted target/subscription exists. Lifecycle/cancellation delivery and
the owner's independent supervisory controls do not depend on such advertisement.

Descriptions may expose semantic roles, labels, values and operations in addition
to raw-event targets. Standard libraries should derive useful descriptions from
their controls. Rich semantic coverage is distinct from protocol validity; an
opaque canvas does not qualify as autonomously operable merely by accepting raw
events. App-supplied metadata is not trusted system identity or proof of cooperation.

Updates take effect atomically when the router applies them. The router targets
against its current accepted state; updates need not occur every frame and no
universal app-visible revision number is required. An action already dispatched
retains its intended target. Retired/reused identifiers must not redirect it to
a replacement element or connection. The concrete identity/retirement scheme is
not selected here.

Applications validate operations on receipt and report defined outcomes rather
than silently retargeting them. Availability may have changed since publication.
Operation-specific preconditions can protect state-sensitive work without making
every input require a global revision. Input recognition or release is distinct
from acceptance and completion of application work; retries must not imply safe
repetition of side effects.

Removing a target, source or connection must resolve active held/captured input
through defined lifecycle handling rather than stranding it. Cancellation does
not undo already executed work. The [focus transition contract](focus_routes.md#selection-changes-and-outstanding-input)
defines router-owned cancellation and held-input suppression; concrete transport
publication/closure semantics remain separate.

## Mappings, actions and recursive policy

An outer input environment, hosting matrix or application may apply mappings
within its authority. A matrix may transform input before local routing, consume
local bindings and forward the remaining permitted stream. For example, it may
turn granted stick motion into a local pointer or invoke a child's advertised
action when separately authorized to do so.

The interface must identify the representation supplied downstream. Committed
text must not accidentally undergo another keyboard-layout conversion, and mapped
actions must not masquerade as physical keys. A child may request another form,
but has no right to bypass the form or restrictions its parent supplies.

Mappings separate typed actions from device bindings. Reusable libraries may
provide active binding sets, explicit value modifiers, stateful triggers and
user-selected overrides. Priority resolves conflicts within authority; it cannot
capture an ungranted source or the protected outer recovery gesture. Custom
application processing need not execute in the privileged router, and no OS-wide
catalogue of application action names is required.

A binding set selects interpretations within an interaction context; it is not
itself a participant identity, focus grant or permission to drive another context.

Each stateful mapping owns its held-source contributions and transition behavior.
Changing bindings while a control is held must release/cancel the old interaction
or explicitly preserve it; it must not strand old state. If several sources drive
one logical control, one release cannot silently erase another's contribution.
The precise trigger vocabulary, aggregation modes and mapping-update API remain
implementation work.

Motion, button/action injection, observation and keyboard authority are distinct
grants. Visible cursor pixels alone grant none of them. Source, connection and
interaction-context identity must remain distinguishable where required for
authority and state, without mandating a complete seat for every visual marker.
Mappings and injected actions cannot manufacture authenticated physical origin
for an upstream operation. The [outer recovery path](compositor_and_seat.md#protected-outer-recovery-chrome)
remains outside a child's interception or remapping authority.

## Transport and implementation evidence

A bounded shared-memory event ring is a suitable implementation candidate, not
a frozen ABI or the exclusive transport. It must enforce slot custody, publication
ordering, peer-index validation, mapping permissions and backing lifetime under
the [IPC contract](../communication/ipc.md). Notifications may wake a consumer
without carrying every event. Unrestricted writers sharing a page are not made
safe merely by agreeing which bytes each owns.

The [Rust physical-key transport](../../../source-rs/platform/contracts/input.rs)
is a small experimental key-event ABI without the complete text/layout model.
The [matrix child queues](../../../source-rs/platform/libraries/matrix/world/children/children.rs)
buffer sixteen ordinary events per stream for four bounded streams, with one
reserved keyboard-cancellation boundary. They preserve button edges and group-local
ordering during motion coalescing and emit scoped cancellation on overflow. They retrieve
events through IPC replies, not a reusable cross-process event ring.
[Input routing](../../../source-rs/platform/libraries/matrix/world/input/input.rs)
exercises independent local targeting and held-state suppression. These are partial evidence,
not conformance to all timing, semantic and lifecycle requirements above.
Owner-only registration now has explicit channel/group retirement notifications;
retirement drains before reusing table storage. Nested routers pause upstream
input polling during that drain while continuing local dispatch. Restricted app
producer grants, discovery and per-producer quotas remain outstanding.

## Boundaries not selected here

The separate [focus contract](focus_routes.md) defines group associations and
recursive local selection. The [presentation contract](rendering.md) defines
accepted-layout hit-testing, served masked capture and pointer-target cursor
preferences. This input foundation does not settle multi-controller application
arbitration, multi-size rendering or exact scene/mask publication protocols.
The shared interaction description is not a decision on those representations.
Wire encodings, key/control registries, timestamp clocks, queue limits, snapshot
protocols, action schemas and text-input protocols remain to be specified and
tested. Existing Rust numeric constants are not promoted to production standards.
