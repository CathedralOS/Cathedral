# Matrix mechanisms

Start at [lib.rs](lib.rs), then [world/world.rs](world/world.rs): accept a parent
grant, place local children, invalidate revoked views, and dispatch local requests.
The owner chooses the child count through `World<N>`. No slot has a launcher,
application, login or desktop role.

[Local commands](world/commands.rs) dispatch placement/revocation and input after
the program authenticates its owner/source endpoint. Child requests use a separate
dispatcher and cannot select these operations. Both the stock matrix and the
[independent three-peer program](../../../distribution/lab/matrix-composition/README.md)
use this machinery with the existing runtime IPC server.

- [Input](world/input/input.rs) admits up to four local or inherited streams,
  each with separate capture and held-input state. Explicit
  [group associations](world/input/groups.rs) let several channels share selection.
  Pointer hit testing
  follows the owner's surface ranges; equal ranges use slot order for input-only
  children. [Children](world/children/children.rs) assign names local to their view binding
  and drain independent groups fairly, preserving order across channels within
  one group. Each stream has a sixteen-event [FIFO](world/events.rs), plus one
  reserved keyboard-cancellation boundary.
  Trailing motion coalesces only within that stream and button state, retaining
  its first report and newest coordinates; another channel's event in the same
  group prevents coalescing across it. Overflow cancels that stream across
  its recipients without deleting another stream's events. Cancellation preserves
  local selection; revoking the selected child forgets it. An inherited CANCEL
  resets held state because its sender suppresses residual holds until release.
  Local source interruption performs that suppression here instead.
- [Input directory](world/children/directory.rs) enumerates only channels already
  routed into the requesting child's view. Revision-checked reads produce a
  coherent snapshot or BUSY; the existing wake hint signals changes. Retiring
  rows remain until their boundary drains. Reading neither consumes input nor
  grants injection. The [runtime reader](../user-runtime/matrix/input.rs) bounds
  one attempt and leaves retry timing to its caller. Descriptions carry a bounded
  UTF-8 label and advisory pointer/key categories. Producers publish only for
  their own channel ordinals before delivery; the [description owner](world/input/descriptions.rs)
  refuses changes to a live description or a late first publication. A separate
  retained DESCRIBED boundary precedes the first ordinary event per receiving
  edge and survives overflow/retirement; nested routers retain it without another
  lookup or per-event payload. Channels never routed to a child remain private.
  Directory reads include these immutable descriptions under the same revision.
  General control vocabularies, mutable descriptions, independently delegated
  registry access and shared read-only tables remain future work; the directory's
  key field is only the receiver's view-wide ceiling. Labels are not credentials.
- [Reception](reception.rs) tracks app-side pointer edges per scoped channel.
  Consume an event once before dispatching it to widgets. Cancellation clears
  only the named pointer; keyboard cancellation leaves pointer holds intact.
  Notes, Colors, the chooser, Split and recovery use this helper. Their document,
  selection and dialog state remains each app's deliberate shared policy.
- [Surfaces](world/surface/surface.rs) conserves allowances, assigns disjoint rank
  ranges (explicit owner placement or free-space allocation), and clips/translates
  retained drawing. Changing one child's allowance
  does not move another child's live rank range. Fragmentation can refuse an
  allocation; live ranges are never silently compacted.
- [Layout](world/layout.rs) accepts owner-computed rectangles and an explicitly
  delegated controller. It validates the whole arrangement before applying it;
  there is no strip width, margin, launcher path or configuration schema here.
- [Storage scopes](world/storage/storage.rs), [management](world/management.rs)
  and [launch observations](world/launches.rs) use independently selected child
  capacities. They work without display state. `Scopes::prepare` is the shared
  connection/revision check and scope-stamping path for stock and Split storage.
  Successful containment proofs are bounded and tied to the binding/provider
  lifetime; see [the forwarding and invalidation rules](world/storage/README.md).
- [Funding](funding.rs) computes a local shortfall and validates growth against
  caller-selected ceilings. It contains no desktop limits or automatic grants.
  The `FUND` protocol reaches the supplying matrix over its interaction endpoint;
  the kernel's `RESOURCE_GROW` performs the conserving transfer.

This is a library, not a compulsory initial process or a desktop server. Another
distribution can compose these mechanisms with its own processes and protocols,
or implement the public service interfaces itself. The existing
[stock matrix executable](../../../distribution/matrix/main.rs) remains the
F5 consumer and owns its desktop slots, configuration, control router and
recovery policy. Shared protocols stay in [platform contracts](../../contracts/matrix.rs).

