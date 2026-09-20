# AML static-layer evidence

`audit.py` checks the exact pin, six-file source inventory, license/manifest digests, 113 opcode/range facts and metadata-only upstream test provenance. `check.py` runs original Omega fixtures and body-mutating negative controls. `fixtures.py` regenerates the bytes and assertions; `cases.json` names each behavior and its control.

```sh
python3 tools/ports/acpi/aml/audit.py
python3 tools/ports/acpi/aml/check.py --omega /tmp/cathedral-omega-eaa7993/release/omega
```

Use `--case NAME` repeatedly for focused checks, or `--positive-only` while debugging. Full acceptance requires every positive and negative. The named fresh compiler is revision `eaa7993a23623cd8fabf45350340479c5c9c7879`, SHA-256 `2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.

No fixture is an imported firmware dump or a copied external uACPI example. These tests establish checked semantic behavior of the bounded static layer. They establish no native layout, handler operation, method execution or complete ACPI-004/005 interpreter support. The authoritative staged scope and pin differences are in `source/libraries/acpi/aml/PORT.md`.

`coverage.json` records the supported static terms without claiming full interpreter translation. `compiler-notes.md` records source-form adjustments; `reproduce-private-helper.py` retains an isolated witnessed spelling regression.

`verification.json` records the completed baseline: 27 positive semantic cases, 27 rejected body mutations, and the 18-file source check. Acceptance ran as the original 23 cases plus four additional cases against the same unchanged package hash. Fixture hashes bind that evidence to the checked assertions; rerunning the command above now checks all 27 cases together.
