# Chooser: Omega login picker and desktop taskbar

The installed chooser is a standalone Omega process. Its
[native entry](native/main.omg) selects [process/main.omg](process/main.omg),
which acquires supplied interaction/storage connections, runs the app and cleans
up. Its manifest requests required imports; it grants no authority. The startup
argument selects picker/taskbar presentation and names the launcher's own scoped
object.

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

The [process reading path](process/README.md) separates application dispatch,
service exchange, storage decoding, image custody and waiting. It uses the common
[Omega user runtime](../../../platform/libraries/user-runtime/README.md) for
scoped IPC, kernel objects, notifications and clock access. The app receives
plain observations and emits named commands; transport handles stay in the
process adapter. Owner HEALTH/LOSS continues while a service reply is pending.

Incoming `ENTRIES` returns at most 51 80-byte rows. The process adapter checks the declared
extent, canonical metadata and ascending cursor, copies only those bytes and
releases the reader before resuming the app. The chooser gathers at most 51
executable candidates while continuing to resolve named roles. `HEAD`/`READ`
are inline; `COPY_TREE` carries a name; settings `PUBLISH` carries 104 bytes.
Images use 4,080-byte sequential chunks into one object of at most 64 KiB. No raw handles,
PIDs, global locators or device access are passed to the app.

Existing chooser grants suffice, including the settings panels. Kernel,
supervision, storage authority and matrix resource grants are unchanged.

The retained [bridge/app/main.omg](bridge/app/main.omg) and
[bridge/component/main.omg](bridge/component/main.omg) support host model tests
and explicit Rust fixtures. Only that bridge declares `ChooserApp` and translates
the legacy packet/opcodes; its Rust host still reserves nine private pages.
`rust-probes` selects the old Rust session for recovery/startup-fault fixtures,
using shared Omega policy through [bridge/native/main.omg](bridge/native/main.omg).
Ordinary F5 does not select either Rust session.

The [bridge policy entrance](bridge/policy/main.omg) alone decodes the temporary
128-word Rust probe packet. Its typed calls enter the same policy the app uses.
That packet is neither an IPC protocol nor an app-facing distribution API.

## Verification

`cargo test --locked -p cathedral-chooser --lib` runs Windows tests against both
actual compiled exports. Session peers cover startup/roles, paged listings,
user copy and launch, settings loss and save reconciliation, intervening edits,
task selection before close, replacement during confirmation and image epoch
races. The host has separate attachment extent/canonical-row tests.

`tools/omega-user/chooser_test.py` checks the actual executable against held and
malformed peers, images, refresh and custody refunds. QEMU acceptance includes normal F5 login, app instances and taskbar close,
settings and lost-save recovery, cold persistence and four recursive desktops.
Exact evidence and outstanding work live in [the handoff](../../../../TASKS_RUST.md).
