# Sign-in recovery

[main.rs](main.rs) enters [recovery.rs](recovery.rs), which orders local input,
screen publication and owner control replies. [screen.rs](screen.rs) owns the
retained card and button geometry.

The distribution profile bundles this executable in the boot catalogue. The
[startup owner](../session/shell/login.rs) runs it in root slot 0 when the stored
picker cannot start or dies. It receives local matrix interaction and a wake
endpoint, no device grants, executable catalogue or descendant budget. The
current catalogue-admission mechanism also supplies a separate storage endpoint,
but the owner never binds a realm to it and this screen never uses it.

Clicking **TRY AGAIN**, or Enter with focus, sets one pending retry. On the
owner-only control stream, HEALTH returns that bit in word 1 and clears it;
LOSS returns nothing to lose. This distribution-private use of HEALTH is read
only while the owner knows slot 0 contains this program. The screen cannot
restart processes or widen its own grants. A dead/stalled screen is not a retry.

The owner retains this screen through path lookup, then stops it before admitting
the real picker. Failed admission returns here. Retrying never creates names,
formats storage or repairs programs. Display/input/root/provider startup must
already have succeeded; those failures need their own recovery ordering.
