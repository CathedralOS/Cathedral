# Existing-app workspace selection

Start at [main.omg](main.omg). [catalogue.omg](catalogue.omg) owns the desktop's
app instances, monotonic tickets, selected instance, switch admission and fallback
choice. [selection.omg](selection.omg) sequences scene preparation, waiting,
local activation, restoration and the final reply for a stock desktop switch.
This is distribution policy, not a platform-wide coupling of focus and visibility.

Publication records an instance only after its launch scene succeeds. Repeated
launches of the same program remain distinct instances. Retiring an instance
removes its catalogue entry and clears selection only if it was selected. A
vacant selection falls back to the first surviving app position; retiring a
background, chrome or inactive app cannot displace an existing selection.
Tickets never wrap or restart on slot reuse. Exhaustion refuses further publication.
Unknown tickets remain missing even while busy; a known ticket waits behind
ongoing work even when it names the current app. These preserve the stock desktop's
existing behavior, not a policy imposed on other matrices.

Rust retains one opaque catalogue through
[policy/catalogue.rs](../../../../../../source-rs/distribution/matrix/policy/catalogue.rs),
not a parallel `active`/`running`/ticket-counter representation. Queries pass a
snapshot through the same compiled Omega code. The catalogue's object numbers
and instance tickets are local display/selection identifiers, never task handles
or service grants. Authenticated chrome still gates listing and selection.

The normal F5 stock matrix enters through
[selection.rs](../../../../../../source-rs/distribution/matrix/service/children/workspace/selection.rs).
That adapter retains the authenticated chrome request, operation ticket, exact
target/previous tasks and one issued command. It dispatches commands rather than
choosing the successful-switch or rollback sequence. The private value adapter is
[policy/workspace.rs](../../../../../../source-rs/distribution/matrix/policy/workspace.rs).
No task or capability handle crosses the Omega policy packet.

Commands issue once and complete only for their matching ticket and command.
While waiting, the host continues ordinary input/service work. Cancellation
retains the first refusal and drains the outstanding scene; it does not manufacture
a completion or abandon a submitted commit. A committed refused switch requires
restoration before replying. An uncommitted refusal does not activate locally.
Restoration cannot itself be cancelled; failure is fatal, never successful cleanup.

[custody.rs](../../../../../../source-rs/distribution/matrix/service/children/workspace/custody.rs)
checks authority and exact task/connection incarnations. The host rechecks before
commit and after its reply, rejects replies to replacement chrome, and defers
mutations until scene custody ends. These are enforcement facts, not app-supplied
identities. [scene.rs](../../../../../../source-rs/distribution/matrix/service/children/workspace/scene.rs)
executes the yielding scene operations. The parent retains immutable images and
shared IPC replies. A late accepted scene can briefly appear before restoration;
this is compensation, not instantaneous revocation.

The Rust event loop, authenticated admission/resource checks, local visibility/input
mutation and synchronous fallback activation mechanism still exist. The fallback
choice comes from Omega; changing the scene still uses the existing host mechanism.
This is not yet the entire matrix process in Omega.

Verification entrances:

- `cargo test --locked -p cathedral-stock-matrix --lib` executes compiled Omega,
  including wrong/duplicate completions, cancellation, late commit, failed restore,
  instance reuse, ticket exhaustion, inactive retirement and every surviving-app subset.
- `python tools/boot-harness-rs/desktop_test.py --selection-async MODE --trace-storage --timeout 240`
  exercises `complete`, `failure`, `stop` and `late-stop` in the F5 composition,
  including input while waiting, repeated switches, protected drafts and cold reopen.
