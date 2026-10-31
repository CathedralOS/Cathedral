# Stock matrix process

Start at [main.omg](main.omg): the stock matrix as a standalone Omega process on
the [process runtime](../../../platform/libraries/user-runtime/README.md) and the
[World](../../../platform/libraries/matrix/world/README.md). It is replacing the
[Rust host](../../../../source-rs/distribution/matrix/CHARTER.md) one responsibility
at a time; F5 keeps the Rust host until this program reaches parity, then the
desktop profile swaps and the [bridge](../bridge/policy/main.omg) goes. The
[native consumer](../native/main.omg) links it the same way as the installed apps
and declares a 128-page stack in [native/link.json](../native/link.json).

Today it is the display root with children it funds itself: the desktop
supervisor admits it exactly as it admits the Rust host, and it serves the
owner, the display, physical input and up to five children.

- [startup.omg](startup.omg): read the environment. Four endpoints make the
  display root; owner control opens first, because the supervisor connects it
  right after the spawn, then the display connection, then the source
  listener. Other environment shapes are refused until their slices arrive.
- [commands.omg](commands.omg): owner words over the kernel-installed control
  pair. HEALTH, BIND (the display extent and keys become the world's grant, with
  the compositor's whole tile allowance and the source streams bound), the
  LAUNCH_IMAGES, LAUNCH_POLICY and LAUNCH_PLACE policy for a slot, the children
  words below, and the World's placement words. Everything else is denied,
  never silently accepted.
- [children.omg](children.omg): RUN_PAGES and RUN_MEMORY stage a slot's next
  allowances; RUN reserves a launch for an image from the owner's catalogue
  under them (frames default to the load plus the private pages, or one
  admission allowance per descendant slot), sets the CPU ceiling and the images
  the child may run in turn, admits it through the runtime's brokered
  admission with interaction, then connects to the child's owner control,
  which the child accepts as its first act before it asks the broker for
  anything. STOP reclaims the subtree and returns the reservation; so does a
  listener that retires, and the exit goes to the console through
  [report.omg](report.omg). Every failed step leaves no reservation behind.
- [slots.omg](slots.omg): one admitted child's listener per turn. Service
  acquisition goes to the runtime's offer, surface words to presentation, every
  other request to the World under the connection's incarnation.
- [presentation.omg](presentation.omg): a child's surfaces on the root display.
  Creation names the compositor tile behind the owner-allotted rank; paint,
  erase and cursor updates are clipped and translated by the World, sent as
  that tile's rectangle and committed only after the compositor accepts them;
  owed erasures flush between turns.
- [source.omg](source.omg): the installed source endpoint. POINTER_SOURCE moves
  the compositor's physical cursor, then routes through the World on the pointer
  stream; KEY_SOURCE routes on the keyboard stream, and RESET cancels both. The
  driver checks that owner words are refused here before it trusts the link.
- [scene.omg](scene.omg): the shared client to the compositor, speaking its
  six-word frames through the runtime's sized requests.
- [report.omg](report.omg): the startup line the boot harness keys on, and a
  retired child's slot and exit in hex.

Prove it with `python tools/boot-harness-rs/omega_desktop_test.py`: the stock
composition with this program in the root matrix's launch, the Omega background
added to the catalogue, and the supervisor built with `omega-root`, which admits
the root as F5 does, runs the background into its slot over a sub-rectangle
of a shifted backdrop, and then parks instead of starting storage. The proof
reads the supervisor's and the drivers' startup lines, sees the child's fill
inside its placement and the backdrop outside, then moves the pointer over
QMP and watches the compositor's cursor follow it through this matrix.

Not yet here: launches from the realm's objects and the stock launch policy,
storage binding and forwarding, images and the begin/commit presentation batch,
loss surveys and close, management and launch observation words, nested
operation under a parent matrix, and the stock policy packages under
[service/](../service/README.md), which still execute inside the Rust host
through the bridge.
