# HII raw representation verification

See [hii.PORT.md](../../../source/contracts/uefi/raw/hii.PORT.md). From the
Cathedral root:

```sh
python3 tools/ports/inventory.py check source/contracts/uefi/raw/hii-inventory.json --checkout reference_code/rust-osdev/uefi-rs
python3 tools/ports/uefi-hii/measure.py
python3 tools/ports/uefi-hii/check_transcription.py
python3 tools/ports/vectors.py source/contracts/uefi/raw/hii.vectors.json
cargo test --locked --manifest-path tools/ports/uefi-hii/Cargo.toml
python3 tools/ports/uefi-hii/check_omega.py
```

The exact optional upstream checkout is required by the inventory/measurement
steps. Cargo dependencies are locked; `x86_64-unknown-uefi` must be installed.
`measure.py` first measures pinned native Rust types/values, then asserts every
numeric fact and GUID byte for UEFI x64. It checks complete assertion coverage.
`--write` refreshes vectors only after those checks pass.

`check_transcription.py` independently compares source field order/types,
constants and explicit fixed layout plans to target-checked facts. Union
WireStorage types only retain opaque bytes/alignment; they do not acquire typed
union member semantics. Tail offsets remain independent from fixed prefix size.
These checks are not an Omega ABI inspection.

`check_omega.py` defaults to `../Omega/target/release/omega`; an explicit binary
path can be passed. It source-checks each raw module, fixed policy module and
pure helper module, then evaluates all helper cases through the fixture's
compile-time result and requires success. The negative control must fail on
`0 + 1 == 0`. This exercises actual Omega helper bodies without native or firmware
execution. Do not change the package sources while an invocation snapshots them.

`layout_projection.omg` and `layout_probe.omg` exercise selected-layout consumer
boundaries separately. Their outcomes do not affect the positive helper fixture
or upgrade opaque union storage into typed unions.

`generate.py` is a restricted translator for the pinned nine HII files. It writes
the Rust probe first. On a fresh checkout, run it, run `measure.py --write`, then
run it again to write source/layout/inventory artifacts using measured geometry.
Review all generated changes and run all checks. It does not update the upstream
pin or make decisions about union semantics.

The upstream test covers every byte input to the four original selector methods.
Local Omega tests cover flags, unknown/reserved selector bits, package and opcode
length/scope extraction, minimum/maximum lengths and truncated storage bounds.
No code here interprets an entire form, invokes a HII protocol or changes firmware
configuration.