Host tests cover a three-child vertical arrangement with the last child as the
authorized controller, stable surface ranges through sibling allowance changes,
and four-child headless scope/management ledgers. Those are mechanism proofs,
not claims of a runnable phone or embedded distribution. The fixed surface and
input limits in the experimental protocol remain explicit resource bounds.
Host input proofs additionally cover independent selection/typing, scoped
cancellation and overflow, child-local names, fair drain, recursive forwarding
and fresh presses after cancellation. Group proofs cover a keyboard and two
pointers sharing selection without sharing capture, nested click-before-key
ordering, keyboard-only cancellation and refusal to silently change associations.
[Registration](world/input/registration.rs) now allocates local channels/groups
through the authenticated owner control connection. Ordinary child interaction
connections cannot register, select or inject by naming them. The internal
[name](world/input/names.rs) includes local versus inherited origin; identical
compact names on those boundaries do not share capture, held state or selection.
`RETIRED` discards one channel's pending input and ends its holds. Its slot remains
reserved until every receiving child consumes that boundary or loses its binding.
`GROUP_RETIRED` separately ends a group after its channels have drained; merely
replacing the last channel preserves remembered selection. Fresh identities
increase monotonically within a view binding; compact-name exhaustion refuses.
App reception frees retired pointer entries. Tests churn channels and groups
beyond the live-table capacity without rebinding geometry.

Stock and Split routers call `input_ready` before polling upstream. While a
retirement drains, local requests continue and another upstream input snapshot
waits. This bounds intermediate tables but introduces head-of-line backpressure
on that shared upstream input path; it is not a latency/isolation guarantee for
arbitrarily slow recipients. A consumer must keep draining lifecycle events.

The [producer entrance](world/input/producer.rs) now reserves an explicit channel
quota and one private group for an authenticated child connection. Requests use
grant-local channel ordinals and a per-child generation; neither identifies an
owner channel or lets one producer act for another. Click-to-focus is a separate
right. Disconnect, view invalidation, release or owner revocation retires its
channels and eventually its group. Regrant cannot bypass pending retirement.
Only this explicit grant reveals the input extent authorized to that producer.

The stock root reserves its fixed source channels at BIND before dynamic grants.
Nested stock hosts now negotiate an incoming channel/group ceiling before admitting
local producers. The [capacity calculation](world/input/capacity.rs) counts live
and retiring local allocations; the [stock adapter](../../../distribution/matrix/service/parent/input.rs)
obtains the parent's acknowledgement before committing the local grant, and widens
the ceiling again after retirement. Other compositions must perform that exchange
themselves; the library's reservation arguments do not negotiate it automatically.

`INPUT_CAPACITY` applies to the requesting connection and requires its current
view epoch. It survives view changes on that connection. Reducing below retained
channel/group aliases returns BUSY with the old limits intact. An empty event
queue does not make its channel identity free. New deliveries exceeding the
ceiling refuse before changing focus, capture or held state. Explicit producers
receive the refusal; already-consumed device/upstream samples instead interrupt
that stream and suppress residual holds without killing the adapter or parent
grant. Retirement and cancellation still reach existing recipients.

The stock broker now supplies [explicit producer acquisition](../../contracts/services/README.md)
on a separate pair, with manifest/owner checks and disconnect retirement. No
default app receives production; disposable desktop variants exercise acquisition.
This remains a transport foundation: independent group-controller delegation,
dynamic reassociation, descriptor discovery and multiple desktop cursors remain
pending. Do not lend an owner control endpoint to an app as a shortcut. The [F5 source adapter](../../../distribution/matrix/service/requests/source.rs)
supplies separate pointer and keyboard channels in one group;
no child receives injection authority from an ID in a request.
The independent program additionally exercises real IPC, malformed requests,
revocation and peer replacement in QEMU. It has no stock distribution dependency;
it now owns child admission through the shared runtime's
[admission coordinator](../user-runtime/matrix/admission/README.md). Nested
adaptation and rendering remain outside that proof's scope.

The installed [Split program](../../../distribution/applications/split/README.md)
is a second visible consumer. Its two-column policy uses the shared
[parent snapshot handler](world/upstream.rs), runtime admission, service offers
and [retained image custody](../user-runtime/matrix/upstream/README.md). The
ordinary desktop test separately checks the stock transaction/persistence path.
