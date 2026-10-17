# Follow the running distribution

Press **F5** at the repository root, or run:

```powershell
python tools/boot-harness-rs/run.py --release --window
```

One desktop boots. Click **Notes** or **Colors** in Programs, then click the
workspace to give the app keyboard focus. Notes holds up to eight notes of 240
characters: **F1** saves (or checks an uncertain save), **F2** reloads the saved
note, **F3** duplicates it as a new note sharing its content, **F4** switches to
the next note. Colors changes on **Enter**. **Close App** closes the workspace.
The workspace lives in a world; **Clone World** copies the current world with
every note in it, by reference, and reopens the program there, and **Next
World** cycles through the worlds. The menu's last line says which world of how
many is open. Opening a different program or world replaces the current one; an
unsaved note stays open until explicitly saved or reloaded, and a busy note is
left open rather than killed. The menu also supports Up/Down/Enter while focused.

Notes persists in `build/boot-harness-rs/cathedral-storage.raw` across app launches
and QEMU restarts. The first boot creates this experimental database image;
subsequent boots preserve it. Earlier lab images remain separate and untouched.
Colors is deliberately transient and receives no storage grant.

## Executable route

Start at [desktop/main.rs](desktop/main.rs), then the
[session entrance](desktop/session/session.rs) and
[shell policy](desktop/session/shell/shell.rs). Admission starts providers and a
funded root matrix. The root contains an isolated launcher and a workspace matrix;
the workspace admits the selected app. Both matrices run the same platform code.
[Worlds](desktop/session/worlds.rs) are top-level objects of the notes root that
the owner lists, creates and clones on the root matrix's own authority; the
workspace is granted one world as its storage scope. The owner supervises
storage and explicitly restores only Notes' storage grant when replacing the
provider.

[profile.json](profile.json) is the executable composition shared by F5, ordinary
CLI boot and [the desktop test](../../tools/boot-harness-rs/desktop_test.py).
The harness builds these packages and supplies their ELF artifacts to
[kernel startup](../kernel/boot/uefi/startup.rs). The kernel sees supplied code and
grants; the distribution assigns application roles and placement.

| Role | Composition entry | Entrance / receiving implementation |
| --- | --- | --- |
| Desktop owner | Initial | [main](desktop/main.rs), [admission](desktop/session/admission.rs), [shell](desktop/session/shell/shell.rs) |
| Display | Launch 0 | [display main](../platform/services/display/main.rs), [retained compositor](../platform/services/display/compositor/compositor.rs) |
| Input | Launch 1 | [input main](../platform/services/input/main.rs), [matrix source](../platform/services/input/matrix.rs) |
| Root matrix | Launch 2 | [matrix main](../platform/services/matrix/main.rs), [service loop](../platform/services/matrix/service/service.rs) |
| Object storage | Launch 3 | [storage main](../platform/services/storage/main.rs), [object contract](../platform/contracts/storage.rs) |
| Workspace matrix | Executable 0 | Same matrix service; child-local extent and attenuated input |
| Notes | Executable 1 | [notes main](applications/notes/main.rs), [editing/storage/view](applications/notes/session/session.rs) |
| Menu / Colors | Executable 2, supplied app argument 3 / 2 | [launcher](desktop/session/shell/launcher.rs), [color app](desktop/session/app/app.rs) |

The menu reports a numbered choice over its private owner control connection.
Only the desktop owner validates that choice and sends RUN/STOP to the matrix.
The menu has no launch handles or storage capability. Notes receives one world
of the private object root as its home; Colors receives none. Apps see local geometry and input, not global
placement, other apps or an ancestor's services. The
[matrix contract](../platform/contracts/matrix.rs) and
[child admission](../platform/services/matrix/service/children/admission.rs)
connect this distribution policy to the platform mechanism.

## Engineering proofs

```powershell
python tools/boot-harness-rs/desktop_test.py
```

This builds the default composition, chooses programs with physical mouse and
keyboard input, checks exact note glyphs and status, protects unsaved drafts,
duplicates and switches notes, repeatedly replaces apps, cold-boots the same
disposable disk to check saved text, and has an independent host model verify the
retained records. It never touches the interactive disk.

Other profiles remain focused mechanism and fault tests, not competing default
OS entry points. [The desktop guide](desktop/README.md) describes its recursive
matrix, storage and provider-recovery proofs. `--smoke` selects the profile's
`user_programs` for exhaustive kernel exercises.

The [implementation map](../STATUS.md) tracks remaining restrictions. In
particular this is a fixed installed menu with one active workspace app, not a
package installer, general desktop window manager or production filesystem.
