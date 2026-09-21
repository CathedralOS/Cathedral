# Field-write retirement witnesses

Status: authored, verification running. These fixtures exercise actual
`retire_operation` and `complete_field_write` bodies. They do not drive a provider
or prove full bytecode/pipeline integration. The retained receipt must pass exact
input, generated-source, entry, build and runner-binary checks before any result
is claimed.

```sh
python3 tools/ports/acpi/interpreter/field-write-execution/check.py \
  --match store_converted_result_64,divide_both_64 \
  --record /tmp/cathedral-write-retirement-smoke2.json
python3 tools/ports/acpi/interpreter/field-write-execution/check.py \
  --verify /tmp/cathedral-write-retirement-smoke2.json
```

Omit `--match` to select all 34 behavior/control pairs. Each intermediate step
compares the complete store and frame, including inactive arrays and every new
write-continuation member. A control alters the expected inactive cache value.
The checker uses the immutable Omega `eaa7993` runner without compiler changes.
Earlier receipts remain bound to their original source identities.
