# Pure x86 register evidence

See [the port record](../../../source/drivers/facts/x86_registers.PORT.md) for
scope, upstream source mapping, limitations and authority boundaries.

```sh
python3 tools/ports/x86_64-registers/check.py --omega /tmp/cathedral-omega-eaa7993/release/omega
```

Use `--host-only` to run pin/transcription audits, actual Rust observations,
UEFI-x64 constant assertions and Rust tests without Omega. It still requires the
Rust target `x86_64-unknown-uefi`. The default compiler path is the sibling Omega
release binary; provide `--omega` to select an exact build.

`generate.py` reconstructs all facts, schema and upstream observations from the
pin. `generate_inventory.py` covers every anchor in the eight selected files and
31 implicit enum variants. `generate_reference_tests.py` extracts the actual
pinned STAR write/validation body, substituting only a recording raw-write leaf.
`generate_fixtures.py` emits 177 fact comparisons, 64 DR7 field combinations and
the standalone `negative.omg`. All four support read-only `--check`; regeneration
without it is an intentional source update requiring review.

`measure.py` executes 177 actual pinned Rust observations, asserts 176
const-compatible facts for UEFI x64 and compares committed vectors. `--write`
updates vectors after review. This observes values, not Omega aggregate layout.
`main.omg` evaluates real translated bodies in a constant expression. The runner
checks three body-mutated copies that must evaluate to one and fail the unchanged
final zero contract. No test invokes a privileged instruction.

The narrow qualified-copy diagnostic can be reproduced independently:

```sh
/tmp/cathedral-omega-eaa7993/release/omega --check tools/ports/x86_64-registers/copy_probe.omg
/tmp/cathedral-omega-eaa7993/release/omega --check tools/ports/x86_64-registers/copy_direct_probe.omg
```

At Omega eaa7993, the first rejects reuse of an imported `[copy]` carrier as
affine; the direct-import form passes. The canonical fixture uses direct imports.
