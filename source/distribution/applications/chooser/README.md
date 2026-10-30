# Chooser: Omega login picker and desktop taskbar

The installed chooser runs entirely through the common Omega app host. Its
[Rust entry](../../../../source-rs/distribution/applications/chooser/main.rs)
declares required interaction/storage imports and passes its startup argument
to the host. No chooser state, rendering, settings or input policy lives in the
normal Rust executable. The argument selects picker/taskbar presentation and
names the launcher's own scoped object; it grants no authority.

The app now accepts named `State` and `Exchange` values and returns a `Command`.
`Exchange` names its frame, payload, transport result, startup argument and wait
request. Its 4,080-byte payload fits one directory page or a bounded raster
chunk. `Phase` names the pending continuation. No host opcode or 4,096-word
packet is part of this app-facing interface.

Start at [app/main.omg](app/main.omg), then
[session/session.omg](app/session/session.omg): acquire supplied services,
list the realm, refresh local observations, poll, dispatch input, present and
wait. An optional 100-tick idle refresh observes task collection after the last
surface notification. Descend through these decisions:

- [Catalogue](app/catalogue/catalogue.omg) streams bounded `ENTRIES` pages,
  resolves hidden roles and publishes the visible names and objects.
- [Input](app/session/input.omg) tracks pointer edges per scoped channel and
  routes to task selection, settings, confirmation or tiles.
- [Actions](app/session/actions.omg) executes launch/copy/close intents and
  interprets their replies. [Management](app/management.omg) confirms force-stop
  against the exact observed ticket; replacement cancels the confirmation.
- [Settings](app/settings/settings.omg) owns the draft. Its
  [persistence](app/settings/persistence.omg) resolves `Matrix`, reads a pinned
  revision, reconciles uncertain writes and applies saved geometry separately.
  An intervening edit is refused; a lost acknowledgement is read back before
  another publish. Owner `LOSS` reports dirty or uncertain settings.
- [Rendering](app/rendering/rendering.omg) sequences retained surfaces and
  complete images. [Images](app/rendering/images.omg) stages bounded chunks;
  [pixels](app/rendering/pixels.omg) dispatches tile/panel rasterization. Epoch
  changes rebuild the scene without discarding application state.

The [policy map](policy/README.md) leads to the same responsibility split as
Rust's `chooser/policy/` and `chooser/session/`: catalogue, shared geometry,
input selection, action intent and reply outcomes. Omega app callers pass named
state and results directly. No 128-word local policy packet lives in the app.
Actual host/Matrix/storage wire data is still decoded at the session boundary.

## Native boundary and ownership

[bridge/app/main.omg](bridge/app/main.omg) selects
[bridge/component/main.omg](bridge/component/main.omg) as one independent
component. Only that bridge declares `ChooserApp` and translates the legacy
host packet/opcodes through [packets.omg](bridge/component/packets.omg). The [packager](../../../../tools/omega-component/README.md)
checks its relocation-free export; [omega-app](../../../../source-rs/platform/libraries/omega-app/README.md)
supplies scoped IPC, attachment custody and the existing task's monotonic clock.
It knows neither the chooser's state layout nor its configuration schema.

Incoming `ENTRIES` returns at most 51 80-byte rows. The host checks the declared
extent, canonical metadata and ascending cursor, copies only those bytes and
releases the reader before resuming Omega. The chooser gathers at most 51
executable candidates while continuing to resolve named roles. `HEAD`/`READ`
are inline; `COPY_TREE` carries a name; settings `PUBLISH` carries 104 bytes.
Images use 4,080-byte sequential chunks into one object of at most 64 KiB. No raw handles,
PIDs, global locators or device access are passed to the component.

The retained Rust host still reserves nine private pages for its legacy state
and packet ABI. The smaller app workspace does not change that host grant.
Existing chooser grants suffice, including the settings panels. Kernel,
supervision, storage authority and matrix resource grants are unchanged.

`rust-probes` explicitly selects the old Rust session for the recovery and
startup-fault fixtures. These features still use the shared Omega policy via
[bridge/native/main.omg](bridge/native/main.omg); ordinary F5 has no fallback to that session.
A standalone process prototype is isolated on `work/chooser-native`, outside
the F5 composition. Its qualified-compiler check currently times out; this is
not a completed process migration. See [the handoff](../../../../TASKS_RUST.md).

The [bridge policy entrance](bridge/policy/main.omg) alone decodes the temporary
128-word Rust probe packet. Its typed calls enter the same policy the app uses.
That packet is neither an IPC protocol nor an app-facing distribution API.

## Verification

`cargo test --locked -p cathedral-chooser --lib` runs Windows tests against both
actual compiled exports. Session peers cover startup/roles, paged listings,
user copy and launch, settings loss and save reconciliation, intervening edits,
task selection before close, replacement during confirmation and image epoch
races. The host has separate attachment extent/canonical-row tests.

QEMU acceptance includes normal F5 login, app instances and taskbar close,
settings and lost-save recovery, cold persistence and four recursive desktops.
Exact evidence and outstanding work live in [the handoff](../../../../TASKS_RUST.md).
