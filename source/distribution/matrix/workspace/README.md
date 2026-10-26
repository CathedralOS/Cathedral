# Existing-app workspace selection

Start at [main.omg](main.omg). Omega sequences scene preparation, waiting,
local activation, restoration and the final reply for a stock desktop switch.
This is distribution policy, not a platform-wide coupling of focus and visibility.

The normal F5 stock matrix enters through
[selection.rs](../../../../source-rs/distribution/matrix/service/children/workspace/selection.rs).
That adapter retains the authenticated chrome request, operation ticket, exact
target/previous tasks and one issued command. It dispatches commands rather than
choosing the successful-switch or rollback sequence. The private value adapter is
[policy/workspace.rs](../../../../source-rs/distribution/matrix/policy/workspace.rs).
No task or capability handle crosses the Omega policy packet.

Commands issue once and complete only for their matching ticket and command.
While waiting, the host continues ordinary input/service work. Cancellation
retains the first refusal and drains the outstanding scene; it does not manufacture
a completion or abandon a submitted commit. A committed refused switch requires
restoration before replying. An uncommitted refusal does not activate locally.
Restoration cannot itself be cancelled; failure is fatal, never successful cleanup.

[custody.rs](../../../../source-rs/distribution/matrix/service/children/workspace/custody.rs)
checks authority and exact task/connection incarnations. The host rechecks before
commit and after its reply, rejects replies to replacement chrome, and defers
mutations until scene custody ends. These are enforcement facts, not app-supplied
identities. [scene.rs](../../../../source-rs/distribution/matrix/service/children/workspace/scene.rs)
executes the yielding scene operations. The parent retains immutable images and
shared IPC replies. A late accepted scene can briefly appear before restoration;
this is compensation, not instantaneous revocation.

The Rust event loop, selection admission checks, local visibility/input mutation
and synchronous fallback activation still exist. This slice moves the ordinary
SELECT_APP workflow, not every workspace mechanism or the entire matrix process.

Verification entrances:

- `cargo test --locked -p cathedral-stock-matrix --lib` executes compiled Omega,
  including wrong/duplicate completions, cancellation, late commit and failed restore.
- `python tools/boot-harness-rs/desktop_test.py --selection-async MODE --trace-storage --timeout 240`
  exercises `complete`, `failure`, `stop` and `late-stop` in the F5 composition,
  including input while waiting, repeated switches, protected drafts and cold reopen.
