# Pure AML helper validation

This tests the staged [interpreter helper slice](../../../../source/libraries/acpi/interpreter/PORT.md),
not AML bytecode or methods. All storage is initialized ordinary byte arrays;
no handler, I/O, physical mapping or namespace capability is supplied.

**Verified 2026-09-20:** 150 semantic scenarios and three body-mutating
controls pass; all fixture bodies source-check (14 sources). The observed
runs were the initial 141 cases plus six capacity and three final-byte cases,
with the same helper source and compiler. The canonical command below selects
the complete 150-case union.

From the Cathedral root:

```sh
python3 tools/ports/acpi/interpreter/fixtures.py --check
python3 tools/ports/acpi/interpreter/evidence.py --check
python3 tools/ports/acpi/interpreter/check.py --omega /tmp/cathedral-omega-eaa7993/release/omega
```

`fixtures.py` reproduces 150 authored Omega helper scenarios and their source
maps. Five are direct translations of the pinned `object.rs` Rust unit tests.
ASL-derived cases identify the executed expression/value chain and explicitly
exclude method, namespace, target-store and bytecode execution. Other cases
exercise independent integer-width, malformed-input and destination-boundary
expectations. `evidence.py` verifies the pin, working source/test bytes, source
hashes and exact anchor mappings: two Rust files, 10 translated anchors and
148 pending. Partial method bodies stay pending rather than being called
fully transcribed. No parent inventory is modified by either command.

`check.py` first source-checks the complete fixture corpus, then runs semantic
constant-evaluation fixtures in two-case groups. Three additional fixtures
mutate actual expected arithmetic/copied-byte/error conditions and must produce
an unsatisfied zero-result requirement with evaluated result 1. `--match TEXT`
is available for a bounded scenario subset; it still checks the complete source
and all three controls. All compiler subprocesses operate on temporary fixture
packages with a local path dependency, and the runner prints the binary hash.

The exact compiler is Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`, SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
Native emission/execution, ABI layouts, an external interpreter oracle, AML
bytecode, firmware and hardware tests are not claimed by this runner.
