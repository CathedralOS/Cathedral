# Split: an ordinary nested matrix

From F5, enter a user and choose **Split**. It runs two persistent Notes windows
side by side. Click one to type. Each Notes window owns its X and dirty-close
dialog. Split's top-right X asks both children about unsaved work; **UNSAVED
NOTES** keeps the workspace open until the drafts are saved, discarded or children close. F1 saves either note;
its saved text survives closing Split and rebooting.
The outer desktop's Settings can resize it without restarting its Notes children.

**AUTO** starts a separate input participant. A yellow cursor appears over the
right document and the participant types `bot`, then keeps moving its pointer.
You can click and type in the left document simultaneously: each input group keeps
its own selection. **STOP** ends that participant and removes its cursor. Resizing
ends the run too; AUTO explicitly starts a fresh grant. F1 still saves each note.
This is a fixed demonstration script, not a general automation interface.

**HIDE RIGHT / SHOW RIGHT** removes/restores the right Notes presentation. Start
AUTO, then hide the right note while it types: it keeps editing but defers drawing.
Showing it presents the updated document. Hidden content cannot receive new
pointer hits; AUTO's already-selected private keyboard route remains authorized.
The control appears when the grant is wide enough (384 pixels). It neither stops
the app nor changes its local dimensions. The stock desktop separately chooses
to deactivate input when switching apps.

**CAPTURE / CLEAR** starts/stops a separate viewer of the right Notes. It can
request a fresh frame while that note is hidden, without raising it or changing
its input route. The owner supplies a 200 by 104 pixel crop, with the known text
area blacked out before any image reaches the viewer. The preview appears over
the left column. This fixed demonstration mask is not a general privacy editor.
The control requires a 512-pixel-wide view and sufficient granted capacity.

Start at [main.rs](main.rs), then [session/session.rs](session/session.rs): acquire
the supplied interaction/storage services, load children, consume upstream
notifications, survey loss asynchronously, and dispatch authenticated requests.

- [Children](session/children/children.rs) reads `Notes` from this program's own realm,
  validates its sealed import declaration, reserves two tasks, and uses platform
  admission with interaction and a separate scoped storage connection. The two
  `Left` and `Right` realms are siblings beneath Split, with independent contents.
  Neither child receives the parent realm, its sibling realm, devices or descendant
  authority. Closing a child returns its reservation.
- [Storage](session/storage/storage.rs) refreshes the upstream binding, delegates
  scope validation/stamping to the same platform `Scopes` used by the stock matrix,
  and progresses one bounded exchange without blocking interaction. Attachments
  travel unmapped. A changed upstream binding invalidates old child revisions;
  Split restores its standing grants only if the newly authorized realm is the
  same one. A transport timeout retires the stream, never reuses a late reply.
- [Rendering](session/rendering/rendering.rs) chooses two columns. Platform `World<4>` owns
  clipping, local names, focus and grant invalidation. Shared image custody keeps
  sealed readers without mapping child pixels during ordinary forwarding. Capture
  temporarily maps the source readers and rasterizes one bounded masked output.
- [Producer lifecycle](session/producer/producer.rs) negotiates incoming capacity,
  admits the sealed [Producer program](../input-demo/main.rs) with an explicit input offer, and owns its
  cursor surface. Its [receiver](session/producer/requests.rs) authenticates the
  separate broker/producer endpoints; the producer cannot draw or access storage.
  Two channels and a private group use the same platform routing policy as physical
  input. The third world slot is a non-rendering participant; Notes occupy two.
  [Cursor rendering](session/producer/rendering.rs) resolves that producer's own
  hover/capture target. [Inherited preferences](session/rendering/cursors.rs)
  propagate separately, so hiding one pointer does not hide another or withdraw
  its input. Notes F6 toggles preferences for its keyboard's local group.
