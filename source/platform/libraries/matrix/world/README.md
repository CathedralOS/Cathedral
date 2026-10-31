# World

Start at [main.omg](main.omg): `request` answers an authenticated child,
`owner_command` and `source_command` serve the program that owns this world, and
[upstream.omg](upstream.omg) adopts a parent's poll. A world receives a grant
from above, places up to four children below it, and routes input between them
so that no child sees beyond its grant. The same record serves a root adapter and
a nested matrix. Identity comes from the connection a request arrived on, never
from its words. Wire owner: `source-rs/platform/libraries/matrix/world/`.

- [level.omg](level.omg) holds the record and its structural rules: inherit a
  grant, place and revoke, forget a departed occupant, hit-test the pointer,
  interrupt a stream everywhere it was delivered, and move a group's keyboard
  selection. A new parent epoch invalidates every placement and input binding;
  a lost grant cannot be repaired by a later regrant alone.
- [geometry.omg](geometry.omg) defines grants and placements and clips a
  placement to the parent's extent and keys.
- [children.omg](children.omg) owns each child's view: an epoch that advances
  only when its own extent or keys change, child-local channel and group names,
  per-stream queues drained in order within a group and fairly across groups,
  and the POLL packet.
- [events.omg](events.omg) is one stream's bounded FIFO inside one child: sixteen
  events plus a boundary slot, trailing-motion coalescing, overflow to CANCEL.
- [input/main.omg](input/main.omg) admits stream names into groups and keeps
  each stream's capture, buttons and held keys. [input/pointer.omg](input/pointer.omg)
  hit-tests, selects on press, captures through a drag and translates
  coordinates. [input/keys.omg](input/keys.omg) delivers to the group focus
  under the child's key mask and suppresses keys held across a focus switch.
- [names.omg](names.omg) distinguishes locally admitted from inherited names.

The program chooses placement, slots and lifecycle; the world enforces
attenuation. It depends only on the reference protocol contracts and performs no
kernel calls. The [independent matrix](../../../../distribution/lab/matrix-composition/README.md)
is its first consumer.

Not yet ported from the Rust library: surfaces and frame demand, input
producers, the input directory and descriptions, cursor preference, capacity
limits, owner-registered streams, stream retirement, the layout controller and
management grants. An inherited DESCRIBED, RETIRED or GROUP_RETIRED event drops
the parent grant until the next poll regrants it. The stock matrix needs those
before it can move onto this world.
