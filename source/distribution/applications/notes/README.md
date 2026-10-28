# Notes in Omega

Normal F5, Split and nested desktops install this Notes component. The existing
editor remains deliberately small: 240 ASCII characters, eight documents per
realm, F1 save/reconcile, F2 reload, F3 duplicate, F4 next, F6 cursor preference
and an app-owned X with Save/Discard/Cancel. Optional storage can be withheld;
temporary notes retain drafts and close protection.

Start at [app/main.omg](app/main.omg): advance completed work, retaining storage
uncertainty and optional service refusal. Then follow:

- [session/session.omg](app/session/session.omg): acquire scoped services,
  receive input, dispatch editor work, render and answer owner control.
  Initial storage binding waits for valid placement, allowing the owner to finish
  assigning the realm; the first placed input is retained across that load.
  [input.omg](app/session/input.omg) handles pointer edges and close choices;
  [keys.omg](app/session/keys.omg) dispatches editor commands;
  [cursors.omg](app/session/cursors.omg) scopes preferences to the keyboard group.
- [document/document.omg](app/document/document.omg): insertion, deletion,
  caret and draft comparison. [encoding.omg](app/document/encoding.omg) owns
  the `NTS1` schema and validates incoming bytes before replacing a document.
- [storage/storage.omg](app/storage/storage.omg): realm enumeration, revision-
  pinned bulk reads, publication, duplication and explicit reconciliation.
  A failed write may have committed; the next F1 reads confirmed state while
  retaining the draft, and another F1 deliberately publishes any difference.
- [rendering/rendering.omg](app/rendering/rendering.omg): retained surfaces,
  frame demand and publication. [pixels.omg](app/rendering/pixels.omg) draws
  four rows around the caret, the close dialog and decorations. Text images are
  staged in two bounded chunks; unchanged decorations remain at the compositor.
- [state.omg](app/state.omg): app-owned state and typed continuation points.
  [operations.omg](app/operations.omg) describes requested work, completed replies,
  input, images and document bytes without native packet offsets.

The [component bridge](bridge/component/main.omg) decodes a completed operation,
advances the app and encodes its next request. `replies.omg` and `commands.omg`
own transport interpretation; `records.omg` transfers bounded bytes while the
document codec owns `NTS1` validation. `images.omg` packs pixels supplied by the
app's renderer, and `diagnostics.omg` encodes the existing startup messages.
Neither the bridge nor the Rust host decides how to edit, save or close Notes.

The [native consumer](bridge/native/main.omg) builds an independent Omega export.
The [Rust executable](../../../../source-rs/distribution/applications/notes/main.rs)
declares the existing imports and enters the shared
[platform host](../../../../source-rs/platform/libraries/omega-app/README.md).
Rust owns runtime transport, waits and attachment custody. Omega owns all normal
editor, persistence, input, rendering and close decisions. No new syscall or
ambient authority is involved. This remains a synchronous x86-64 bridge to the
reference OS, not the final Omega application ABI.

[Compiled export tests](../../../../source-rs/distribution/applications/notes/tests/behavior.rs)
exercise actual Omega code against controlled peers. The desktop harness checks
exact pixels, independent drafts, durable records and cold boot. Feature-selected
fault/admission/producer/render probes explicitly enable `rust-probes`, retaining
the old Rust session for those mechanism fixtures. Such runs do not validate this
Omega editor. The standalone Rust document tests remain an independent schema
oracle. Native startup diagnostics retain the `note-loaded` marker; detailed Rust
editor timing/probe markers have not been ported.

Build with `python tools/omega-component/build.py
source/distribution/applications/notes/bridge/native` or let Cargo prepare the component.
Run `cargo test --locked -p cathedral-notes` from `source-rs/` on Windows to replay
the compiled export; the integrated desktop runs it in QEMU.
