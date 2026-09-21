# Canonical Buffer/String Index checks

`check.py` executes 117 authored behavior/control pairs against the existing byte_storage::make_byte_index checked
Omega body and three representative constant positive/rejecting pairs. Tests
compare the complete ObjectStore: every payload and link, all 32 namespace entries
and path segments, and all 64 initialized byte blocks including nonlogical tails.
The seed poisons inactive object/byte metadata to expose accidental replacement.
Controls change field backing/offset/length, returned outcome, object count,
namespace path, untouched link or a byte outside logical backing. Constant checks
retain full payload/namespace comparison and first/last bytes per block within the
compiler's constant budget; they do not substitute for full runtime byte checks.

```sh
python3 tools/ports/acpi/aml/byte-index/check.py
python3 tools/ports/acpi/aml/byte-index/reference.py
python3 tools/ports/acpi/aml/byte-index/inventory.py --check
python3 tools/ports/acpi/aml/byte-index/verify_record.py
```

Use `check.py --write-cases` to deliberately regenerate authored expectations.
Public `--write` refreshes the retained observation record. Runtime batches use
three independent workers, ten scenario pairs each, through the existing checked
runner; no native publication is claimed. Exact production/source/build, harness,
fixture and binary hashes are verified; the runtime receipt binds its actual
execution root and generated dependency build text/hash.

The public probe invokes actual `Interpreter::new/load_table/evaluate` twice per
row with synthetic `Return(Index(BACK,index,NullName))`. Fourteen successful rows
verify RefOf kind, fresh wrapper and field identities, unchanged backing identity,
and exact bit offset/8-bit length; thirty rows observe IndexOutOfBounds twice.
No field token, private method or hardware is accessed. All callbacks are trapped
except construction of one inert mutex. Empty/short/256-byte Buffer and ASCII
String sources and unsigned maximum indices are included. These observations do
not claim field read/write or target Store execution. Exact AML, root/build text,
probe/lock inputs, comparison-source hashes and all27 upstream hashes are retained.

This is additional evidence for existing production code. No new Omega module is
introduced. Named/Local/Arg references resolve to their backing identity; opaque
references remain unsupported. Namespace entry counts are preserved even when
they exceed normal namespace capacity because this helper allocates object slots.
