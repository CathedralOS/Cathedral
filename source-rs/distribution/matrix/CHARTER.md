# Cathedral's stock matrix program

This executable belongs to the replaceable distribution. Its bounded child control
router, persisted `Matrix` desktop arrangement, launcher/app roles and recovery
policy are not requirements on other matrix programs. The
[desktop extension contract](contracts/matrix.rs) owns those wire operations
and the saved configuration format. Existing packet values and saved bytes are
unchanged by this ownership split.

The [platform matrix library](../../platform/libraries/matrix/README.md) owns
reusable attenuation, local input, surface allowances, storage scopes and task
management ledgers. It takes a child capacity and owner-supplied geometry; it
does not load desktop configuration or choose launchers. This executable uses
that library with five children: chrome, three app instances and background. Its IPC service loop and forwarding adapters
remain here with its current composition; there is not yet a separately packaged
general-purpose matrix server runtime.

A matrix receives an environment grant and provides narrower environments to
its children. The same [world](../../platform/libraries/matrix/world/world.rs) manages local children and input at
every level. No world policy branches on depth or on a global task identity.
The built-in distribution supplies layout and supervision policy.

Start at [main.rs](main.rs), which selects startup intent, then
[service entrance](service/service.rs): consume parent invalidations, wake affected children, observe connection lifetime,
dispatch an owner or child request, reply. [parent entrance](service/parent/parent.rs) adapts
either a root grant or the upstream matrix endpoint into the same `Grant`.
[requests entrance](service/requests/requests.rs) authenticates owner/source/child requests;
[owner.rs](service/requests/owner.rs) handles local layout, while
[children entrance](service/children/children.rs) owns admission, pending control replies and teardown.
[lifecycle.rs](service/children/lifecycle.rs) collects exact task outcomes;
[recovery.rs](service/children/recovery/recovery.rs) reloads a failed configured launcher;
[fallback.rs](service/children/recovery/fallback.rs) admits explicitly supplied recovery code.
[world entrance](../../platform/libraries/matrix/world/world.rs) owns grant changes and child dispatch; descend into
[children.rs](../../platform/libraries/matrix/world/children.rs) for views/epochs/queues and
[input.rs](../../platform/libraries/matrix/world/input.rs) for focus, capture and key attenuation.

## Child boundary

The [shared protocol](../../platform/contracts/matrix.rs) admits `VIEW`, `NEXT` and
clipped `PAINT`/`ERASE` surface commands on child connections. Authentication comes from the accepted IPC endpoint.
Children supply no task, world, ancestor or sibling selector. Their replies
contain only their own extent, permitted keys, local binding revision and input.
They receive no position in the parent, ancestor extent, tree depth, sibling
count, global cursor position or owner API. Unknown/owner operations and extra
fields receive the same empty `DENIED` response.

Each child has its own revision counter. Moving its parent or changing a sibling
does not advance it. Capture keeps delivering but clamps coordinates to the
child's viewport; cancellation has zero coordinates. Keys intersect the upstream
mask. Changing focus cancels undelivered old input and suppresses held-key
transfer. Overflow cancels the affected child without notifying its siblings.

Admitted children receive an empty coalesced wake hint over a separate endpoint.
The service consumes pending parent hints before ordinary child dispatch and refreshes
local authority/input through the parent protocol. During publication it can answer
other children's queries from the existing local snapshot; parent refresh and
dependent mutations resume after publication completes. Older two-endpoint fixtures
refresh on every request. Hints disclose no state; full hint queues already contain
a wake. Parent transport closure retires the service. Parent loss or a new
parent binding clears descendant placement; reappearance requires explicit
owner regranting. Old revisions cannot read queued events in a new binding.
Revisions are scoped to the accepted connection, not transferable capabilities.
Clients must discard them on provider replacement. A timed-out upstream RPC
retires the service so a late reply cannot be mistaken for another operation.
Owner requests routed to a child use a FIFO control stream the child answers in
order: an unanswered request is counted and its late reply skipped, so a slow
child costs the owner a timeout rather than its life. At the unanswered-request
limit, probes stop until late replies drain. A broken control direction alone
does not prove task death or authorize discarding work; the owner can explicitly stop it.

## Delegated task management

The owner may grant a live child management of the other local slot using
`MANAGEMENT_GRANT`. The [management ledger](../../platform/libraries/matrix/world/management.rs) binds this to
the authenticated connection incarnation. `MANAGED_TASK` reveals only whether
that permitted slot holds a task, issuing an endpoint-local ticket for that exact
task. `TERMINATE_TASK` checks the ticket against the current private kernel child
handle before [collecting it](service/children/management.rs). It bypasses LOSS;
the manager already holds authority to discard the task's work. It cannot grant
itself authority, select another slot, identify a PID, or use an old confirmation
against a replacement. Revocation/regrant invalidates outstanding tickets.
Disconnect removes the grant and its local counter; target teardown invalidates
observations. Owner REVOKE/BIND also clear management grants.

