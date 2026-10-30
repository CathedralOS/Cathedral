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

The installed executable starts at [native/main.omg](native/main.omg), which
selects the [process entrance](process/main.omg). That entrance establishes
installed authority, runs the editor, and releases local custody. Descend into:

- [application/application.omg](process/application/application.omg): one editor
  state and reusable command/result workspace. [operations.omg](process/application/operations.omg)
  dispatches real editor requests to their owners. Borrowing this workspace avoids
  retaining large sum-type return buffers at every process dispatch step in the
  current compiler; the process still uses the kernel's unchanged 32 KiB stack.
- [services/services.omg](process/services/services.omg): obtain the supplied
  interaction and optional storage endpoints. [exchange.omg](process/services/exchange.omg)
  publishes once and multiplexes owner control with the selected service wait;
  [control.omg](process/services/control.omg) answers HEALTH/LOSS from editor state.
  [waiting.omg](process/services/waiting.omg) parks until actual input or owner
  work, retaining MORE across bounded input batches.
- [storage/requests.omg](process/storage/requests.omg): bind the scoped store and
  encode requests. [records.omg](process/storage/records.omg) reads immutable
  document attachments, checks transport extents, and releases reader custody.
  Schema/revision validation stays in the editor's storage and document modules.
- [presentation/presentation.omg](process/presentation/presentation.omg): encode
  geometry and presentation requests. [images.omg](process/presentation/images.omg)
  constructs the 46,080-byte text image in private pages with at most 8 KiB of
  pixel scratch per chunk, then seals and unmaps it. No pixel array spans an IPC
  wait. Accepted compositor readers have independent lifetimes.

No Rust application host participates in this installed process. The kernel,
matrix and service providers remain the Rust reference. The native adapter is
currently x86-64 only; no kernel limits, storage schema or compiler source changed
for this app migration. The editor's nested declarations now use `machine`.

The [component bridge](bridge/component/main.omg) and
[Rust executable](../../../../source-rs/distribution/applications/notes/main.rs)
remain explicit comparison fixtures. The bridge passes the same editor model
through the shared Rust app host. Feature-selected fault, admission, producer
and render probes instead enable `rust-probes`; those mechanism tests do not
validate the standalone Omega process. Harness fixture setup selects the Rust
entry explicitly. Ordinary builds never silently substitute it.

Run `python tools/omega-user/notes_test.py --compiler build/omega-user/compiler/omega.exe`
for actual-process storage, image, owner-control, idle/wake and resource-refund
checks. Run `cargo test --locked -p cathedral-notes` from `source-rs/` for the
retained compiled editor tests. The desktop harness additionally checks exact
pixels, independent drafts, successful saves, duplication, close protection and
cold reopen. See [the current handoff](../../../../TASKS_RUST.md) for qualification
scope and remaining work.
