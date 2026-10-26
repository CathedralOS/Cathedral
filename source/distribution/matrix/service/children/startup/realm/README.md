# Realm startup and setup

Start at [main.omg](main.omg). It initializes one job, issues one host operation,
accepts its matching completion once, and descends into the owning sequence:

- [discovery.omg](discovery.omg): refresh the inherited view, inspect one scoped
  realm listing, configure a workspace or start a picker, then enter ordinary
  startup recovery. A Matrix configuration wins over Login/Users; both picker
  entries must exist. An unplaced matrix finishes this turn without marking
  startup complete or consuming recovery. Missing realm authority leaves setup
  to its owner; other inspection failures propagate.
- [configuration.omg](configuration.omg): read/validate saved configuration,
  admit the launcher, bind storage, install app policy and place chrome. Only then
  start the optional background, publish the scene and grant chrome authority.
  Already configured is idempotent, even while children exist; other occupied or
  storing arrangements refuse BUSY.
- [picker.omg](picker.omg): arm the existing bounded recovery policy before
  reading the picker, admit it over Users, place it, install the selected user's
  standing grant, and notify. Setup failure propagates to the enclosing matrix.
- [background.omg](background.omg): read, admit, place and record the background.
  Failed placement stops the admitted background before returning the error.

A launcher that did not acquire storage is stopped. Storage refusal also stops
it. Failed launcher placement clears workspace/standing policy before stop, then
resets the app surface range. Neither cleanup path grants chrome authority or
reports configuration success. Background failure during configuration is
different: it arms the existing one-retry policy and lets configuration finish.
The refusal remains available to the host diagnostic without becoming the final
configuration error.

The ordinary F5 stock matrix executes these sequences, including nested instances
and local picker/background retries. [operations.omg](operations.omg) is private
host vocabulary, not a new service API. State retains only the issued command,
job, result and cleanup obligation. Images, IDs, grants and exact child records
remain in the host. Issuing a command twice or completing a different command
cannot advance setup. The native bridge shares the process with Rust and is not
an isolation boundary.

The Rust reading path is
[startup/startup.rs](../../../../../../../source-rs/distribution/matrix/service/children/startup/startup.rs)
for discovery and
[configuration/configuration.rs](../../../../../../../source-rs/distribution/matrix/service/children/configuration/configuration.rs)
for applying configuration. Descend into `workspace.rs` for configuration
resources, `background.rs` for background resources, and `startup/picker.rs` for
the picker. `configuration/objects.rs` owns immutable reads and remembered storage
bindings; `layout.rs` retains live layout application. `mod.rs` is plumbing only.

Host mechanism errors still abort the current invocation, retaining its existing
caller error handling. Known protocol refusals and local launcher/background
placement results go through Omega's cleanup decisions. This preserves the old
behavior; it neither makes startup asynchronous nor promises cleanup after an
unrecoverable transport/kernel failure. The executable image remains host-owned
until its existing release point, including during rollback.

Compiled tests in [tests/realm.rs](../../../../../../../source-rs/distribution/matrix/tests/realm.rs)
exercise setup ordering, canonical completion, all realm-entry subsets, deferred
startup, rollback, nonfatal background refusal and terminal failures. Guest
verification and exact scope are in `TASKS_RUST.md`.

[Stock resource offers and profile ceilings](../../launch/resources/README.md) now execute
in Omega too. The host composes exact images/scopes into launch plans and owns
grants, allocation and transport funding. Service matching and live-layout plans
also execute in Omega; image decoding and applying/publishing layout remain host
work. This is not a fully Omega matrix executable or a new platform runtime.