Stock CONFIGURE grants its chosen launcher management of the app slot alongside
layout authority. Local launcher recovery explicitly grants the new connection
fresh authority, including over an app it did not launch. An explicit owner
management change cancels that automatic recovery policy. No configuration-file
schema or kernel ABI changes. The distribution's
[confirmation controls](../applications/chooser/session/management/management.rs)
provide the loss warning and default Cancel; the platform does not claim to
authenticate a human click or provide a protected global escape surface.
Active storage/publication exchanges and loss surveys must finish before a stop
can run; BUSY requires another explicit request. Ordinary CLOSE still requires
a clean LOSS reply and a launch receipt alone conveys no management authority.

## Configured launcher recovery

Successful stock `CONFIGURE` arms one retry for its launcher after a fault or
nonzero return. A clean return and explicit owner STOP do not restart it.
The matrix collects the dead task, releases its surfaces and funding, then reads
the executable again through its asynchronous storage path. Admission gives
the replacement fresh endpoints, storage binding, layout and management authority. No old
launch-observation receipt transfers to it. Code reads use the existing object
identity within the same realm incarnation; this is not a persistent global reference.

Recovery snapshots the launcher's current arrangement, rather than rerunning
configuration. The sibling app retains its task, surfaces, geometry epoch, focus
and draft. Changed ancestor display/storage authority, explicit owner changes
to the launcher policy, or a separately admitted slot occupant cancel recovery.
Old deferred events drain before replacement. The retry is consumed even when
reading/admission fails. If the owner supplied a `LAUNCHER_FALLBACK` before
CONFIGURE, exhaustion admits that executable into the retired launcher's view
and budget. Otherwise the app remains alive without its strip. The fallback
gets local interaction, a control link and a single-use `RETRY_LAUNCHER` request
bound to its exact task/connection. It receives no storage, layout, management,
additional images or descendant capacity. Boot-template admission includes its
own code grant, which cannot reserve a process with zero available child slots.
The selected fallback decides when to send the request; the provider authenticates
its endpoint, not a physical human gesture.

The fallback request is acknowledged before retirement; the next asynchronous
read uses the captured realm identity and placement. Each explicit request buys
one attempt, never another automatic retry. A new failure can show a fresh
fallback, while fallback death/clean exit and owner STOP/revocation never act as
input or request another attempt. Ancestor authority changes cancel the saved
context. Recovery remains dependent on the live matrix, input and storage paths;
this is not the platform's still-unspecified protected global escape surface.

`LAUNCH_IMAGES` explicitly allows the owner's existing executable grants to
accompany future store-based admissions in one slot. It defaults to none. F5
uses it to give the user matrix the boot recovery program, then selects that
program as its fallback. Neither ordinary apps nor the fallback inherit the
user matrix's executable catalogue. The platform chooses no distribution binary
or UI text. The copied-image
[launcher recovery fixture](../../../tools/boot-harness-rs/launcher_recovery_test.py)
exercises real faults, explicit retries, fallback failure/exit, clean launcher
exit, owner STOP and storage revocation, checking draft retention, live layout,
input, saving and cold persistence.

## Limits of this milestone

[Storage delegation](../../platform/libraries/matrix/world/storage/README.md) passes whole private object roots
through the same matrix chain, independently of display grants, with sealed
attachments forwarded in both directions. Owner assignment
is exclusive between siblings; every request checks a local binding revision.
The visible desktop admits children with separate interaction and catalog pairs,
plus a wake receiver. [Shared admission](../../platform/libraries/user-runtime/matrix/admission/admission.rs)
owns channel construction, endpoint transfer and rollback; the [runtime bundle](../../platform/libraries/user-runtime/matrix/matrix.rs)
names the child-visible roles. Child storage forwarding resumes from the service
loop without occupying the upstream interaction pair. Root provider failures
retire the affected storage binding. A separate parent catalog transport failure
retires only catalog authority; legacy shared parent failures retire the matrix. Revocation/death aborts staged work; replacement requires fresh grants. Both the
headless storage fixture and visible desktop exercise this boundary.

