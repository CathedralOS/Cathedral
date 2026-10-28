# Building an Omega process

[build.py](build.py) packages an independently compiled Omega implementation of
`runtime::entry::Program`, assembles the platform-owned native entry and kernel
call leaves, and links an ELF accepted by the reference kernel. It does not
translate Omega, generate app logic, link Rust, or import a Linux runtime.
Use the existing [qualified compiler](../omega-ui/README.md#build), plus LLVM
`clang` and `ld.lld` on PATH. The other agent's compiler checkout need not move.

```powershell
python tools/omega-user/build.py source/distribution/lab/user-entry/native
python tools/omega-user/run_test.py
python -m unittest discover -s tools/omega-user -p "test_*.py"
```

The compiler may request initial review of the runtime's native boundaries.
Inspect the review path printed in `build/omega-component/distribution/lab/user-entry/native/review.log`
and the declarations in `source/platform/libraries/user-runtime/kernel.omg` and
`arch/x86_64/gate.omg` against `entry.S`. The ten calls are yield, environment,
channel creation, send, receive, close, diagnostic write, clock acquisition,
clock read and event wait. The native leaves must preserve the declared buffers,
range bounds, blocking and calling convention. Accept the inspected rows in the
generated review document and resume with the same qualified compiler:

```powershell
build/omega-ui/compiler/target/release/omega.exe update --project source/distribution/lab/user-entry/native --offline --resume
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
