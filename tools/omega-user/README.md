# Building an Omega process

[build.py](build.py) packages an independently compiled Omega implementation of
`runtime::entry::Program`, assembles the platform-owned native entry and kernel
call leaves, and links an ELF accepted by the reference kernel. It does not
translate Omega, generate app logic, link Rust, or import a Linux runtime.
Use the standalone-runtime compiler below, plus LLVM `clang` and `ld.lld` on PATH.
The desktop's compiler and the other agent's checkout need not move.

```powershell
python tools/omega-user/build.py source/distribution/lab/user-entry/native --compiler path/to/new/omega.exe
python tools/omega-user/run_test.py --compiler path/to/new/omega.exe
python tools/omega-user/connection_test.py --compiler path/to/new/omega.exe
python tools/omega-user/connection_test.py --service --compiler path/to/new/omega.exe
python tools/omega-user/connection_test.py --admission --compiler path/to/new/omega.exe
python tools/omega-user/background_test.py --compiler path/to/new/omega.exe
python tools/omega-user/colors_test.py --compiler path/to/new/omega.exe
python tools/omega-user/notes_test.py --compiler path/to/new/omega.exe
python tools/omega-user/chooser_test.py --compiler path/to/new/omega.exe
python -m unittest discover -s tools/omega-user -p "test_*.py"
```

The standalone producer is source
`92aec21e421bb10fe28012c521cf34b3813088e5`, release opt-level 3, SHA-256
`8ec9d529695afcce984dc8228dd561f019937fa8ae1843385f0c6baf2f766b35`.
Build `cargo build --locked --release -p omega` in an isolated worktree at that
exact source, following Omega's build-slot instructions. The local
`build/omega-user/compiler/omega.exe` is the saved executable with its
source/hash/build receipt beside it. Qualification and artifact paths are
recorded in TASKS_RUST.md. This compiler includes upstream checking fixes that
allow the chooser to compile within the existing bound, without changing its
font/raster implementation or manually restating app contracts.