- [Capture lifecycle](session/capture/capture.rs) admits an explicitly offered
  viewer in the fourth slot, funds one pending output, requests a source frame,
  and releases demand on completion or failure. Its
  [request dispatcher](session/capture/requests.rs) authenticates separate broker
  and capture endpoints. [Raster integration](session/rendering/capture.rs) pins
  the accepted local scene for one dispatch, masks it, then freezes output before
  transfer. The viewer has no storage, input-production or private-pixel allocation
  grant. Source/recipient loss or parent geometry changes retire the demo.
- [Requests](session/requests.rs) distinguishes owner control from installed child
  streams. [Loss](session/loss.rs) keeps serving children during the survey. Missing
  or malformed replies prevent close; a timed-out control stream stays uncertain
  until that child exits, so a late reply cannot authorize a later close.

This program imports no stock matrix implementation or desktop contract. The
layout checker enforces that boundary. Its installer entry is a normal executable
under the user realm, with a `Notes` program beneath it; two instances load that
same code and hold independent drafts and persisted notes.

The current scope is deliberately bounded: no dynamic child chooser, restart policy, or transactional presentation batches. Unsupported child
operations are denied. Loss of the parent transport ends the subtree; a revoked
view withdraws child views. A later usable grant is arranged again by this program.
Read-only images remain shared along the chain. Capacity and transport failures
are not a complete recovery UI yet.

`python tools/boot-harness-rs/desktop_test.py --nested` exercises the installed
program through the ordinary F5 composition, including focus isolation, denied
cross-scope access (shared policy tests), dirty close, live resize, repeated subtree teardown and cold boot.
The host `World` test separately covers parent revocation/regrant; the visible
fixture does not inject that control operation into the outer desktop.

The **DEEPER** button asks the current children about unsaved work, collects
clean children, and runs the same Split executable under a `Nested` realm. Its
code is a named immutable content copy; missing `Notes`, `Left` and `Right`
subtrees are copied individually. Every copy is atomic. An interrupted setup
resumes by name and never replaces existing nested data. The complete setup is
not one transaction; it is kept private until admission. Reopening Split starts
its normal two notes; DEEPER reopens the saved nested workspace. Copies are
snapshots, not enrollment in managed code updates.

The desktop lends Split at most sixteen surfaces, leaving room for ordinary
sibling apps while funding its optional producer and capture viewer.
Split grants nine surfaces and a bounded two-child budget to its nested
matrix. Its inner DEEPER refuses with **NO ROOM** while preserving live notes.
Split declares thirty-eight channel/notification allowances and four descendants:
its two Notes use eighteen allowances (five queues plus two notifications at each
end per child), its own interaction/storage connections use two notifications,
the optional producer and capture viewer each use five queues plus two
notifications at each end. The producer reserves 18 private and two reader pages;
the viewer reserves two private pages for mailboxes and 34 reader pages. Capture
output stays funded by Split until all readers release it. Before admitting the viewer, Split
requests any missing physical backing through its owner's `FUND` protocol. It
retains a process-load allowance plus its private-page budget for its own work;
owner refusal leaves the existing applications running and shows CANNOT CAPTURE.
The inner Split receives
117 private pages, 69 readers and twenty channel/notification allowances; it has
no room for AUTO alongside its two Notes. The private grant covers both 41-page
Omega Notes reservations, two 14-page header versions during replacement and
shared mailbox writers. Each Notes also receives six reader pages and two
notification allowances. The enclosing desktop's ordinary
app share is capped at 168 private and 144 reader pages, covering that loan or
its optional children, mailboxes, masked output and retained/replacement images
for programs requesting descendant capacity (ordinary leaves retain their 128-page ceiling). Root/user budgets come
from the current distribution composition
and owner admission policy. No child acquires devices or
ambient code/storage authority. Each Notes host creates its interaction and
storage notifications; its connection queues are supplied by the owner.

