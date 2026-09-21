# Concatenate execution witnesses

Status: authored, not yet verified. There are 110 behavior/control pairs: 86
complete-store/frame retirement cases and 24 encoded AML cases through the real
loader and Program executor. Independent byte/integer/hexadecimal arithmetic
supplies expectations; no alternate Rust implementation is used.

```sh
python3 tools/ports/acpi/interpreter/concat-execution/check.py \
  --match inline_inline_64,target_named_integer_64,target_field_64,target_field_disabled_64,last_slot_local_rollback_64,self_string_64,aml_inline_inline_64,aml_self_64,aml_nested_64 \
  --record /tmp/cathedral-concat-execution-smoke9.json
python3 tools/ports/acpi/interpreter/concat-execution/check.py \
  --verify /tmp/cathedral-concat-execution-smoke9.json
```

Omit `--match` for the complete corpus. The checker binds all ACPI Omega inputs,
fixture/comparator dependencies, generated text, exact entries, build text,
runner binary and before/after stability. Existing source-bound receipts are not
rewritten. No passing result is claimed until receipt verification succeeds.
Broader unchanged regression replay remains required before this extension lands.

See [the port contract](../../../../../source/libraries/acpi/interpreter/execution/concat.PORT.md)
for source pin, primary rules, allocation atomicity, Field continuation and
remaining opcode/description boundaries.
