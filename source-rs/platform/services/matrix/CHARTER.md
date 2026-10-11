# Experimental matrix provider

A matrix receives an environment grant and provides narrower environments to
its children. The same [world](world.rs) manages local children and input at
every level. No world policy branches on depth or on a global task identity.
The built-in distribution supplies layout and supervision policy.

Start at [main.rs](main.rs), which selects the supplied parent adapter, then
[service.rs](service.rs): refresh parent authority, observe connection lifetime,
dispatch an owner or child request, reply. [parent.rs](service/parent.rs) adapts
either a root grant or the upstream matrix endpoint into the same `Grant`.
[requests.rs](service/requests.rs) authenticates owner/source/child requests;
[owner.rs](service/requests/owner.rs) handles local layout, while
[children.rs](service/children.rs) owns admission, pending control replies and teardown.
[world.rs](world.rs) owns grant changes and child dispatch; descend into
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

Every child request first refreshes upstream authority. Parent loss or a new
parent binding clears descendant placement; reappearance requires explicit
owner regranting. Old revisions cannot read queued events in a new binding.
Revisions are scoped to the accepted connection, not transferable capabilities.
Clients must discard them on provider replacement. A timed-out upstream RPC
retires the service so a late reply cannot be mistaken for another operation.

## Limits of this milestone

This is a protocol and input experiment: two local child slots, five navigation
keys, eight queued events per child and extents up to 4096. The
[distribution fixture](../../../distribution/lab/matrix/README.md) runs two
identical service executables with separate state. Each owns and admits its own
children using inherited launch loans and dynamic parent endpoints. Distribution
policy uses private owner commands; children cannot invoke them. Input is demand-driven and uses synchronous upstream
queries; the fixed acyclic fixture bounds these calls. This is not the eventual
low-latency delivery design.

The visible matrix desktop now uses this provider for lifecycle, primitive surface
clipping, pointer routing and focus. Root adapters connect the existing compositor
and physical decoder. Rendering is one retained rectangle per path, with two
matrix levels supported by the bounded key space. Rich scenes, storage namespaces,
audio, event-driven input and trusted-path UI remain unintegrated. The earlier
visual session remains a separate, richer regression fixture.

The locality tests compare protocol observations. They do not establish complete
system noninterference: IPC, task, clock, link and region handles now resolve locally, and region
mapping addresses use local slots. An explicitly granted
global clock, shared channel-pool contention and execution resources remain. Those
interfaces require a confinement audit; this lab must not be represented as a
production guarantee that a process cannot infer anything about its surroundings.
