# Stock matrix process

Start at [program/main.omg](program/main.omg): the stock matrix as a standalone Omega process on
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
it runs nested under a parent matrix as well: the F5 desktop supervisor admits
the root exactly as it admits the Rust host and drives it with the same login
and user entry words, the root admits a user's matrix from a realm object as
it admits any program, and that matrix configures its workspace from the
realm's saved record itself. Each serves the owner, the display or the parent,
physical input or the parent's snapshots, the provider links or the upstream
storage connection, and up to five children with their interaction and storage
connections.

The process is five packages by responsibility, each depending only on the
ones listed after it: the program, the connections, the children, the realm
setup and the shared state. The program package is the one the native links.

### [program/](program/build.omg)

The process entry: the turn loop, startup and the owner's words.

- [startup.omg](program/startup.omg): read the environment. Four endpoints make the
  display root; owner control opens first, because the supervisor connects it
  right after the spawn, then the display connection, then the source
  listener. Two endpoints are a parent matrix's broker pair; owner control
  opens first here too, then the parent's interaction, its storage pair and
  the first snapshot of the view it grants. Other shapes are refused.
- [commands.omg](program/commands.omg): owner words over the kernel-installed control
  pair. HEALTH, BIND at the root (the display extent and keys become the
  world's grant, with the compositor's whole tile allowance and the source
  streams bound; under a parent the view is the parent's to grant), the
  LAUNCH_IMAGES, LAUNCH_POLICY and LAUNCH_PLACE policy for a slot, the children,
  storage and stored words below, words routed to a child, LOSS (the canonical
  bare word, distinct from LAUNCH_IMPORTS), CONFIGURE, the
  desktop supervisor's recovery words (a nested matrix's launcher fallback,
  a slot's save-recovery grant, the recovery poll, RESTORE_STORAGE after
  provider replacement), RESOLVE_PATH in this matrix's realm, and the World's
  placement words. Everything else is denied, never silently accepted.

### [connections/](connections/build.omg)

The storage, the parent and the display.

- [upstream.omg](connections/upstream.omg): the parent matrix. Its interaction connection
  is polled for this matrix's view and the one input event each snapshot
  carries, which the World adopts and routes on under the parent's names: when
  the parent rings this matrix's bell, when the last snapshot had more, and
  before an owner word that needs a view none has granted yet. The same
  connection carries every surface, image and batch word upward.
  Regular polling submits once and returns to the turn loop; a completed
  snapshot waits until input retirement permits adoption. Wakeups that arrive
  during the outstanding poll are remembered for the next request.
  [parent/connection.omg](connections/parent/connection.omg) owns poll, visual
  and funding requests and dispatches their exact replies. These three callers
  share the runtime's existing two-request window; backpressure yields without
  resubmitting an accepted request. Setup explicitly requests the v4 pipeline
  handshake; child listeners opt in before accepting a hello. It makes at most
  three receive attempts per turn and contributes the broker's actual wait handles and partial
  transfer deadline. [parent/calls.omg](connections/parent/calls.omg) keeps the
  remaining synchronous callers on that same dispatcher, so they cannot consume
  a poll reply as a visual result. Startup and explicit refreshes still wait;
  regular polling, presentation and launch funding return to the turn loop.
  [omega_parent_poll_test.py](../../../../tools/boot-harness-rs/omega_parent_poll_test.py)
  holds the stock Omega child's poll at a Rust parent and requires an owner
  HEALTH reply before release, followed by delivery of the queued key.
- [listing.omg](connections/listing.omg): names in this matrix's realm, read over its own
  binding one ENTRIES page at a time and compared row by row with the
  32-byte name wanted, which is how discovery, configuration and RESOLVE_PATH
  resolve names without an attachment of this matrix's own;
  [realm_names.omg](realm/realm_names.omg) holds the names startup looks for.
- [presentation.omg](connections/presentation.omg): a child's surfaces on the display, or
  on the parent's view. Creation names the backing behind the owner-allotted
  rank: the compositor tile at the root, a surface of this matrix's own view
  created upstream under a parent. Paint, erase, cursor and sealed image
  updates are clipped and translated by the World, sent as that backing's
  update and committed only after the display or the parent accepts them.
  Inside a child's begin/commit batch they are staged and published together
  under one transaction; the compact rectangles word ([rects.omg](connections/rects.omg))
  is such a batch of one or two paints. Images stay retained behind their
  backing, one per compositor tile, until it is painted over or erased; owed
  erasures flush between turns. A scene switch and a republish after the
  geometry changed replay retained surfaces in one transaction.
