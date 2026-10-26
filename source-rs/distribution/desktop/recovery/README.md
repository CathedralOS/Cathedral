# Sign-in recovery

[main.rs](main.rs) enters [recovery.rs](recovery.rs), which orders local input,
screen publication and owner control replies. [screen.rs](screen.rs) owns the
retained card and button geometry.

The distribution profile bundles this executable in the boot catalogue. The
[startup owner](../session/shell/login.rs) runs it in root slot 0 when the stored
picker cannot start or dies, and while the storage provider starts. It receives
a scoped service broker, local matrix interaction and a wait-only notification,
with no device grants, executable catalogue or descendant budget. Control setup
completes before service acquisition. Drawing commands and immutable card images
use one retained shared interaction connection; there are no copied payload calls.
The current boot-catalogue grant still offers storage, but the owner never binds
a realm to it and this screen never acquires it. Launcher fallback admission offers
only interaction and explicitly verifies that storage acquisition is unavailable.

While storage opens, **STARTING...** has no active button. Clicking **TRY AGAIN**
after failure, or Enter with focus, sets one pending retry. On the
owner-only control stream, HEALTH returns that bit in word 1 and clears it;
LOSS returns nothing to lose. [protocol.rs](protocol.rs) defines STATUS for the
owner to switch between starting and failed presentation. The owner revokes the
old input lease and grants a fresh one around that change, so clicks made during
readiness cannot become queued retries. The pending retry bit is cleared too.
This distribution-private use of HEALTH is read only while the owner knows
slot 0 contains this program. The screen cannot
restart processes or widen its own grants. A dead/stalled screen is not a retry.

The owner retains this screen through path lookup, then stops it before admitting
the real picker. Failed admission returns here. Retrying never creates names,
formats storage or repairs programs. Display/input/root startup must already
have succeeded. Storage opens asynchronously with a 3,000-tick readiness bound;
failure cancels/collects that exact task and retires its control endpoints.
An explicit retry funds one attempt, preserving the automatic restart counter.
