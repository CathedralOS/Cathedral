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

This packaging route permits exactly one local component, with no dynamic
imports, remaining relocations, TLS or mutable globals. It is not a generic ELF
loader or a proof that native code is trustworthy. The compiled code runs in an
ordinary sandboxed process. A tiny Rust entry and the common platform host remain
until Omega can replace the executable startup and syscall bindings directly.