- [storage/storage.omg](connections/storage/storage.omg): this matrix's storage. At the root the provider
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
  Child requests retain their exact listener ticket, grant and sealed input in
  [children.omg](connections/storage/children.omg). Each provider link has one
  [exchange.omg](connections/storage/exchange.omg) receiver shared by child
  relays and local work. Containment checks, binding queries and ordinary replies
  resume across turns. A retired child drains its accepted exchange before its
  input is released; the new listener cannot inherit its reply. Grants are
  checked again before a queued operation is sent, including after learning a
  changed upstream binding. Selecting a new provider channel revokes the old
  source's grants before its handshake, including if that handshake fails.
  Startup, owner binding and listing still use
  [synchronous.omg](connections/storage/synchronous.omg); those callers never
  read replies behind the exchange's back. Child launches use retained reads
  and install a prepared binding locally after rechecking authority.

- [source.omg](connections/source.omg): the installed source endpoint. POINTER_SOURCE moves
  the compositor's physical cursor, then routes through the World on the pointer
  stream; KEY_SOURCE routes on the keyboard stream, and RESET cancels both. The
  driver checks that owner words are refused here before it trusts the link.
- [scene.omg](connections/scene.omg): the shared client to the compositor, speaking its
  six-word frames through the runtime's sized requests, with a sealed image
  behind an image update.
- [cursors.omg](connections/cursors.omg): pointer presentation preferences the
  World resolves from the child under each pointer. Under a parent, the
  preference on an inherited pointer goes up under the parent's view once per
  change; at the root the physical pointer's preference sets the compositor's
  pointer style.

### [children/](children/build.omg)

The children this matrix admits, funds, presents and retires.

- [lifecycle.omg](children/lifecycle.omg): RUN_PAGES and RUN_MEMORY stage a slot's next
  allowances; RUN reserves a launch for an image from the owner's catalogue
  under them (frames default to the load, never below one slot's 256 frames,
  plus the private pages and 256 per descendant slot), sets the CPU ceiling and the images
  the child may run in turn, admits it through the runtime's brokered
  admission offering interaction and storage, then connects to the child's
  owner control, which the child accepts as its first act before it asks the
  broker for anything. STOP reclaims the subtree and returns the reservation;
  so does a listener that retires, and the exit goes to the console through
  [report.omg](shared/report.omg). A child's owner control opens without
  blocking: a host with descendants sets up its broker services before it
  accepts control, so the matrix sends the hello, keeps serving, finishes
  the handshake on a later turn once the child answers, and takes the child
  back down if it never does; routed owner words wait with BUSY meanwhile.
  The kernel takes a reservation back only
  once the ended child's frames are no longer charged, so a refused release
  waits and is retried each turn and before any funding is reckoned. An
  ended host's descendant reservations come back under fresh local names and
  are released too, at reclaim and before funding. Every failed step leaves
  no reservation behind.
- [forward.omg](children/forward.omg): requests over the children's owner
  controls, one pending per child. For an owner word routed to a child the
  stock control policy decodes the first hop above the operation into the
  slot and sets the deadline the word's work needs; the word goes on with the
  hop removed and the child's reply answers the owner. The owner's LOSS asks
  every child at once what stopping it would lose and answers, once all have,
  only whether anything might be. A sibling's close asks its target the same
  and stops it only on an answer that nothing would be lost; missing,
  malformed, busy and late answers never count as permission. A late reply is
  drained unanswered, and the slot takes no further request until it has
  been.
- [slots.omg](children/slots.omg): one admitted child's listener per turn. Service
  acquisition goes to the runtime's offer, surface and batch words to
  presentation (a sealed attachment is accepted only behind an image word),
  LAUNCH and LAUNCH_STATUS to the stored launches, FUND to the funding, the
  workspace words and CLOSE to the workspace, every other request to the
  World under the connection's incarnation.
  [responses.omg](connections/responses.omg) retains a computed plain answer when
  the child's reply mailbox is full. Later turns retry that answer, not the
  operation that produced it. Its connection ticket is checked again, and
  reclamation clears it before the slot can be reused. A child can have two
  requests outstanding. [responses.omg](connections/responses.omg) retains the
  first in the deferred slot and keeps a second current if that slot is already
  occupied. Independent funding and rendering keep both exact tickets, with
  backpressure instead of treating an occupied slot as a fatal transport error.
