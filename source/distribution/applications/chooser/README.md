# Chooser: Omega login picker and desktop taskbar

The installed chooser runs entirely through the common Omega app host. Its
[Rust entry](../../../../source-rs/distribution/applications/chooser/main.rs)
declares required interaction/storage imports and passes its startup argument
to the host. No chooser state, rendering, settings or input policy lives in the
normal Rust executable. The argument selects picker/taskbar presentation and
names the launcher's own scoped object; it grants no authority.

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

The existing [policy entrance](policy/main.omg) remains the single source for
metadata classification, catalogue limits, shared paint/hit geometry, selection,
intent and refusal decisions. The Omega session calls it directly. The policy's
small packet is an internal distribution convention, not an IPC service or app
startup ABI.

## Native boundary and ownership

[session-native/main.omg](session-native/main.omg) selects the complete app as
one independent component. The [packager](../../../../tools/omega-component/README.md)
checks its relocation-free export; [omega-app](../../../../source-rs/platform/libraries/omega-app/README.md)
supplies scoped IPC, attachment custody and the existing task's monotonic clock.
It knows neither the chooser's state layout nor its configuration schema.

Incoming `ENTRIES` returns at most 51 80-byte rows. The host checks the declared
extent, canonical metadata and ascending cursor, copies only those bytes and
releases the reader before resuming Omega. The chooser gathers at most 51
executable candidates while continuing to resolve named roles. `HEAD`/`READ`
are inline; `COPY_TREE` carries a name; settings `PUBLISH` carries 104 bytes.
Images use sequential staging into one object of at most 64 KiB. No raw handles,
PIDs, global locators or device access are passed to the component.

The temporary native ABI reserves nine private pages for state and packet.
Existing chooser grants suffice, including the settings panels. Kernel,
supervision, storage authority and matrix resource grants are unchanged.

`rust-probes` explicitly selects the old Rust session for the recovery and
startup-fault fixtures. These features still use the shared Omega policy via
[native/main.omg](native/main.omg); ordinary F5 has no fallback to that session.
The former app-local Rust policy marshalling is compiled only for probes/tests.

## Verification

`cargo test --locked -p cathedral-chooser --lib` runs Windows tests against both
actual compiled exports. Session peers cover startup/roles, paged listings,
user copy and launch, settings loss and save reconciliation, intervening edits,
task selection before close, replacement during confirmation and image epoch
races. The host has separate attachment extent/canonical-row tests.

QEMU acceptance includes normal F5 login, app instances and taskbar close,
settings and lost-save recovery, cold persistence and four recursive desktops.
Exact evidence and outstanding work live in [the handoff](../../../../TASKS_RUST.md).
