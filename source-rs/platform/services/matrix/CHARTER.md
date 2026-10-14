# Experimental matrix provider

A matrix receives an environment grant and provides narrower environments to
its children. The same [world](world/world.rs) manages local children and input at
every level. No world policy branches on depth or on a global task identity.
The built-in distribution supplies layout and supervision policy.

Start at [main.rs](main.rs), which selects the supplied parent adapter, then
[service entrance](service/service.rs): consume parent invalidations, wake affected children, observe connection lifetime,
dispatch an owner or child request, reply. [parent entrance](service/parent/parent.rs) adapts
either a root grant or the upstream matrix endpoint into the same `Grant`.
[requests entrance](service/requests/requests.rs) authenticates owner/source/child requests;
[owner.rs](service/requests/owner.rs) handles local layout, while
[children entrance](service/children/children.rs) owns admission, pending control replies and teardown.
[world entrance](world/world.rs) owns grant changes and child dispatch; descend into
[children.rs](world/children.rs) for views/epochs/queues and
[input.rs](world/input.rs) for focus, capture and key attenuation.

## Child boundary

The [shared protocol](../../../contracts/matrix.rs) admits `VIEW`, `NEXT` and
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

## Limits of this milestone

[Storage delegation](world/storage/README.md) passes whole private catalog roots
through the same matrix chain, independently of display grants. Owner assignment
is exclusive between siblings; every request checks a local binding revision.
The visible desktop admits children with separate interaction and catalog pairs,
plus a wake receiver. [Connection construction](service/children/connections.rs)
owns allocation and rollback; the [runtime bundle](../../libraries/user-runtime/matrix.rs)
names the child-visible roles. Child storage forwarding resumes from the service
loop without occupying the upstream interaction pair. Root provider failures
retire the affected storage binding. A separate parent catalog transport failure
retires only catalog authority; legacy shared parent failures retire the matrix. Revocation/death aborts staged work; replacement requires fresh grants. Both the
headless storage fixture and visible desktop exercise this boundary.

Presentation follows [the bounded batch policy](world/presentation/presentation.rs),
[authenticated requests](service/requests/presentation.rs), then
[upstream publication](service/parent/presentation.rs). `PRESENT_BEGIN` stages up
to four distinct existing surfaces (including sealed images); `PRESENT_COMMIT`
publishes the complete clipped batch recursively. A failed stage poisons the
batch until abort. Abort, binding loss and death drop staged readers. The latest
successful revision can be retried within its connection and view epoch.
`PRESENT_RECTS` carries one or two existing rectangle updates in a single atomic
packet at every matrix hop, using the same revision history. Publication resumes
from the service loop after upstream replies; dependent events occupy one bounded
slot per stream, while unrelated local reads can progress. The publisher keeps
geometry stable until completion. See [publication scheduling](world/surface/README.md#publication-scheduling).
Surface allocation remains outside transactions. Owner placement/replay still
uses individual updates. Completion means logical scene publication, not vblank
or tear-free scanout. Transport timeouts retire the matrix; retries do not cross
provider death. The desktop fixture checks old pixels before commit, complete
publication, rejected batches, repeated commits and bitmap forwarding.

This is a protocol and input experiment: two local child slots, five navigation
keys, eight queued events per child and extents up to 4096. The
[distribution fixture](../../../distribution/lab/matrix/README.md) runs two
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