Presentation follows [the bounded batch policy](../../platform/libraries/matrix/world/presentation/presentation.rs),
[authenticated requests](service/requests/presentation.rs), then
[upstream publication](service/parent/presentation.rs). `PRESENT_BEGIN` stages up
to forty distinct existing surfaces (including sealed images); `PRESENT_COMMIT`
publishes the complete clipped batch recursively. A failed stage poisons the
batch until abort. Abort, binding loss and death drop staged readers. The latest
successful revision can be retried within its connection and view epoch.
`PRESENT_RECTS` carries one or two existing rectangle updates in a single atomic
packet at every matrix hop, using the same revision history. Publication resumes
from the service loop after upstream replies; dependent events occupy one bounded
slot per stream, while unrelated local reads can progress. The publisher keeps
geometry stable until completion. See [publication scheduling](../../platform/libraries/matrix/world/surface/README.md#publication-scheduling).
Surface allocation remains outside transactions. Owner placement/replay still
uses individual updates. Completion means logical scene publication, not vblank
or tear-free scanout. Transport timeouts retire the matrix; retries do not cross
provider death. The desktop fixture checks old pixels before commit, complete
publication, rejected batches, repeated commits and bitmap forwarding.

This is a protocol and input experiment: two local child slots, five navigation
keys, eight queued events per child and extents up to 4096. The
[distribution fixture](../desktop/README.md) runs two
identical service executables with separate state. Each owns and admits its own
children using inherited launch loans and dynamic parent endpoints. Distribution
policy uses private owner commands; children cannot invoke them. Input wakes idle clients through their granted hint channel. Upstream state
refresh, owner storage grant/revoke and nontransactional surface operations still use synchronous calls; quiet servers retain a bounded
maintenance wake to discover legacy links. This is not a latency guarantee.

The visible matrix desktop now uses this provider for lifecycle, primitive surface
clipping, pointer routing and focus. Root adapters connect the existing compositor
and physical decoder. Rendering retains independently named rectangles and sealed
bitmaps in bounded stacking ranges. Apps persist private color state through delegated
catalogs. Rich scenes, audio and trusted-path UI remain unintegrated. Local primitives are retained and
replayed through changed placement without requiring an app repaint; revoke and
connection replacement clear retained state. The earlier
visual session remains a separate, richer regression fixture.

The locality tests compare protocol observations. They do not establish complete
system noninterference: IPC, task, clock, link and region handles now resolve locally, and region
mapping addresses use local slots. An explicitly granted
global clock and shared execution resources remain. IPC storage now has backed
launch-account reservations that survive creator replacement and support conserving
owner/idle-child subdivision. Active funding transfer remains absent. CPU turns follow the launch
ownership tree, so inner matrices subdivide their outer matrix's turns. Measured
CPU burst/refill ceilings now gate every ancestor account and survive replacement.
These are owner-assigned caps with periodic-timer overrun, not priority grants,
capacity reservations or deadline guarantees. The remaining timing
interfaces require a confinement audit; this lab must not be represented as a
production guarantee that a process cannot infer anything about its surroundings.

Text-capable compositions grant letter/digit/Space/Backspace and function-key bits
through the same intersection/focus rules; the default navigation grant is unchanged.
`POLL` reserves bits 56 and above for surface allowance, outside the physical key mask.
`RUN_PAGES` accepts the existing kernel page bound; actual admission still requires
conserved owner funding. The root-only owner `STORAGE_STATUS` reports retired
provider streams without reconnecting them. It is unavailable to app connections
and nested adapters; distribution supervision interprets it.

## Desktop roles and instances

[Configuration](service/children/configuration.rs) resolves the chrome from the
saved `Matrix.launcher` path and the background from the realm-local `Background`
path. Both are installed executables, replaceable independently of the platform.
The background gets one bottom surface, no keys, storage, channel-creation budget
or descendants. It can see pointer events in uncovered background space. Chrome
gets its own top surfaces, layout authority and local app management. The installer
preserves customized executable bytes; the current UI does not offer role selection.

[Workspace](service/children/workspace.rs) authenticates the current chrome endpoint
before listing or selecting an instance. Tickets are local, monotonic and retired
on exit. Ordinary apps cannot enumerate siblings. [Launch](service/children/launch.rs)
chooses a free app slot and reads code through the requester's scoped realm.
[Lifecycle](service/children/lifecycle.rs) asks LOSS before chrome-requested close,
reclaims the exact process/subtree and selects a surviving app.

Selection hides sibling surfaces and input without changing their local geometry,
process state or grants. Hidden updates remain clipped to zero upstream; selecting
an instance replays its retained scene. This is distribution tab-switching policy
over the platform's owner-controlled visibility mechanism. It is not process
suspension, per-instance storage cloning, or a universal window-manager protocol.
Multiple instances of one program share its persistent realm; their private drafts
are independent and existing revision checks govern conflicting saves.

Three apps fit the current desktop policy, subject to subtree funding. Split consumes
more task/channel capacity than a leaf and may be refused while other subtrees run.
The owner control route now uses three bits per hop (five children); recompile
stock control clients together. The legacy unconfigured two-child lab policy remains.


The installed Nested variant uses [startup](service/children/startup/startup.rs)
to prepare a scoped user template and run the same chooser over its `Users`.
Selecting a user retires the picker and expands the user's surface allowance
before servicing its first publication. That user requests the usual CONFIGURE
flow through `WORKSPACE_ARGUMENT`; the argument grants no authority. The outer
boot owner still invokes CONFIGURE explicitly for its user. Provider/root adapter
selection depends exclusively on supplied endpoints, at every depth.

Desktop app declarations may request explicit funding via `CTHIMP04`; insufficient
offers refuse before spawn. A failed reservation releases provisional geometry.
App selection uses [scene publication](service/parent/scene.rs) to stage the old
and new app surfaces together. A rejected stage/commit is aborted and local
selection restored; unrelated hidden apps need no replay. This does not make
startup, layout editing or process teardown one atomic scene transaction.
