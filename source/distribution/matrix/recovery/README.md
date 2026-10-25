# Launcher recovery policy

[main.omg](main.omg) decides whether the stock matrix waits, reads the launcher's
stored program, admits the result, or shows owner-supplied fallback code. A failed
initial launcher gets one automatic attempt. Failure after that goes to fallback.
An authenticated fallback request permits one explicit attempt; it never restores
the automatic allowance. Clean exit, owner stop and fallback failure do not restart.

Pending recovery waits for upstream/storage and child-control work. A changed
parent grant, storage binding or occupied launcher slot cancels it. An acknowledged
explicit retry may retire its own fallback before reading. Replacing or revoking
fallback code cancels the saved context, including a queued request. Initial
fallback supply preserves the budget only while the launcher is running.

The [Rust adapter](../../../../source-rs/distribution/matrix/policy/recovery.rs)
holds one opaque policy word. The
[recovery orchestrator](../../../../source-rs/distribution/matrix/service/children/recovery/recovery.rs)
retains the exact task, endpoint incarnation, saved placement, storage identity and
pending image. Rust authenticates requests, performs asynchronous reads, restores
only the captured authority and reclaims failed admissions. Sibling processes keep
running; restarting the launcher does not restart its matrix or recover its private
unsaved state.

This private native bridge uses operation 9, event in word 1, verified boolean
facts in 2-4 and policy state in 16. Events initialize, retire, check readiness,
choose an action, start/finish a read, finish fallback admission, request an explicit
retry, change fallback, and finish explicit retirement. It is not a public IPC
contract. No PID, endpoint or capability enters Omega's policy packet.

Compiled policy tests exercise repeated explicit attempts, cancellation and
authentication facts. `python tools/boot-harness-rs/launcher_recovery_test.py IMAGE`
copies an existing one-user desktop image and runs seven live modes: recovery,
clean exit, startup fault, owner stop, owner revoke, fallback clean exit and
fallback fault. It checks unsaved sibling Notes content, exact bounded restart
counts, explicit retry and fresh/cold recovery. The input image is left unchanged.
