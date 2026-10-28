# Packaging native Omega applications

[build.py](build.py) resolves the checked-in package closure, compiles its native
consumer, and packages one pure export for the reference OS. [elf.py](elf.py)
validates its restricted ELF container and preserves relative code/data spacing.
The same validator is used by the [UI builder](../omega-ui/build.py).

Use the supported optimized compiler described in the
[UI build guide](../omega-ui/README.md#build). Selection is `--compiler`, then
`OMEGA_BIN`, then the prepared `build/omega-ui/compiler` executable, then PATH.

```powershell
python tools/omega-component/build.py source/distribution/applications/colors/bridge/native
cd source-rs
cargo test --locked -p cathedral-colors --lib
cd ..
python -m unittest discover -s tools/omega-component -p "test_*.py"
python tools/boot-harness-rs/desktop_test.py --trace-storage
python tools/boot-harness-rs/desktop_test.py --nested-desktop --nested-depth 4 --trace-storage
```

Cargo invokes this through the apps' build scripts, including Background's
`bridge/native` consumer; F5 needs no extra preparation
after the compiler has been built. Output is under `build/omega-component/`,
with compiler/source fingerprints, ELF hash, entry offset and packaged byte
hashes in `receipt.json`. Cache hits verify every packaged artifact. Concurrent
Cargo/editor/direct builds serialize on the output lock. Source changes during
compilation refuse publication. Generated binaries and Omega package-manager
workspaces are not checked in.

Compiler review and compilation run through [process.py](process.py), with a
4 GiB memory allowance and a five-minute deadline. On Windows a kernel job
limits the whole compiler process tree and closes it if the wrapper exits.
The wrapper joins the job before starting the compiler; other sessions remain
outside it. On Unix the inherited address-space limit applies per process.
All component and UI build invocations also acquire one shared compiler slot,
so parallel Cargo app builds cannot multiply the compiler memory allowance.
Waiting for that slot happens before the compilation deadline starts.
These limits make runaway compilation fail locally instead of exhausting the
host. They do not turn an incomplete compile into a successful package.
The default is one checking worker, which kept the current full client review
around 650 MiB. An explicit `RAYON_NUM_THREADS` setting is retained, while the
memory allowance still applies. Concurrent checking previously exhausted the
host during this work; higher worker counts need measured qualification.

This packaging route permits exactly one local component, with no dynamic
imports, remaining relocations, TLS or mutable globals. It is not a generic ELF
loader or a proof that native code is trustworthy. The compiled code runs in an
ordinary sandboxed process. A tiny Rust entry and the common platform host remain
until Omega can replace the executable startup and syscall bindings directly.

On macOS the kernel address-space ceiling preserves the launcher’s measured
existing VM reservation, then allows the declared allocation budget above it.
Large shared-image/allocator reservations are not resident compiler memory.
The ceiling remains kernel-enforced and inherited by child processes; this is
an address-space allowance, not an aggregate resident-set guarantee. An inherited
stricter hard ceiling is retained.
