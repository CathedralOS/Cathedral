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
python -m unittest discover -s tools/omega-user -p "test_*.py"
```

The v4 connection witness is qualified with source `ccb6d4791d8d4`,
[Omega #525](https://github.com/CathedralOS/Omega/pull/525), release opt-level 3
(published as `a0bc08225d743ab888090f10476817c485820807`).
Its compiler SHA-256 is
`089d89f492f6916c8d5e4fef217e1fca4107679cb7d438425a0733de5fe4cfce`.
The fixes preserve enum-case tail transfers and their local loans through native
component compilation. Keep this compiler separate from F5's
[qualified desktop producer](../omega-ui/README.md#build): the newer compiler
has removed `state`, while that broader source migration remains unfinished.
`--compiler` explicitly chooses the producer for these standalone runs.
Compilation is capped at 4 GiB, five minutes and one checking worker by default.
Component and UI builds share one compiler slot across Cargo processes.

The compiler may request initial review of the runtime's native boundaries.
Inspect the review path printed in `build/omega-component/distribution/lab/user-entry/native/review.log`
and the declarations in `source/platform/libraries/user-runtime/kernel.omg` and
`arch/x86_64/gate.omg` against `entry.S`. The table includes process/IPC/clock calls plus shared-memory allocation,
release, typed read-view transfer, bounded shared word access and notification
creation, handoff, signal, wait, control and drop. The native leaves must preserve the declared buffers,
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
image-page allowance. The ordinary F5 build path is unchanged.

The current compiler rejects a raw `int` inside checked user assembly as a
hidden control transfer. The native leaves use Omega's existing declared
`TableFunction` boundary instead; this preserves explicit external-contract
admission and does not teach the compiler to treat arbitrary interrupts as safe.
A native Cathedral target/entry provider can eventually replace this adapter.
