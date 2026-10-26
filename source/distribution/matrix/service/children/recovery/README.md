# Asynchronous launcher recovery

[main.omg](main.omg) owns named `Recovery` state and its `Phase`. Start at
`next_work` for authority/vacancy checks and command selection; follow
`read_started`, `read_completed` and `admission_finished` for matching completion
handling. `Command::Admit` carries its correlation ticket as a case payload.
The policy never reads or writes native packet slots.

It decides whether the stock matrix waits, reads the launcher's
stored program, admits the result, or shows owner-supplied fallback code. A failed
initial launcher gets one automatic attempt. Failure after that goes to fallback.
An authenticated fallback request permits one explicit attempt; it never restores
the automatic allowance. Clean exit, owner stop and fallback failure do not restart.

Pending recovery waits for upstream/storage and child-control work. A changed
parent grant, storage binding or occupied launcher slot cancels it. An acknowledged
explicit retry may retire its own fallback before reading. Replacing or revoking
fallback code cancels the saved context, including a queued request. Initial
fallback supply preserves the budget only while the launcher is running.

The [Rust adapter](../../../../../../source-rs/distribution/matrix/policy/recovery.rs)
holds two opaque workflow words. Omega emits each read/admission command once,
then waits for its matching completion. Rust issues a nonzero, process-local
monotonic ticket for each read; replacement launchers share that allocator, so
an old operation cannot acquire a new meaning. Exhaustion refuses a new read.
The ticket is a correlation value, not authority to access an image or process.
The
[recovery orchestrator](../../../../../../source-rs/distribution/matrix/service/children/recovery/recovery.rs)
retains the exact task, endpoint incarnation, saved placement, storage identity and
pending image. The storage exchange carries a typed caller-or-recovery origin;
there is no longer an unlabelled local completion delivered to whichever launcher
exists. The [completion handler](../../../../../../source-rs/distribution/matrix/service/children/recovery/completion.rs)
feeds read success/failure to Omega and retains a sealed image only when accepted.
Stale, duplicate or cancelled completions release their attachment. Admission must
name the exact held ticket and reports its own completion separately.
Rust authenticates requests, performs asynchronous reads, restores
only the captured authority and reclaims failed admissions. Sibling processes keep
running; restarting the launcher does not restart its matrix or recover its private
unsaved state.

The [Omega bridge](../../../bridge/policy/recovery.omg) alone translates
phases, commands and results to host words. This private native bridge uses
operation 9, event in word 1, value facts in 2-4, phase/ticket state in 16-17 and an admission-command ticket in 24. Events
0-3 initialize, retire, check readiness and advance the workflow. Events 4-5
report the issued ticket and read completion; 6-9 finish fallback, request retry,
change fallback and finish explicit retirement; 10 refuses an unavailable read;
11 completes admission. It is not a public IPC contract. No PID, endpoint or
capability enters Omega's packet.

Cancellation keeps the existing owner-command ordering. A control request that
needs a busy storage stream is deferred; recovery cannot admit ahead of that
queued request. Processing stop discards recovery even when it reports that the
crashed task is already gone (`PEER_CLOSED`); successful revocation also discards
the saved context and any image. The stream is still drained by Rust, with its
existing deadlines and retirement rules, so a late wire reply cannot satisfy
another storage exchange.
The matrix continues handling independent input and presentation during a read.
Admission and fallback mechanisms remain synchronous host operations; this slice
does not replace the whole matrix event loop or all upstream calls.

Compiled policy tests exercise repeated explicit attempts, cancellation and
authentication facts. `python tools/boot-harness-rs/launcher_recovery_test.py IMAGE`
copies an existing one-user desktop image and runs seven live modes: recovery,
clean exit, startup fault, owner stop, owner revoke, fallback clean exit and
fallback fault. It checks unsaved sibling Notes content, exact bounded restart
counts, explicit retry and fresh/cold recovery. The input image is left unchanged.

`python tools/boot-harness-rs/desktop_test.py --launcher-async recover --timeout 240`
boots the full F5 composition with disposable probes. Its nested user matrix holds
a real recovery read for three seconds; sibling Notes must consume another key
before release, protect its draft, save and reopen on cold boot. `revoke` and
`stop` variants send actual routed owner commands while the read is held and
require no later admission or fallback. All variants close the subtree afterward.
Only these fixture profiles enable the gate and Rust launcher fault keys; ordinary
F5 has neither. Compiled tests additionally cover stale completion after a new
launcher, duplicate replies, wrong tickets and allocator exhaustion.
