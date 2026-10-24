# Private notes

The installed editor now lives in [Omega Notes](../../../../source/distribution/applications/notes/README.md).
[main.rs](main.rs) declares the existing scoped imports and invokes the shared
[platform host](../../../platform/libraries/omega-app/README.md). Omega owns
editor behavior; Rust supplies runtime transport and buffer custody.
The Rust session below remains only for explicitly selected `rust-probes`
fault/admission/producer/render fixtures.

 Press **F5**,
choose a user, click **Notes** in its chrome, then click the note to focus it.
The equivalent default boot is:

```powershell
python tools/boot-harness-rs/run.py --release --window
```

The [notes fixture](../../../../tools/boot-harness-rs/profiles/notes.json) also runs
two instances with separate private roots, one inside a nested matrix, for
isolation and recovery tests. Neither knows its position, siblings, disk provider
or the root owner's diagnostics.

Type letters, digits, spaces or Enter for a newline; Left/Right move the insertion
cursor and Backspace deletes. **F1** saves. **F3** duplicates the saved note and
switches to the copy; **F4** switches to the next note. Green means the text
matches confirmed storage, amber an unsaved draft, red that storage needs
reconciliation. On red, F1 reads current storage while keeping the draft; a later
F1 deliberately saves any remaining difference. **F2** reloads and discards the
draft only after a successful read. A failed read never erases it. Duplicate and
switch act only on a clean, saved note.

**F6** demonstrates scoped cursor preference: toggle hiding for pointers already
received in that keyboard's local input group. Input remains active. Leaving the
note restores the next target's policy; returning can reuse the live note's
preference. Cancellation or view replacement resets it. The
[cursor client](../../../../source/distribution/applications/notes/app/session/cursors.omg) uses only supplied local aliases, including
when Notes runs beneath nested matrices.

Below the text, a bar shows the notes in this realm: a dim segment per note and a
bright one on the current note. A realm holds up to eight notes.

The **X** in the upper-right corner belongs to Notes. A clean document closes
immediately. Unsaved work opens **SAVE / DISCARD / CANCEL** in the editor area;
typing is suspended until that choice is resolved. Cancel restores the draft.
Save closes only after success (or reconciliation confirms the exact draft);
a failed save keeps the dialog and draft. Discard explicitly exits without
publishing the draft. Normal process exit lets the matrix release surfaces,
input and the launch slot; no sibling-close request is needed.

If the owner withholds optional persistence, Notes starts empty with a purple
base and **TEMP NO SAVE** instead of the note bar. Its X offers Discard/Cancel;
F1/F2 cannot save or discard the draft. The normal F5 launcher grants storage;
`desktop_test.py --service-imports undeclared` tests the absent-service case.
A supplied service failing is different from an absent grant: failed saves
retain the reconciliation path above. After automatic recovery exhaustion,
the launcher's **RETRY SAVES** action permits one new provider attempt.

Each note is one private object of up to 240 ASCII characters, placed in the
realm its storage binding reports. The capacity is this editor's choice, not a
storage limit. The app cannot tell a subtree from a whole root and never sees
what lies beside its realm; on the desktop its realm is its own program object,
so the notes nest under the code that edits them. Notes writes its child documents
and leaves that root object alone. Split instead grants each instance a separate
Left/Right container.
A duplicate is the store's copy operation: a new object number beside the
original pointing at the same content, written without reading, hashing or
rewriting the text. Editing either note afterwards leaves the other as it was.
Shift, layout selection, selection, clipboard and IME are not implemented; the
diagnostic font is uppercase-shaped.

## Follow execution

Start at [Omega main](../../../../source/distribution/applications/notes/app/main.omg),
then its named session, document, storage and rendering entrances. The app receives
only matrix-supplied services and validates document data before replacing a draft.
The host can retire the optional control link while wake/input remain usable;
kernel parent-death teardown still applies to the entire subtree.

The retained [Rust session](session/session.rs), [document](document.rs),
[closing](session/closing.rs), [storage](session/storage.rs) and
[view](session/view.rs) implement the old fixture, not the ordinary installed app.

Storage IPC enters the parent's
[storage coordinator](../../../distribution/matrix/service/parent/storage/storage.rs),
then the [object provider](../../../platform/services/storage/service.rs).
The [distribution supervisor](../../desktop/session/persistence/persistence.rs)
owns provider replacement and explicit re-grants. The app owns only its local
binding and optimistic revision. Another app's replacement grant does not revive
this app's cached binding: the first refused save preserves its draft, then F1
reconciles the new binding.

The Omega renderer publishes the same sealed 180x64 text image and three retained
decorations. The app-host reserves nine private pages for opaque state/packet and
stages the 12-page text object through two chunks, under existing app budgets.
Bulk reads are bounded before mapping and released after copying into the packet.
Matrix/display reader budgets still fund retained images at every hop.
Unchanged background, close and tab indicators stay retained; ordinary editing
replaces only the text bitmap. Detailed `trace-input` editor markers currently
remain with the Rust fixture; native transport still supports `trace-storage`.

## Proofs

```powershell
python tools/boot-harness-rs/desktop_test.py
python tools/boot-harness-rs/matrix_database_test.py --release --notes
python tools/boot-harness-rs/matrix_database_test.py --release --notes-recovery
```

The desktop test types into the default F5 composition, saves, duplicates, edits
the copy, switches back, cold-boots the same disposable disk and checks every
glyph, status color and bar segment. An independent host model then verifies the
retained records on a copy of the image: the four installed programs, two notes
under Notes with one shared manifest for the duplicated text, the color under
Colors, exact heads, edges, history and quota, and nothing in the ungranted root. The notes fixtures drive two instances through physical key
events at several lengths, middle edits, draft discard and focus isolation, then
verify exact text and status after a cold boot with the same kind of oracle. The
recovery profile stalls after the first commit but before its reply; ordinary
supervision restores grants and F1 reconciles without an extra revision while the
sibling keeps its own unsaved draft through binding replacement.

Normal F5 requests a fresh lab disk. A CLI launch without `--fresh` retains
`build/boot-harness-rs/cathedral-storage.raw`. The notes fixture retains its own
`notes-storage.raw`. QEMU termination does not emulate physical device-cache loss.

Live layout changes retire old drawing handles. The session keeps the document,
cursor, save uncertainty and close dialog, and rebuilds only its view. A drawing
refusal triggers a fresh VIEW check: a changed/revoked epoch permits rebuilding;
a refusal under the same live epoch remains an error rather than an endless retry.
Input press tracking resets at rebinding. The chooser and Colors follow the same
rule. Normal Colors setup no longer runs the adversarial surface probes; those
remain in its lab build.

The `render-probes` feature selects the Rust fixture and enables two Notes-only yield gates in
[render_probe.rs](session/render_probe.rs). The settings harness's `--render-race`
mode observes each gate before resizing, proving both partial surface creation
and interrupted bitmap publication recover without reloading the document.
The default distribution never enables these gates.

The view uses four surfaces in either mode: base, text, tabs/status and close.
Tabs are one small bitmap, keeping persistence independent of surface funding.
