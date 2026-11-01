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

Today it is the display and storage root with children it funds itself, and
it runs nested under a parent matrix as well: the desktop supervisor admits
the root exactly as it admits the Rust host, the root admits a nested instance
from a realm object as it admits any program, and each serves the owner, the
display or the parent, physical input or the parent's snapshots, the provider
links or the upstream storage connection, and up to five children with their
interaction and storage connections.

- [startup.omg](startup.omg): read the environment. Four endpoints make the
  display root; owner control opens first, because the supervisor connects it
  right after the spawn, then the display connection, then the source
  listener. Two endpoints are a parent matrix's broker pair; owner control
  opens first here too, then the parent's interaction, its storage pair and
  the first snapshot of the view it grants. Other shapes are refused.
- [upstream.omg](upstream.omg): the parent matrix. Its interaction connection
  is polled for this matrix's view and the one input event each snapshot
  carries, which the World adopts and routes on under the parent's names: when
  the parent rings this matrix's bell, when the last snapshot had more, and
  before an owner word that needs a view none has granted yet. The same
  connection carries every surface, image and batch word upward.
- [commands.omg](commands.omg): owner words over the kernel-installed control
  pair. HEALTH, BIND at the root (the display extent and keys become the
  world's grant, with the compositor's whole tile allowance and the source
  streams bound; under a parent the view is the parent's to grant), the
  LAUNCH_IMAGES, LAUNCH_POLICY and LAUNCH_PLACE policy for a slot, the children,
  storage and stored words below, words routed to a child, and the World's
  placement words. Everything else is denied, never silently accepted.
- [forward.omg](forward.omg): owner words routed to a child. The stock
  control policy decodes the first hop above the operation into the slot and
  sets the deadline the word's work needs; the word goes on to that child's
  owner control with the hop removed and the child's reply answers the owner.
  One routed word is pending per child; a late reply is drained unanswered
  after the owner heard TIMED_OUT.
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
- [presentation.omg](presentation.omg): a child's surfaces on the display, or
  on the parent's view. Creation names the backing behind the owner-allotted
  rank: the compositor tile at the root, a surface of this matrix's own view
  created upstream under a parent. Paint, erase, cursor and sealed image
  updates are clipped and translated by the World, sent as that backing's
  update and committed only after the display or the parent accepts them.
  Inside a child's begin/commit batch they are staged and published together
  under one transaction; the compact rectangles word ([rects.omg](rects.omg))
  is such a batch of one or two paints. Images stay retained behind their
  backing until it is painted over or erased; owed erasures flush between turns.
- [storage.omg](storage.omg): this matrix's storage. At the root the provider
  links the composition supplied connect when the first grant or request
  needs them; under a parent the broker's storage connection is the one
  source, over which the parent reports this matrix's binding revision and
  realm. STORAGE_GRANT binds a child's storage connection to a source and
  scope, STORAGE_REVOKE and STORAGE_BINDING forget and report it,
  STORAGE_STATUS names failed sources at the root. Each child's storage
  request is relayed one hop under the World's scopes: authenticated, stamped,
  proven when both scope words are taken, forwarded with its sealed attachment
  and, on the way up, this matrix's own revision, and answered with the reply
  and attachment. The matrix's own reads, the whole of a program object behind
  a launch, take the same path over the first source. A lost or late link
  retires that source's bindings until a grant reconnects it; a parent that
  refuses this matrix's revision is asked for the binding again once.
- [stored.omg](stored.omg): children from objects in the realm. RUN_OBJECT
  reads the program over the matrix's own binding, resolves its declaration
  through [declaration.omg](declaration.omg) (the platform's bounded image
  scan, the decoded record and the stock import selection against the
  services LAUNCH_IMPORTS offers the slot), fits it to the staged allowances
  with the stock resource policy, reserves it from the sealed image, admits it
  as RUN does and binds its storage to the realm the owner named; a binding
  that cannot be made takes the child back down. A child's own LAUNCH asks
  for a sibling: the stock admission decision chooses the slot the owner
  opened with LAUNCH_POLICY, the program is read under the requester's scope
  so a launcher launches only what it can see, the child takes the policy's
  allowances, its object as its realm and the owner's LAUNCH_PLACE placement
  with the ranks the policy allotted, and the requester receives a receipt it
  can query with LAUNCH_STATUS until the child ends.
- [source.omg](source.omg): the installed source endpoint. POINTER_SOURCE moves
  the compositor's physical cursor, then routes through the World on the pointer
  stream; KEY_SOURCE routes on the keyboard stream, and RESET cancels both. The
  driver checks that owner words are refused here before it trusts the link.
- [scene.omg](scene.omg): the shared client to the compositor, speaking its
  six-word frames through the runtime's sized requests, with a sealed image
  behind an image update.
- [report.omg](report.omg): the startup lines the boot harness keys on, and a
  retired child's slot and exit in hex.

Prove it with `python tools/boot-harness-rs/omega_desktop_test.py`: the stock
composition with this program in the root matrix's launch and in the place of
the user template, object three of the stock realm as F5 installs it, and the
supervisor built with `omega-root`, which admits the root as F5 does, starts
the storage service, runs the template nested over its own realm at an inset,
configures it through routed owner words to run the template's launcher as a
strip and to open an app slot to the launcher's launches with a standing
policy and a placement, and then parks. The proof reads the supervisor's, the
services' and the nested matrix's startup lines, sees the launcher paint its
tiles inside the nested area, clicks the Colors tile so the launcher launches
Colors through the nested matrix, sees Colors paint its view, a healthy
indicator and its checker image through both matrices, moves the pointer over
QMP and watches the compositor's cursor follow it, then clicks Colors for
focus and presses Enter, which saves a record through both storage relays and
repaints the view in the saved colour.

Not yet here: the desktop mode with its chrome, app entries, selection and
layout words, the matrix's realm reads behind RESOLVE_PATH and the startup
discovery that CONFIGURE asks for, loss surveys and close, management words,
the funding and input capacity a nested matrix negotiates with its parent,
and the rest of the stock policy under [service/](../service/README.md), which
still executes inside the Rust host through the bridge; this process links
that package directly for what it already uses.
