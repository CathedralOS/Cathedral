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
- [Rendering](session/rendering.rs) chooses two columns. Platform `World<3>` owns
  clipping, local names, focus and grant invalidation. Shared image custody keeps
  sealed readers without mapping child pixels. There is no intermediate pixel copy.
- [Producer lifecycle](session/producer/producer.rs) negotiates incoming capacity,
  admits the sealed [Producer program](../input-demo/main.rs) with an explicit input offer, and owns its
  cursor surface. Its [receiver](session/producer/requests.rs) authenticates the
  separate broker/producer endpoints; the producer cannot draw or access storage.
  Two channels and a private group use the same platform routing policy as physical
  input. The third world slot is a non-rendering participant; Notes occupy two.
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

The wrapper grants nine surfaces and a bounded two-child budget to its nested
matrix. Its inner DEEPER refuses with **NO ROOM** while preserving live notes.
Split declares fifteen channels and three descendants: its two Notes use ten
channels, and the optional producer uses five more. The inner Split receives
80 private pages, 64 readers and ten channels; it has no room for AUTO alongside
its two Notes. Root/user budgets come from the current distribution composition
and owner admission policy. No child acquires devices or
ambient code/storage authority. Ordinary Notes still creates no channels.

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
