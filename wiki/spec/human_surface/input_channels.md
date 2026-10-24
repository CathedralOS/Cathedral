# Input channels and discovery

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-10-23 |

## Channel model

The owner accepted this refinement of the [input foundation](input.md) on
2026-10-23. A logical input channel identifies an independently routed control
stream within supplied authority. It is not a user account, device identity,
application view or rendering frame. It is also distinct from a kernel IPC channel:
several logical input channels may share one IPC connection or event ring.

The protocol supports a resource-bounded number of channels, not two built-in
human/agent classes. Sources may be physical devices, agents, remote connections
or other authorized producers. An owner can bind two controllers to distinct
channels, combine keyboard and mouse into one, or authorize a source to drive
several channels. Multiple-source bindings must preserve contribution state as
required by the input foundation.

A matrix may present several authorized upstream sources as one downstream
channel or switch the source driving a stable channel. The child sees the
supplied virtual stream, not necessarily the physical devices or participants.
This cannot forge upstream authority or authenticated physical origin. Source
switching and merging retain held-state, ordering and cancellation obligations;
sharing an identity does not create independent app interaction state.

All channels follow the same authority rules. There is no privileged default
channel. Startup policy may establish a channel driven by physical devices;
absent or zero-initialized bindings confer no implicit operator input authority.
Authenticated physical origin, where required, comes from the trusted source
path and is not inferred from a channel's name, identity or default designation.

## Producer registration and registry ownership

The input compositor/router owns its channel registry. Authorized producers
register channels with that service through IPC and receive scoped authority to
submit input for the admitted channels. A keyboard driver, gamepad driver, agent
or another input router may be a producer. Registration and event submission are
capability-gated and resource-bounded; publishing descriptor bytes or claiming a
channel identity does not confer either authority.

Receiving applications do not publish the channels they consume. Their acceptance
description says what input they can handle; producer registration establishes
the supplied channels. An ordinary program can hold both roles if separately
granted both. Registration, submission, directory observation and event consumption
are distinct rights, even when bundled in one service connection.

Registry mutation remains owned by the service. It can grant consumers read-only
shared registry views for cheap lookup while handling registration and updates
through its protocol. A mapped view must contain only metadata the recipient may
observe; mapping a global table and merely asking readers to ignore other rows
does not provide isolation. No fixed single-page capacity or shared writable
registry is implied.

The outer hardware-owning environment supplies physical drivers with registration
authority to its input compositor as appropriate. There is no ambient or privileged
lookup of a universally "real" input compositor. A nested matrix can run its own
input compositor with independent registrations, synthetic producers and authorized
forwarding from its parent. It may run its own drivers against delegated physical
devices or virtual interfaces. Input registration grants no hardware access by
itself and cannot bypass the outer recovery path or upstream provenance rules.

## Acceptance and application roles

The [focus contract](focus_routes.md) defines privately controlled groups that
associate permitted channels and compose local selections into recursive routes.
Channel registration alone grants no control over another group's selection.

An application may publish one acceptance description and receive channel-tagged
events through it. It need not create a bespoke interface, view or context for
every new participant. Modalities describe accepted forms of input; channel count
describes independently routed streams. Those are separate dimensions.

The application's own policy maps channels to player slots, carets, a common
selection or an exclusive controller. Independent channels do not require the
application to provide independent UI state or unlimited concurrent control.
Capacity and contention must have defined outcomes. An app can use the same
behavior for all admitted channels or retain per-channel state when needed.

The routing owner admits producer registrations and source bindings within its grants;
applications assign their own roles. Knowing or writing a channel ID into event
bytes does not authorize driving it. The receiving boundary authenticates the
sender's permitted channels. This does not require an inner app to see upstream
identities or the physical device topology.

## Identity and lifetime

A channel has an opaque stable identity that is never silently reused for another
channel in the authority scope exposing it. No integer, UUID width, generation
scheme or globally discoverable identifier is prescribed. Nested owners may use
local identities without revealing an ancestor's identifiers.

