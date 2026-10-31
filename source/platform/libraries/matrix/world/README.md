# World

Start at [main.omg](main.omg): `request` answers an authenticated child,
`owner_command` and `source_command` serve the program that owns this world, and
[upstream.omg](upstream.omg) adopts a parent's poll. A world receives a grant
from above, places up to four children below it, and routes input between them
so that no child sees beyond its grant. The same record serves a root adapter and
a nested matrix. Identity comes from the connection a request arrived on, never
from its words. Wire owner: `source-rs/platform/libraries/matrix/world/`.

- [level.omg](level.omg) holds the record and its structural rules: inherit a
  grant, place and revoke, forget a departed occupant, hit-test the pointer
  (higher surface ranks stack above lower ones), interrupt a stream everywhere
  it was delivered, and move a group's keyboard selection. A new parent epoch
  invalidates every placement and input binding; a lost grant cannot be
  repaired by a later regrant alone.
- [geometry.omg](geometry.omg) defines grants and placements and clips a
  placement to the parent's extent and keys.
- [children.omg](children.omg) owns each child's view: an epoch that advances
  only when its own extent or keys change, child-local channel and group names,
  per-stream queues drained in order within a group and fairly across groups,
  the POLL packet, the directory of admitted bindings at one revision per
  membership with the description behind each row, the child's own capacity
  ceiling, and its cursor preference per delivered pointer.
- [events.omg](events.omg) is one stream's bounded FIFO inside one child: sixteen
  events plus a boundary slot, trailing-motion coalescing, overflow to CANCEL.
  Each event packs into one word beside its order number; a described channel
  announces its description once, before its first event.
- [description.omg](description.omg) validates the bounded label words a source
  may attach to a channel: controls within the vocabulary and a non-empty,
  canonically padded, UTF-8 label without control characters.
- [surfaces.omg](surfaces.omg) owns surface names and allowances: an owner
  gives a child a range of ranks inside the world's allowance, a child creates
  named surfaces on those ranks, and its paint, image, cursor and erase updates
  are clipped to its view and translated for the parent. Retained updates
  replay under new geometry; retired surfaces owe the parent an erasure. A child
  keeps at most sixteen ranks so four children fit on a process stack.
- [demand.omg](demand.omg) composes render demand: the parent's demand, each
  child's visibility and one pending owner-requested frame decide whether a
  child's poll asks for frames, and the frame handshake runs ticket by ticket.
- [input/main.omg](input/main.omg) admits stream names into groups and keeps
  each stream's capture, buttons, held keys, description and last position.
  [input/pointer.omg](input/pointer.omg) hit-tests, selects on press, captures
  through a drag and translates coordinates. [input/keys.omg](input/keys.omg)
  delivers to the group focus under the child's key mask and suppresses keys
  held across a focus switch.
- [input/registration.omg](input/registration.omg) is the owner's side of local
  names: register groups and channels, inject on them, select, retire. A retired
  name stays charged until every recipient consumes its boundary, a released
  group follows its last channel, and no name is reused.
- [input/producer.omg](input/producer.omg) grants a connected child a quota of
  private channels in a private group, authenticated by connection and
  generation, with an optional focus right; the child names them by ordinal.
- [input/describe.omg](input/describe.omg) fixes a channel's description before
  any recipient binds the name. [input/cursor.omg](input/cursor.omg) resolves
  the cursor a stream currently presents, for an owner's renderer.
  [input/capacity.omg](input/capacity.omg) turns capacity pressure on a consumed
  sample into a stream interruption and reports the inherited headroom.
- [names.omg](names.omg) distinguishes locally admitted from inherited names.

The program chooses placement, slots and lifecycle; the world enforces
attenuation. It depends only on the reference protocol contracts and performs no
kernel calls. The [independent matrix](../../../../distribution/lab/matrix-composition/README.md)
is its first consumer.

Not yet ported from the Rust library: the layout controller and management
grants. The stock matrix needs those before it can move onto this world. The
record is sized for the reference kernel's eight-page process stack; a larger
stock matrix will need more stack or a sparser record.
