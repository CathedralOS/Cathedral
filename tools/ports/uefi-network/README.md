# UEFI network raw and pure-behavior fixtures

The complete scope, deviations, source pin, license and limits are in
[`network.PORT.md`](../../../source/contracts/uefi/raw/network.PORT.md).
These developer fixtures never join a production boot import graph.

From the Cathedral root:

```sh
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/uefi-rs source/contracts/uefi/raw/network-inventory.json
python3 tools/ports/uefi-network/measure.py
python3 tools/ports/uefi-network/check_transcription.py
python3 tools/ports/uefi-network/check.py --omega ../Omega/target/release/omega
../Omega/target/release/omega --check tools/ports/uefi-network/layouts.omg
```

The inventory checks all 720 lexical anchors. Seven remain blocked: five
runtime tails and two borrowed typed packet projections. Formatting and module
namespace omissions each have explicit reasons. `--require-transcribed` therefore
fails intentionally for the whole claimed slice.

`measure.py` compiles pinned upstream Rust, measures 703 facts on the host, and
then compiles an assertion for every numeric value/GUID byte under
`x86_64-unknown-uefi`. That target must be installed in the current Rust toolchain.
`src/lib.rs` is generated from the observed values on every run. Cargo's locked
local-path dependency ensures the correct source; the probe checks the git pin
before compilation. Measurements are upstream evidence, not Omega output.

`check_transcription.py` checks every raw field/order/type, retained union/tail
mapping, scalar/GUID constant, C-profile geometry and authored plan against the
independent vectors. It does not infer Omega's native layout.

`check.py` executes the actual Omega helper/test bodies through constant
semantic evaluation. The positive fixture requires the evaluated result to be
zero; the negative control changes that requirement to result plus one and must
be rejected. It prints the compiler artifact hash. The fixtures have no std,
Console, firmware callback or native-run dependency. No native execution or
emitted ABI compatibility is claimed.

Expected compiler blocker fixtures (not passing tests):

```sh
../Omega/target/release/omega --check tools/ports/uefi-network/layout_capacity.omg
../Omega/target/release/omega --check tools/ports/uefi-network/layout_projection.omg
../Omega/target/release/omega --check tools/ports/uefi-network/layout_union_view.omg
```

They expose the complete 34-field mode exceeding the 32-field schema, and
public fields becoming private on synthesized policy values. The union-view
fixture attempts a checked shared byte-prefix recast; the field-access error is
the first observed failure, so later recast validation remains unobserved.
`network_large_layouts.omg` retains the complete plan outside passing imports.
Do not silently truncate its two final fields to make compilation succeed.

Regeneration after review of the exact existing pin:

```sh
python3 tools/ports/uefi-network/generate.py
python3 tools/ports/uefi-network/measure.py --write
python3 tools/ports/uefi-network/generate_layouts.py
python3 tools/ports/uefi-network/map_inventory.py
python3 tools/ports/uefi-network/check_transcription.py
```

`generate.py` is a restricted lexical extractor for this pin, not a general
Rust-to-Omega translator. `map_inventory.py` applies reviewed operation mappings
by exact source line to an existing snapshot. A changed pin requires a fresh
inventory snapshot and human review of these mappings, generated source,
fixtures, licenses and expectations. Do not use `--write` to accept a mismatch
without reviewing it. Pure helper code and semantic test bodies are hand-authored.
