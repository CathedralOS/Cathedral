# Private notes

An ordinary application of the matrix and object storage APIs. Press **F5**,
click **Notes** in Programs, then click the workspace to focus it. The equivalent
default boot is:

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

Below the text, a bar shows the notes in this home: a dim segment per note and a
bright one on the current note. A home holds up to eight notes.

Each note is one private object of up to 240 ASCII characters, placed under the
home its storage binding reports. The capacity is this editor's choice, not a
storage limit. On the desktop that home is one world; the app cannot tell a
world from a whole root or from a clone of another world, and it never sees the
worlds beside its own. A duplicate is the store's copy operation: a new object
number beside the original pointing at the same content, written without reading,
hashing or rewriting the text. Editing either note afterwards leaves the other as
it was. Shift, layout selection, selection, clipboard and IME are not
implemented; the diagnostic font is uppercase-shaped.

## Follow execution

[main.rs](main.rs) enters [session/session.rs](session/session.rs): load supplied
connections, read confirmed state, process local input, save/reconcile, present.
[document.rs](document.rs) owns bounded edits and the `NTS1` encoding.
[storage.rs](session/storage.rs) lists the home, reads the current note through a
sealed bulk reply, publishes the whole encoded document as one attachment, and
asks the store to copy. It never silently retries an uncertain commit. A lost
transport remains retired; app replacement is required because replies lack
correlation identities. The owner may retire the app's control link; the app
keeps serving its world without it.

Storage IPC enters the parent's
[storage coordinator](../../../platform/services/matrix/service/parent/storage/storage.rs),
then the [object provider](../../../platform/services/storage/service.rs).
The [distribution supervisor](../../desktop/session/persistence/persistence.rs)
owns provider replacement and explicit re-grants. The app owns only its local
binding and optimistic revision. Another app's replacement grant does not revive
this app's cached binding: the first refused save preserves its draft, then F1
reconciles the new binding.

[view.rs](session/view.rs) publishes one sealed 180x64 text bitmap through the
existing surface chain and draws the note bar as retained rectangles. The app
holds 32 private pages: the retained bitmap, its replacement, and one page per
publication. Its four reader pages receive bulk replies. Matrix and display reader
budgets are explicitly funded by the profile; every hop holds the old and new
bitmap during replacement. No kernel, device or storage policy is added for this app.

## Proofs

```powershell
python tools/boot-harness-rs/desktop_test.py
python tools/boot-harness-rs/matrix_database_test.py --release --notes
python tools/boot-harness-rs/matrix_database_test.py --release --notes-recovery
```

The desktop test types into the default F5 composition, saves, duplicates, edits
the copy, switches back, clones the whole world and edits the clone's note,
returns to the original, cold-boots the same disposable disk and checks every
glyph, status color and bar segment in both worlds. An independent host model
then verifies the retained records on a copy of the image: two worlds of two notes
each, shared manifests behind the duplicated and cloned text, exact heads, edges,
history and quota, and nothing in the ungranted root. The notes fixtures drive two
instances through physical key events at several lengths, middle edits, draft
discard and focus isolation, then verify exact text and status after a cold boot
with the same kind of oracle. The recovery profile stalls after the first commit
but before its reply; ordinary supervision restores grants and F1 reconciles
without an extra revision while the sibling keeps its own unsaved draft through
binding replacement.

The default desktop disk is `build/boot-harness-rs/cathedral-storage.raw`, created
only when absent and never auto-reformatted. The notes fixture retains its own
`notes-storage.raw`. QEMU termination does not emulate physical device-cache loss.
