# Direct Buffer ToString evidence

The [port record](../../../../../source/libraries/acpi/aml/object-to-string.PORT.md)
defines direct IDs, full backing admission, selected-prefix ASCII validation and
zero-tailed semantic results. Final repository-path 195 checked pairs, three constant pairs and 44 public
observations passed; retained receipts validate against current hashes.

From the repository root:

```sh
python3 tools/ports/acpi/aml/object-to-string/inventory.py --check
python3 tools/ports/acpi/aml/object-to-string/check.py
python3 tools/ports/acpi/aml/object-to-string/reference.py
python3 tools/ports/acpi/aml/object-to-string/verify_record.py
```

`check.py` runs 195 actual checked behavior/control pairs in three independent
workers with unique temporary builds and deterministic batch receipt order, then
three positive/changed-body constant-expression pairs. Controls alter the actual
expected output byte or failure case, not just the final contract. All 256 output
bytes are checked. It binds 14 used source/build files, the fixture/driver/cases,
shared runner sources, pinned binary hashes, generated bodies and execution root.
It never rebuilds the shared runner; its immutable paths are declared in check.py.

`reference.py` uses an isolated Cargo target, pinned public Interpreter APIs and
44 synthetic AML ToString operations. Every mapping/I/O/service call is trapped;
mutex creation returns inert numeric handles. The receipt retains all public
outcomes, full AML bytes, 27 exact upstream source/manifest/license hashes and
probe inputs. Reproduction compares the complete receipt; `--write` intentionally
refreshes it. The raw pin includes a discovered NUL and accepts valid UTF-8; Omega
excludes NUL and applies the documented ASCII profile. 33 observations match and
11 differ. They are not claimed as an additional 44 Omega tests.

`verify_record.py` validates both retained stages against current inputs, all
selected positive/control bodies and counts, diagnostic evidence, pinned checkout
hashes, original AML generation and observed comparison classifications. The
verifier is an audit utility; it is not retroactively included among executed
fixture inputs. It does not require the temporary host executable to survive.

No native Omega, allocator, reference evaluation, target write or opcode execution
integration is claimed. Source and tool inputs remain frozen during evidence runs.