- [funding.omg](children/funding.omg): funding under the stock funding policy. Before
  a launch from the realm this matrix makes sure its own allowances cover the
  child and the connections to it, asking its owner with FUND for any
  shortfall; the root has no one to ask. A child admitted with descendant
  capacity may ask in turn: within its grant's ceiling this matrix covers the
  request, asking its owner first if it must, and grows the child's launch in
  place, asking once more for process slots a retired sibling left out of
  the child's reach. Each child request retains its ticket and reservation;
  local launch funding has its own work item. A retired requester cannot grow
  a replacement child. Accepted parent replies are drained even after cancellation;
  any upstream grant remains this matrix's allowance, without authorizing the
  cancelled launch. Owner/setup admission still uses a synchronous wrapper.
- [stored.omg](children/stored.omg): owner RUN_OBJECT and admission from a sealed
  image. Resolve the declaration, fit the offered capacity, reserve, admit and
  bind the child's storage; unwind a failed step. Startup jobs and retained
  child launches share this admission path. Owner reads and binding still wait.
- [launch/launch.omg](children/launch/launch.omg): a child's LAUNCH reserves its
  exact reply ticket, caller and storage incarnations, grant revision, target
  and standing policy before yielding for the executable. The scoped read uses
  [reads.omg](connections/storage/reads.omg) over the same source exchange as
  app storage. Its result can finish inside a transitional synchronous drain,
  freeing the transport while custody stays with the launch continuation.
  [authority.omg](children/launch/authority.omg) rechecks the caller, grant and
  unchanged policy before admission; cancellation releases late reader rights.
  [placement.omg](children/launch/placement.omg) assigns ranks, places and
  activates the child, or reclaims it on failure. Receipt queries name only the
  requester's launch. [resources.omg](children/launch/resources.omg) resolves
  the executable into an admission plan. The launch retains the sealed image
  while funding waits, then rechecks its authority before installing that plan.
  The login picker remains alive during reads and funding; an invalid image is
  refused before retirement. Failure after retirement queues bounded picker
  recovery. Kernel spawn and brokered admission still wait.

- [workspace.omg](children/workspace.omg): the workspace's app instances
  under the stock catalogue. A launch into an app slot publishes the instance
  and activates it; activation hides the other apps and switches the scene in
  one transaction through presentation, then gives the new app the input
  routes and the chrome its management. APP_ENTRY lists the instances and
  SELECT_APP activates one, for the placed chrome only; when an app ends the
  chrome hears of it and the next instance becomes active. APPLY_LAYOUT
  rearranges the workspace under the stock layout: the World moves every
  placed child at once, the app slots keep their new places for later
  launches, and presentation publishes the moved surfaces again. CLOSE from
  the placed chrome closes the active app; outside a workspace either of the
  first two children closes the other. The reply waits for the target's
  answer.

### [realm/](realm/build.omg)

Setting the matrix up from its realm.

- [jobs.omg](realm/jobs.omg): realm setup under the stock startup policy, which
  sequences each job while this process executes its commands. Jobs run on a
  small stack in the State, one command per turn, so the matrix keeps
  answering its owner and children between them; a command that starts
  another job waits for that job's result. A nested instance discovers its
  realm once the parent has placed it: a Matrix record configures a
  workspace, Login beside Users starts a scoped picker, anything else leaves
  setup to the owner; the owner's CONFIGURE runs the configuration job, its
  reply waiting with it, idempotent once a workspace stands. Later turns replay a failed picker or background within
  the policy's budget, which [recovery.omg](children/recovery.omg) arms from each
  child's end.
- [configuration.omg](realm/configuration.omg): the configuration job. The saved
  record is read and decoded with the stock codec, the launcher it names and
  the optional Background resolved, the geometry and keys validated against
  this matrix's view with the stock placement, the launcher admitted from its
  image under the launcher offer and bound to the whole realm, the app slots
  given the workspace offer and placement as their standing policy, the
  launcher placed as the bottom bar on the allowance's top ranks, the
  background started, the scene flushed and the launcher granted layout,
  management and recovery authority. [background.omg](realm/background.omg) and
  [picker.omg](realm/picker.omg) run their jobs the same way: the background with
  presentation only over the whole view, the picker over the Users folder
  with the second slot opened to the user it chooses.

### [shared/](shared/build.omg)

The the State and the console lines every package uses.

