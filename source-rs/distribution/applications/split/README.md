# Split: an ordinary nested matrix

From F5, enter a user and choose **Split**. It runs two temporary Notes windows
side by side. Click one to type. Each Notes window owns its X and dirty-close
dialog. Split's top-right X asks both children about unsaved work; **UNSAVED
NOTES** keeps the workspace open until the drafts are discarded or children close.
The outer desktop's Settings can resize it without restarting its children.

Start at [main.rs](main.rs), then [session/session.rs](session/session.rs): acquire
the supplied interaction/storage services, load children, consume upstream
notifications, survey loss asynchronously, and dispatch authenticated requests.

- [Children](session/children.rs) reads `Notes` from this program's own realm,
  validates its sealed import declaration, reserves two tasks, and uses platform
  admission with an interaction-only offer. No child receives storage, catalogue,
  device or descendant authority. Closing a child returns its reservation.
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
same code and hold independent drafts.

The current scope is deliberately bounded: no child persistence, dynamic child
chooser, restart policy, or transactional presentation batches. Unsupported child
operations are denied. Loss of the parent transport ends the subtree; a revoked
view withdraws child views. A later usable grant is arranged again by this program.
Read-only images remain shared along the chain. Capacity and transport failures
are not a complete recovery UI yet.

`python tools/boot-harness-rs/desktop_test.py --nested` exercises the installed
program through the ordinary F5 composition, including focus isolation, denied
child storage, dirty close, live resize, repeated subtree teardown and cold boot.
The host `World` test separately covers parent revocation/regrant; the visible
fixture does not inject that control operation into the outer desktop.
