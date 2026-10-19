# Split: an ordinary nested matrix

From F5, enter a user and choose **Split**. It runs two persistent Notes windows
side by side. Click one to type. Each Notes window owns its X and dirty-close
dialog. Split's top-right X asks both children about unsaved work; **UNSAVED
NOTES** keeps the workspace open until the drafts are saved, discarded or children close. F1 saves either note;
its saved text survives closing Split and rebooting.
The outer desktop's Settings can resize it without restarting its children.

Start at [main.rs](main.rs), then [session/session.rs](session/session.rs): acquire
the supplied interaction/storage services, load children, consume upstream
notifications, survey loss asynchronously, and dispatch authenticated requests.

- [Children](session/children.rs) reads `Notes` from this program's own realm,
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
- [Rendering](session/rendering.rs) chooses two columns. Platform `World<2>` owns
  clipping, local names, focus and grant invalidation. Shared image custody keeps
  sealed readers without mapping child pixels. There is no intermediate pixel copy.
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

Split currently forwards through three matrix scope stamps (root, user, Split),
with an 800-tick deadline above the stock user/root's 700/600. This is a bounded
reference chain, not unbounded recursion. The stock launcher funds ten channels
for its two children. A 24-handle process cap covers five supplied endpoints,
eight retained rights while the first child acquires services, and ten temporary
rights while admitting the second. The distribution supplies 32 root channels,
lends 20 to the user matrix and 10 to Split; the global backing pool has 36.
All reservations remain conserved; ordinary Notes still creates no channels.

`Left`/`Right` are resolved by name on each launch. Initial empty-object creation
and naming are separate transactions; interrupted first-time naming can leave an
unnamed empty object in Split's own realm. No established child data is overwritten.
Atomic named creation remains follow-up storage work.
