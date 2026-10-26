# Local startup recovery

[main.omg](main.omg) owns the stock matrix's picker and background retry state.
Start with `Startup`, `ChildRole` and `Context`, then `child_retired` for failure
routing. `take_picker` and `take_background` consume queued work; `picker_started`
counts attempts and selects the display mode. Results are named `RetirementEffect`,
`PickerAction` and `PickerMode` values. This policy never indexes a native packet.
The picker gets two total admission attempts. Once exhausted, the host returns
failure so its owner can report the failed nested desktop. A new explicit launch
creates a fresh matrix. A failed selected user, or rejected admission after the
picker has been retired, resets the picker budget and requests its error display.

A failed background gets one retry. The allowance is consumed before admission,
so another failure cannot create a restart loop. Initial background admission
failure during explicit workspace configuration also queues that attempt. Picker
and background work have separate state; resetting one does not replenish the
other. Clean exits and owner stops do not request recovery. An owner-managed
login picker leaves both crash and admission-failure recovery to that owner.
In particular, invalid root Login code must return a refusal to the desktop's
recovery screen; it cannot queue a local picker whose realm was never configured.

The [Rust coordinator](../../../../../../source-rs/distribution/matrix/service/children/startup/startup.rs)
resolves the realm, reads programs and admits them. Its
[lifetime handler](../../../../../../source-rs/distribution/matrix/service/children/startup/lifetime.rs)
supplies verified task outcomes and local roles, waits for child-control/loss work,
and executes the selected actions after teardown. Rust retains realm object IDs,
task identity and authority. The selected-user admission path reports rejection
only after checking its actual remaining slots. No ancestor reads the inner realm
to select a replacement program.

The [Omega bridge](../../../bridge/policy/startup.omg) translates those values
for the [Rust native adapter](../../../../../../source-rs/distribution/matrix/policy/startup.rs),
which holds five opaque words. Private bridge operation 10 uses event in word 1, value
facts in 2-7 and policy state in 16-20. Events report retirement, count picker
admission, report selection failure, report initial background admission failure,
take pending picker work and take pending background work. Busy checks retain
pending work. This is a temporary local bridge, not an app IPC contract.

Compiled tests cover all retirement fact combinations, owner/local distinction,
attempt exhaustion, blocked work and independent budgets. The ordinary nested
desktop test supplies a fresh current-code image; `nested_failures_test.py IMAGE`
then injects picker, selected-user and background faults into separate copies.
It checks exact retry counts, repeated subtree reclamation, usable inner chrome
after background failure and an unsaved outer Notes draft throughout.
