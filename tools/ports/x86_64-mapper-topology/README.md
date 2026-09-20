# Mapper topology evidence

```sh
python3 tools/ports/x86_64-mapper-topology/audit.py
python3 tools/ports/x86_64-mapper-topology/check.py --omega /tmp/cathedral-omega-eaa7993/release/omega
```

`generate_reference.py --check` binds exact private coordinate bodies and explicit observation substitutions to the pin. `generate.py --check` runs the pinned Rust dependency and compares all generated Omega expectations. `check.py` then evaluates the actual Omega machines and rejects a changed expected behavior in every fixture. `--case` narrows checks and `--host-only` selects the Rust/reference leg.

There are 22 fixtures containing 3,131 Rust numeric observations and eight additional checked-input policies. All 512 recursive indices run through six permitted size/coordinate combinations. The package's `mapper-topology.PORT.md` distinguishes public Rust calls, private mirrors and unimplemented pointer/custody boundaries. No live recursive table or CR3 instruction is invoked.

`verification.json` records the completed baseline: all 22 semantic fixtures and 22 changed-body controls passed, along with the 14-file source check and pin audit. It includes exact compiler/dependency hashes and the reference/fixture hashes.