Split selects shared child storage before admission. Its storage coordinator owns
one upstream shared Client across local setup reads and forwarded child requests,
so request IDs do not restart when a temporary Scope is dropped. Forwarding yields
on IPC lifetime and notification interests, bounded by the original storage and
partial-attachment deadlines. Parent failure retires the connection without replay.
Broker acquisition and interaction share one persistent `Services` client, owned
by Output across drawing, polling and later service acquisition. All eight child
slots are selected for shared transport before admission: two interaction, two
storage, producer broker/data and capture broker/data. Each producer/viewer owns
one broker Client and a separate service Client, preserving request IDs and
immutable image custody through later replies. Capture dispatch temporarily uses
the viewer's world slot, then restores the broker slot before replying; transport
correlation never uses the remapped world slot. Input wake hints use independent
notifications: the owner retains signal/control and the child acquires only wait
authority. [Child control](session/children/control.rs) opens a separate shared
connection while the service loop continues answering startup requests. LOSS
queries use exact transport correlation; timeout leaves that child's loss unknown
until it exits, without killing it or reusing its late answer. The same Split
program accepts this control protocol when nested. Its outer stock owner still
uses copied control pending that owner's migration. Removal retires shared slots before closing remaining child
rights; only BAD_HANDLE for the exact, never-reused endpoint names is harmless.

Scope restrictions are folded only after upstream containment checks; a shared
absolute deadline bounds the full exchange. See the platform
[delegation contract](../../../platform/libraries/matrix/world/storage/README.md).
This removes the packet's fixed depth cap without promising unlimited resources
or cheap unbounded recursion. The current proof path adds reads; it is not cached.

`Left`/`Right` use atomic named creation: content, hierarchy edge, name/type
indexes and quota publish together. A collision resolves the existing name;
uncertain transport aborts setup and the next launch reconciles by lookup.
`python tools/boot-harness-rs/desktop_test.py --deep --trace-storage` adds actual
same-code nesting, independent edits, dirty-close propagation, capacity refusal,
repeated collection and cold reboot to the usual visible Split checks.

`python tools/boot-harness-rs/desktop_test.py --participants --trace-storage`
checks the default installed AUTO participant, independent physical/synthetic
selection, confined visible cursor, repeated stop/start in the same matrix,
resize revocation with surviving Notes, saved edits and cold reopen. Cursor assets,
target-dependent suppression, arbitrary participant discovery and more than the
bounded four input channels remain outside this demo.

The disposable `--participant-fault hold`, `disconnect` and `pressure` workflows
use that same admission/IPC path with instrumented participant/Notes images.
They observe real receiver channel/group retirement while input is held, endpoint
loss, and overflow during a three-second receiver stall. The pressure case checks
that another channel queued to the same app survives, that the left app remains
usable, and that fresh input works after cancellation. All three repeat admission
in the same matrix, check non-reuse of retired identities and cold-open saved edits.
The harness requires logged flood/stall ordering; it does not infer overflow from
elapsed time alone. These are bounded QEMU proofs, not a latency guarantee or
exhaustive fault coverage. The ordinary F5 images do not contain these probes.

AUTO describes its channels as `AUTO pointer` and `AUTO keys`. Descriptions travel
once before input on each receiving edge, while ordinary events retain compact
channel tags. The receiver probes query the scoped directory and require both
those labels and the inherited generic `Pointer`/`Keyboard` descriptions. These
are virtual input hints, not authenticated participant identities or badges.

`python tools/boot-harness-rs/desktop_test.py --render-demand` exercises those
installed controls with disposable observer features. It requires actual edits
while Notes acknowledges no frame demand, no right-child surface publications
in that interval, independent sibling editing, no fresh hits into the hidden
area, correct restored content, repeated admission and exact cold-open text.
The observers add no authority and compile out of ordinary F5. Visibility replay
is per surface; this demo does not promise an atomic multi-surface transition.
Capture-generated demand and masks are not implemented by this workflow.

The manifest requests 43 channel/notification allowances and four descendant slots.
Each Notes child receives 42 private pages, seven reader pages and three
notification allowances, including its control mailbox. Nested Split receives
122 private pages, 74 reader pages and 25 channel/notification allowances. These
are explicit lab grants; the kernel's notification-handle ceiling is unchanged.
The deep harness requires successful shared-control setup for every admitted
leaf and nested Split, alongside its visible close/persistence assertions.
