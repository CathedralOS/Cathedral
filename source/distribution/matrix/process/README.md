# Stock matrix process

Start at [main.omg](main.omg): the stock matrix as a standalone Omega process on
the [process runtime](../../../platform/libraries/user-runtime/README.md) and the
[World](../../../platform/libraries/matrix/world/README.md). It is replacing the
[Rust host](../../../../source-rs/distribution/matrix/CHARTER.md) one responsibility
at a time, linking the [stock policy](../service/README.md) and the
[import contracts](../../../platform/contracts/imports/README.md) directly;
F5 keeps the Rust host until this program reaches parity, then the desktop
profile swaps and the [bridge](../bridge/policy/main.omg) goes. The
[native consumer](../native/main.omg) links it the same way as the installed apps
and declares a 128-page stack in [native/link.json](../native/link.json).

Today it is the display and storage root with children it funds itself: the
desktop supervisor admits it exactly as it admits the Rust host, and it serves
the owner, the display, physical input, the provider links and up to five
children with their interaction and storage connections.

- [startup.omg](startup.omg): read the environment. Four endpoints make the
  display root; owner control opens first, because the supervisor connects it
  right after the spawn, then the display connection, then the source
  listener. Other environment shapes are refused until their slices arrive.
- [commands.omg](commands.omg): owner words over the kernel-installed control
  pair. HEALTH, BIND (the display extent and keys become the world's grant, with
  the compositor's whole tile allowance and the source streams bound), the
  LAUNCH_IMAGES, LAUNCH_POLICY and LAUNCH_PLACE policy for a slot, the children
  and storage words below, and the World's placement words. Everything else is
  denied, never silently accepted.
- [children.omg](children.omg): RUN_PAGES and RUN_MEMORY stage a slot's next
  allowances; RUN reserves a launch for an image from the owner's catalogue
  under them (frames default to the load plus the private pages, or one
  admission allowance per descendant slot), sets the CPU ceiling and the images
  the child may run in turn, admits it through the runtime's brokered
  admission offering interaction and storage, then connects to the child's
  owner control, which the child accepts as its first act before it asks the
  broker for anything. STOP reclaims the subtree and returns the reservation;
  so does a listener that retires, and the exit goes to the console through
  [report.omg](report.omg). Every failed step leaves no reservation behind.
- [slots.omg](slots.omg): one admitted child's listener per turn. Service
  acquisition goes to the runtime's offer, surface and batch words to
  presentation (a sealed attachment is accepted only behind an image word),
  every other request to the World under the connection's incarnation.
- [presentation.omg](presentation.omg): a child's surfaces on the root display.
  Creation names the compositor tile behind the owner-allotted rank; paint,
  erase, cursor and sealed image updates are clipped and translated by the
  World, sent as that tile's update and committed only after the compositor
  accepts them. Inside a child's begin/commit batch they are staged and
  published together under one compositor transaction; the compact rectangles
  word ([rects.omg](rects.omg)) is such a batch of one or two paints. Images
  stay retained behind their tile until it is painted over or erased; owed
  erasures flush between turns.
- [storage.omg](storage.omg): the storage root. The provider links the
  composition supplied connect when the first grant or request needs them;
  STORAGE_GRANT binds a child's storage connection to a source and scope,
  STORAGE_REVOKE and STORAGE_BINDING forget and report it, STORAGE_STATUS names
  failed sources. Each child's storage request is relayed one hop under the
  World's scopes: authenticated, stamped, proven when both scope words are
  taken, forwarded with its sealed attachment and answered with the provider's
  reply and attachment. The matrix's own reads, the whole of a program object
  behind a launch, take the same path over the first source. A lost or late
  link retires that source's bindings until a grant reconnects it.
- [stored.omg](stored.omg): children from objects in the realm. RUN_OBJECT
  reads the program over the matrix's own binding, resolves its declaration
  through [declaration.omg](declaration.omg) (the platform's bounded image
  scan, the decoded record and the stock import selection against the
  services LAUNCH_IMPORTS offers the slot), fits it to the staged allowances
  with the stock resource policy, reserves it from the sealed image, admits it
  as RUN does and binds its storage to the realm the owner named; a binding
  that cannot be made takes the child back down.
- [source.omg](source.omg): the installed source endpoint. POINTER_SOURCE moves
  the compositor's physical cursor, then routes through the World on the pointer
  stream; KEY_SOURCE routes on the keyboard stream, and RESET cancels both. The
  driver checks that owner words are refused here before it trusts the link.
- [scene.omg](scene.omg): the shared client to the compositor, speaking its
  six-word frames through the runtime's sized requests, with a sealed image
  behind an image update.
- [report.omg](report.omg): the startup line the boot harness keys on, and a
  retired child's slot and exit in hex.

Prove it with `python tools/boot-harness-rs/omega_desktop_test.py`: the stock
composition with this program in the root matrix's launch, a realm holding
only the Omega Colors, and the supervisor built with `omega-root`, which admits
the root as F5 does, starts the storage service, runs Colors from that realm
object into an app slot over a sub-rectangle of a shifted backdrop with the
object as its realm, and then parks. The proof reads the supervisor's and the services' startup lines, sees
Colors paint its view, a healthy indicator and its checker image through this
matrix, moves the pointer over QMP and watches the compositor's cursor follow
it, then clicks Colors for focus and presses Enter, which saves a record
through the storage relay and repaints the view in the saved colour.

Not yet here: a child's own LAUNCH word with the stock admission decision and
launch receipts, the matrix's realm reads behind RESOLVE_PATH and startup,
loss surveys and close, management and launch observation words, nested
operation under a parent matrix with an upstream storage binding, and the
rest of the stock policy under [service/](../service/README.md), which still
executes inside the Rust host through the bridge; this process links that
package directly for what it already uses.
