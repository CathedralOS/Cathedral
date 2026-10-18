# Follow the running distribution

Press **F5** at the repository root, or run:

```powershell
python tools/boot-harness-rs/run.py --release --window
```

One desktop boots to a chooser over the folder of users. **NEW USER** copies the
template user beside it; clicking a user replaces the boot picker with that
user's full-screen workspace. A user is a
matrix with its own chrome: a second chooser over the user's realm, listing the
user's copies of Notes and Colors. Click one to launch it, and click the
program's area to give it keyboard focus. **CLOSE** asks the running program
whether it would lose work and keeps it open if so. The boot chooser's display
and input grant is revoked on entry, leaving the user's launcher on the left
and its app in the remaining area. There is no logout control yet; restart
QEMU to select another user. Notes holds up to eight
notes of 240 characters: **F1** saves (or checks an uncertain save), **F2**
reloads the saved note, **F3** duplicates it as a new note sharing its content,
**F4** switches to the next note. Colors changes on **Enter**. Both choosers
also take Up/Down/Enter while focused. The user launcher's **STORAGE ON/OFF**
row controls future launches: OFF runs Notes as a temporary unsavable editor;
Colors refuses with NEEDS SERVICE because its storage import is required.
Turn storage on and relaunch to return to saved data. Existing copied programs
retain their code; use NEW USER in the picker for a copy of the current template
when trying newly added app features.

Everything persists in `build/boot-harness-rs/cathedral-storage.raw` across
launches and QEMU restarts: the users you made, their notes and colors. The
first boot creates this experimental database image and installs the programs
into it; later boots preserve it and reinstall only programs whose bytes
changed. Earlier lab images remain separate and untouched. Every program's
state nests under its own object, and a user is a subtree copy of the template,
so a new user costs a few records until its notes diverge.

## Executable route

Start at [desktop/main.rs](desktop/main.rs), then the
[session entrance](desktop/session/session.rs) and
[shell policy](desktop/session/shell/shell.rs). Admission starts providers and a
funded root matrix. The desktop runs the chooser in the root's first slot,
shown only the folder of users, and opens the second slot with a standing
launch policy: whatever the chooser launches lands there with that budget and
placement. When a user matrix appears, the desktop furnishes it the same way,
its chrome in its first slot over the whole user realm and a standing policy
for the second. [Phase layout](desktop/session/shell/layout.rs) retires the boot
picker and places the user's chrome/app within the full display. The owner
then supervises storage and restores the realms after a
provider replacement. Every matrix runs the same platform code.

[profile.json](profile.json) is the executable composition shared by F5,
ordinary CLI boot and [the desktop test](../../tools/boot-harness-rs/desktop_test.py).
The harness builds the boot launches and supplies their ELF artifacts to
[kernel startup](../kernel/boot/uefi/startup.rs); it trims the `programs` to
their loadable bytes, appends the optional `display_title` footer, and installs
them into the storage image as objects of
the notes root, in order and nested as the profile says, with `folder` entries
as empty containers. The kernel sees supplied code and grants; the distribution
assigns budgets and placement, and the choosers launch programs from their
realms by object number. The UI shows USER TEMPLATE / USER 1 in the picker and
LAUNCHER / NOTES / COLORS in the user's workspace. See the
[chooser](applications/chooser/README.md) for the bounded display-title format.

| Role | Composition entry | Entrance / receiving implementation |
| --- | --- | --- |
| Desktop owner | Initial | [main](desktop/main.rs), [admission](desktop/session/admission.rs), [shell](desktop/session/shell/shell.rs) |
| Display | Launch 0 | [display main](../platform/services/display/main.rs), [retained compositor](../platform/services/display/compositor/compositor.rs) |
| Input | Launch 1 | [input main](../platform/services/input/main.rs), [matrix source](../platform/services/input/matrix.rs) |
| Root matrix | Launch 2 | [matrix main](../platform/services/matrix/main.rs), [service loop](../platform/services/matrix/service/service.rs) |
| Object storage | Launch 3 | [storage main](../platform/services/storage/main.rs), [object contract](../platform/contracts/storage.rs) |
| Chooser | Program 1 | [chooser](applications/chooser/session/session.rs), shown the folder of users |
| Users | Program 2, a folder | Holds the template and every user copied from it |
| Template user | Program 3, inside Users | Same matrix service; a user matrix hosts its chrome and one app |
| Chrome | Program 4, inside the template | The chooser again, over the user's realm |
| Notes | Program 5, inside the template | [notes main](applications/notes/main.rs), [editing/storage/view](applications/notes/session/session.rs) |
| Colors | Program 6, inside the template, role 2 with argument 2 | [color app](desktop/session/app/app.rs) |

A chooser lists its realm and asks its host to run an entry with `LAUNCH`. The
host reads the object through the chooser's own binding, so only what the
chooser can see can run, starts it as the chooser's sibling in the slot its
owner opened, grants it the entry's subtree, and names the slot in the reply.
`CLOSE` asks the host to stop that sibling; the host first asks the sibling
with `LOSS` what it would lose and stops it only on an answer of nothing. A
chooser holds no other launch authority and writes to its realm only to copy
its first entry on NEW in user-picker mode. In user-launcher mode that row
instead withholds storage from future launches. Apps see local geometry and input, not global
placement, other apps or an ancestor's services. The
[matrix contract](../platform/contracts/matrix.rs) and
[child launches](../platform/services/matrix/service/children/children.rs)
connect this distribution policy to the platform mechanism.

## Engineering proofs

```powershell
python tools/boot-harness-rs/desktop_test.py
```

This builds the default composition, copies a user from the template, launches
its programs with physical mouse and keyboard input, checks exact note glyphs
and status, protects unsaved drafts, duplicates and switches notes, checks
optional storage and required-service refusal, repeatedly
replaces apps, checks the boot picker disappears and cannot receive input,
cold-boots the same disposable disk to check saved state, and
has an independent host model verify the retained records. It never touches
the interactive disk.

Recursive whole-user LOSS and teardown/recreation remain covered by the
headless `--matrix-test` fixture; they are not a logout workflow in this desktop.

Other profiles remain focused mechanism and fault tests, not competing default
OS entry points. [The desktop guide](desktop/README.md) describes its recursive
matrix, storage and provider-recovery proofs. `--smoke` selects the profile's
`user_programs` for exhaustive kernel exercises.

The [implementation map](../STATUS.md) tracks remaining restrictions. In
particular this is one chooser over a folder and one app per user at a time,
not a package installer, general desktop window manager or production filesystem.
