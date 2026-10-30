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
python tools/omega-user/background_test.py --compiler path/to/new/omega.exe
python tools/omega-user/colors_test.py --compiler path/to/new/omega.exe
python -m unittest discover -s tools/omega-user -p "test_*.py"
```

The standalone producer is source
`a88a8b63956f99da3f6a1338d6e9d9bf2badc61d`, release opt-level 3, SHA-256
`91a48585ca9626deeb784235c8ed1fe9f5367167016856354d1ec652632480e1`.
It is retained in Omega's `work/cathedral-background-runtime` history and
[PR #846](https://github.com/CathedralOS/Omega/pull/846), published as
`7516f0970de014e3c13009bf67d9c69f087d4671`. Reproduce the tested executable with
`cargo build --locked --release -p omega` in an isolated worktree at that exact
source, following Omega's build-slot instructions. The local
`build/omega-user/compiler/omega.exe` is the saved tested executable with its
source/hash receipt beside it. It passes standalone Background's owner-control
and desktop/cold-reboot checks, runtime connection and primitive witnesses, plus
the focused compiler regressions. Nested and
startup-failure evidence from the earlier `8ae60290` producer is recorded separately
in TASKS_RUST.md; that executable remains at `compiler-background-baseline/`.
Do not substitute a newer compiler without affected qualification.
The preceding `66f71d0e` owner-control producer remains locally preserved at
`build/omega-user/compiler-owner-baseline/`; its history is in TASKS_RUST.md.
Keep this compiler separate from the
[legacy desktop app producer](../omega-component/README.md) and the migrated
[UI library producer](../omega-ui/README.md#build). Desktop app source still
contains `state`; that broader source migration remains unfinished.
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
reply acceptance/read.
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
image-page allowance. The ordinary F5 profile selects Background's and Colors'
`distribution/applications/{background,colors}/native/main.omg` entries. The mixed-producer
harness uses `OMEGA_USER_BIN`, or the saved standalone compiler above, for that
entry. Rust entries retain their normal Cargo build.

An optional `process.json` beside the native consumer declares `imports`
(`contract`, `minimum`, `maximum`, `required`), `channels` and `descendants`.
[manifest.py](manifest.py) encodes the platform's existing funded import record;
the linker places it first in the read-only load, and packaging verifies that
position. This requests launch authority from the owner; it grants none itself.
Both installed standalone apps supply such a record.

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