Temporary source loss and channel retirement are distinct. An authorized owner
may preserve an unavailable logical channel and later bind an approved source,
or retire it and introduce a new channel. A port change or matching friendly name
does not establish continuity by itself. This is declared owner policy, not a
universal automatic reassignment rule.

Retirement permanently ends that channel identity. Rebinding a preserved channel
must cancel or resynchronize the previous source's held state and distinguish
in-flight input from the old binding. Delayed events must not be attributed to a
replacement source as though they were newly generated by it. New channels have
new identities; app-held references cannot silently identify replacements.

## Scoped discovery

The input service supplies a directory of the channels granted to its consumer,
plus lifecycle/change notifications. Descriptions can provide friendly names,
supported controls and labels, interpretation metadata and availability. They
describe the supplied interface, which may be transformed or synthetic, not a
claim to reveal the underlying physical hardware. Names are not credentials.

Metadata needed to interpret already granted input can accompany that authority;
it need not require separate permission or a round trip per event. Listing
unrelated devices or private source details requires separate authority.

Initial enumeration and ongoing notifications must compose without missing a
change. Overflow/reconnection recovery must reestablish channel bindings and
availability as well as input state; stale directory data cannot silently restore
retired input. Exact snapshot and change-notification encodings remain engineering
work under these requirements.

## Compact events and shared descriptions

Routine input events carry a compact channel reference and event data, plus the
timing/order information required by their stream contract. They need not repeat
names, descriptors or a full unique identity. Descriptions are established or
queried separately and may be cached by the consumer.

An explicitly granted read-only shared channel table beside an event ring is a
supported implementation direction. IPC access alone does not grant a mapping.
Variable-sized metadata can live separately; mapped references use validated
offsets/handles rather than provider-local pointers. Reads and updates must follow
the [shared-memory custody contract](../communication/ipc.md).

A stream-local index is an encoding of identity, not identity itself. Reuse must
preserve the meaning of unread events and retained references, using a concrete
safe retirement/binding scheme. Table identity/descriptor records must remain
stable for valid readers; changes require safe publication. The service cannot
overwrite an entry still referenced by old events and call it a new channel.
Directory storage, retained records and event capacity remain funded resources.

## Implementation boundary

The Rust lab's [physical-key transport](../../../source-rs/platform/contracts/input.rs)
and [matrix queues](../../../source-rs/platform/libraries/matrix/world/children.rs)
do not implement this general channel directory or shared-table/ring interface.
The matrix transport can now multiplex four bounded streams with child-local
channel and group names. Channels may share selection while keeping independent
capture/held state. Queues preserve ordering across channels in one group and
fairly serve independent groups. Its host tests exercise isolation, overflow,
selective cancellation and nested forwarding. Names are not reused within a view
binding. Owner-authorized registration and explicit channel/group retirement now
reclaim bounded table slots without reusing identities. Local and inherited names
have separate internal namespaces. Host and IPC proofs cover repeated retirement,
receiver cleanup and retained group selection between channel replacements.
Upstream polling pauses while descendant retirement boundaries drain; this
introduces backpressure on that shared path, not a production latency guarantee.
Explicit owner-granted child producers now have reserved quotas, connection/view
checks, per-child generations, private channel ordinals and disconnect retirement.
The stock root can grant this; nested stock hosts refuse local producers until
their upstream channel ceiling is negotiated. Root-produced events still recurse.
Manifest-based producer acquisition, recursive allowance negotiation,
discovery and reassociation of an existing stream remain unimplemented. F5 supplies a pointer channel
and a keyboard channel in one group. Shared app-side reception tracks pointer
edges separately; this does not impose separate document/player state on apps.
The [pointer lab](../../../source-rs/distribution/lab/compositor/POINTERS.md) is
earlier bounded routing evidence, not conformance to this channel contract.

Exact encodings, table publication, reclamation and resynchronization algorithms
remain to be implemented and tested. This acceptance does not select focus topology,
per-channel view arrangements, minimization behavior or rendering policy. Those
remain separate contracts; general channels do not imply automatic app-created views.
