# Follow the running distribution

Press **F5** at the repository root, or run:

```powershell
python tools/boot-harness-rs/run.py --release --window
```

One desktop boots. Click **Notes** or **Colors** in Programs, then click the
workspace to give the app keyboard focus. Notes accepts up to 60 characters;
**F1** saves (or checks an uncertain save), **F2** reloads the saved note. Colors
changes on **Enter**. **Close App** closes the workspace. Opening a different
program replaces the current one; an unsaved note stays open until explicitly
saved or reloaded. The menu also supports Up/Down/Enter while focused.

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
The owner supervises storage and explicitly restores only Notes' storage grant
when replacing the provider.

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
| Database storage | Launch 3 | [database main](../platform/services/storage/database/main.rs), [catalog contract](../platform/contracts/storage.rs) |
| Workspace matrix | Executable 0 | Same matrix service; child-local extent and attenuated input |
| Notes | Executable 1 | [notes main](applications/notes/main.rs), [editing/storage/view](applications/notes/session/session.rs) |
| Menu / Colors | Executable 2, supplied app argument 3 / 2 | [launcher](desktop/session/shell/launcher.rs), [color app](desktop/session/app/app.rs) |

The menu reports a numbered choice over its private owner control connection.
Only the desktop owner validates that choice and sends RUN/STOP to the matrix.
The menu has no launch handles or storage capability. Notes receives a private
catalog; Colors receives none. Apps see local geometry and input, not global
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
repeatedly replaces apps, and cold-boots the same disposable disk to check saved
text. It never touches the interactive disk.

Other profiles remain focused mechanism and fault tests, not competing default
OS entry points. [The desktop guide](desktop/README.md) describes its recursive
matrix, storage and provider-recovery proofs. The old status/counter composition
is preserved as [legacy.json](../../tools/boot-harness-rs/profiles/legacy.json),
with its [init](init/session/session.rs) and
[session protocol](libraries/session-protocol/lib.rs). Old scene capture/recovery
flags deliberately select that fixture. `--smoke` still selects the profile's
`user_programs` for exhaustive kernel exercises.

The [implementation map](../STATUS.md) tracks remaining restrictions. In
particular this is a fixed installed menu with one active workspace app, not a
package installer, general desktop window manager or production filesystem.
