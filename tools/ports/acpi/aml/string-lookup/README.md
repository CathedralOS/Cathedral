# Direct String-name lookup evidence

See [the port record](../../../../../source/libraries/acpi/aml/string-lookup.PORT.md)
for direct String admission, scoped lookup and deferred target evaluation.

```sh
python3 tools/ports/acpi/aml/string-lookup/inventory.py --check
python3 tools/ports/acpi/aml/string-lookup/check.py
python3 tools/ports/acpi/aml/string-lookup/check.py --verify-record
```

The retained final replay passed 97 actual checked positive/changed-body-control pairs
and three constant positive/rejecting-control pairs. Each case supplies an
explicit namespace and expected stable ID/path or semantic failure. Every
successful result checks all 16 initialized Path segments and its metadata.
Controls alter an expected ID/error; the three dirty-scope cases instead alter
expected segment15 while keeping the ID correct.

The actual lookup and all result projections occur in a shared scalar-returning
test checker. Authored cases construct their inputs and expectations and receive
its i32 result. This limits repeated compiler origin analysis; it does not replace
runtime execution with a host oracle or change the production helper. Defaults
are tested through an actual initialized containing record.

Three workers use independent temporary builds. The receipt binds the exact
13-file source/build closure, three fixture/driver inputs, two runner recipe
inputs, generated bodies, execution-root build recipe and pinned compiler/runner
hashes. Verification regenerates every body and checks exact counts/results and
all current hashes. The driver uses the existing checked runner and never
rebuilds its shared binary.

No new public Rust or private-expression observation is claimed by this
composition. Native execution, field reads, method invocation and whole DerefOf
execution remain separate work.