- [state.omg](shared/state.omg): the one State every part holds, with each
  slot's admission, policy, funding and batch, the storage, the workspace's
  instances and the realm setup's progress.
- [report.omg](shared/report.omg): the startup lines the boot harness keys on, in
  the Rust host's words where the two reach the same point, a retired
  child's slot and exit, a failed launch's stage and code, what this matrix
  asks its owner for and why it refused a child's request, a storage link it
  retired and why, its own page and channel use after a launch, and after a
  reclaim the pages and frames still charged and the launch slots free, in
  hex.

Prove it with `python tools/boot-harness-rs/omega_desktop_test.py`: the stock
composition with this program in the root matrix's launch and in the place of
the user template, object three of the stock realm as F5 installs it, and the
ordinary `shell` supervisor, whose login and user entry words reach the Omega
root unchanged. The proof reads the supervisor's and the services' startup
lines, sees the picker offer a new user, creates one and enters it, sees the
root admit the user's matrix over the user's realm and that matrix configure
itself from the realm's saved record, sees its launcher paint its tiles along
the bottom, clicks the Colors tile so the launcher launches Colors into the
workspace, sees Colors paint its view, a healthy indicator and its checker
image through both matrices, moves the pointer over QMP and watches the
compositor's cursor follow it, then clicks Colors for focus and presses
Enter, which saves a record through both storage relays and repaints the view
in the saved colour.

Forwarded images and presentation batches resume across turns through
[transfers/transfer.omg](connections/transfers/transfer.omg). Images retain one
sealed backing; [transaction.omg](connections/transfers/transaction.omg) advances
BEGIN, each translated entry and COMMIT, or ABORT after a rejection. The original
child ticket, child view, parent epoch and transaction revision survive each wait.
Input polling, service acquisition, storage and owner HEALTH remain independent.
Scene mutations and child retirement wait behind publication; failed handshakes
retain their cleanup request. Lifecycle scene replay, binding and
launch still contain synchronous waits. See TASKS_RUST.md for verified scope.

The root display uses [scene.omg](connections/scene.omg) as its sole receiver.
[The backing selector](connections/transfers/backing.omg) translates child
publications to that display or to the supplied parent; both resume across
turns. [Maintenance](connections/transfers/maintenance.omg) retains cursor
updates and owed erasures in the same scene barrier. Hardware input is queued
independently of cursor drawing. Lifecycle scene replay remains synchronous.

Existing-app selection starts at [children/workspace.omg](children/workspace.omg)
and resumes through [children/selection.omg](children/selection.omg). It previews
retained surfaces without changing visibility or input, publishes one transaction
through the shared backing receiver, then commits local demand and focus. The
requester's exact ticket and both app bindings survive the wait. A refusal aborts
the staged scene; invalidation after an accepted commit restores the prior scene
before releasing the barrier. Launch-time activation shares this transaction;
its failure reclaims the new child before answering the launch. Owner layout,
binding and explicit refresh still have synchronous callers.

Owner control enters [program/commands.omg](program/commands.omg). A valid STOP
marks retirement during a pending scene and is acknowledged after the transaction
settles and the child is reclaimed. LOSS reports BUSY during selection, launch
and realm setup. Ordinary image and cursor publication does not block a read-only
loss survey: [children/forward.omg](children/forward.omg) asks the current children
and checks that the same task set remains before reporting clean. Routed owner
words likewise progress independently of drawing. Actual close/reclamation still
waits behind the scene barrier. An expired question is settled before reading any queued
answer; a late clean answer cannot become permission to close.

A forwarded timeout reports an unknown outcome, not cancellation of accepted
work. The control connection retains that pending exchange until its late reply
is drained. `omega_selection_test.py --cancel nested` exercises two Omega relays
under a held commit, observes STOP timing out, then checks eventual cancellation,
restoration, reclamation and a distinct storage-binding reply over the same route.

Sparse control timing pairs each `Matrix control:` value with a `Matrix t:`
sample of monotonic and account-CPU microseconds. The leading stamp is taken at
startup to correlate one instance within a boot; it is not a process identity
or authority token. Events 8/9 record SELECT_APP/CLOSE dispatch and scene queueing.
Run `python tools/boot-harness-rs/control_timing.py SERIAL_LOG` to inspect question
and queue intervals. It reports raw elapsed/CPU spans, outstanding requests and
adjacent diagnostic cost; logging perturbs execution. A span includes this
matrix's other work, and off-CPU time alone cannot identify what it waited for.