The preceding `a88a8b63956f99da3f6a1338d6e9d9bf2badc61d` producer is preserved
locally at `build/omega-user/compiler-chooser-baseline/`. Earlier runtime and
compiler evidence remains in TASKS_RUST.md. Do not substitute a newer compiler
without affected qualification.
Keep this compiler separate from the
[legacy desktop app producer](../omega-component/README.md) and the migrated
[UI library producer](../omega-ui/README.md#build). Some retained component source still contains `state`; that broader migration
remains unfinished.
`--compiler` explicitly chooses the producer for these standalone runs.
Compilation is capped at 4 GiB, five minutes and one checking worker by default.
Component and UI builds share one compiler slot across Cargo processes.

The compiler may request initial review of the runtime's native boundaries.
Inspect the review path printed in `build/omega-component/distribution/lab/user-entry/native/review.log`
and the declarations in `source/platform/libraries/user-runtime/kernel.omg` and
`arch/x86_64/gate.omg` against `entry.S`. The table includes process/IPC/clock calls plus shared-memory allocation,
release, typed read-view transfer, bounded shared word access and notification
creation, handoff, signal, wait, control and drop, plus lookup of installed owner
endpoints, typed endpoint acceptance, bounded immutable object creation/send, and sealed
reply acceptance/read, private chunk copying and sealing/unmapping, a
page-contained wait for up to 48 interests, child launch indexing, spawn with a
staged 176-byte startup record, owner-control connection, cancellation,
collection of the 40-byte exit record, endpoint transfer, and own-account
resource and notification budget snapshots.
The native leaves must preserve the declared buffers,
range bounds, blocking and calling convention. Accept the inspected rows in the
generated review document and resume with the same qualified compiler:

```powershell
python tools/omega-component/process.py 4294967296 path/to/new/omega.exe update --project source/distribution/lab/user-entry/native --offline --resume
```

Local locks contain checkout-specific identities and are ignored. The builder
does not automatically accept newly requested native authority. No owner-level
OS design decision is implied by this compiler review.

The component compiler and native linker have separate receipts. The linked
receipt embeds the component receipt and hashes the entry adapter, linker
layout, component and builder. Publication rejects input changes during linking.
There is no dynamic loader, runtime relocation, writable load or additional
image-page allowance. The ordinary F5 profile selects the chooser's, Background's, Colors' and Notes'
`distribution/applications/{chooser,background,colors,notes}/native/main.omg` entries. The mixed-producer
harness uses `OMEGA_USER_BIN`, or the saved standalone compiler above, for that
entry. Rust entries retain their normal Cargo build.

An optional `process.json` beside the native consumer declares `imports`
(`contract`, `minimum`, `maximum`, `required`), `channels` and `descendants`.
[manifest.py](manifest.py) encodes the platform's existing funded import record;
the linker places it first in the read-only load, and packaging verifies that
position. This requests launch authority from the owner; it grants none itself.
All four installed standalone apps supply such a record.

The current compiler rejects a raw `int` inside checked user assembly as a
hidden control transfer. The native leaves use Omega's existing declared
`TableFunction` boundary instead; this preserves explicit external-contract
admission and does not teach the compiler to treat arbitrary interrupts as safe.
A native Cathedral target/entry provider can eventually replace this adapter.

`colors_test.py` runs the actual standalone Colors process against a controlled
Rust service. It holds a save while requiring owner HEALTH/LOSS responses,
rejects that save and checks confirmed pixels and loss reporting, validates
checker/close image backing, and repeats with the owner connection retired.
It also checks that idle Colors parks, one real notification resumes it, and
a queued batch longer than the app drain bound needs no second signal.
Cancellation must refund channels, pages and notifications in both cases.
The ordinary desktop test additionally checks successful saves and cold reopen;
this focused test alone is not desktop acceptance.

`notes_test.py` runs standalone Notes against a controlled service. A keystroke
arriving with first placement must survive the sealed-document load. One wake
must drain more than eight queued inputs, and idle Notes must remain parked.
The provider holds a save while owner HEALTH/LOSS progresses, rejects it, then
checks that the draft and error survive. A full-size case also checks all four incoming read chunks and every outgoing
record byte. A second case retires owner control
while the independent storage connection remains usable. Cancellation with a
published image must refund pages, channels and notifications. The fixture
funds twenty private pages and four reader pages for Notes: the text image needs
twelve private pages, and three shared mailboxes plus a document need four reader
pages. These fixture grants do not alter kernel limits or default matrix grants.

`chooser_test.py` runs the actual login/taskbar process against a controlled
provider. It checks 51-row directory pages and continuation, rejects malformed
metadata/order/count, answers owner HEALTH/LOSS during a held listing, validates
image bytes/padding, and observes refresh without another notification. Cancelling
the process refunds its pages, channels and notifications. Full desktop and
settings/recovery tests cover user copy, app launch, saved configuration and
nested login against the ordinary services.

`connection_test.py --service` runs the standalone multi-client service fixture.
It checks asynchronous setup, sealed request/reply custody, independent owner
progress, malformed handoffs and cleanup under a tight reader allowance. See the
[service witness](../../source/distribution/lab/user-entry/README.md#independent-service-clients)
for the exact scenarios. `connection_test.py --admission` runs the standalone
supervisor that admits Rust peers from lent launches, offers them interaction
and storage, and reclaims them; see the
[admission witness](../../source/distribution/lab/user-entry/README.md#child-admission).
Together these qualify the reusable transport and child lifecycle that the
[independent Omega matrix](../../source/distribution/lab/matrix-composition/README.md)
now runs on; `tools/boot-harness-rs/matrix_composition_test.py --omega` and
`--upstream` boot it. The installed stock matrix still uses its Rust resource host.
