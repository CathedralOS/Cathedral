# Concatenate execution witnesses

Status: authored, not yet verified. There are 110 behavior/control pairs: 86
complete-store/frame retirement cases and 24 encoded AML cases through the real
loader and Program executor. Independent byte/integer/hexadecimal arithmetic
supplies expectations; no alternate Rust implementation is used.

```sh
python3 tools/ports/acpi/interpreter/concat-execution/check.py \
  --match inline_inline_64,target_named_integer_64,target_field_64,target_field_disabled_64,last_slot_local_rollback_64,self_string_64,aml_inline_inline_64,aml_self_64,aml_nested_64,malformed_right_before_fit_64,cycle_64 \
  --record /tmp/cathedral-concat-execution-smoke11.json
python3 tools/ports/acpi/interpreter/concat-execution/check.py \
  --verify /tmp/cathedral-concat-execution-smoke11.json
```

Omit `--match` for the complete corpus. The checker binds all ACPI Omega inputs,
fixture/comparator dependencies, generated text, exact entries, build text,
runner binary and before/after stability. Existing source-bound receipts are not
rewritten. No passing result is claimed until receipt verification succeeds.
Broader unchanged regression replay remains required before this extension lands.

After the focused check passes, run the complete 110-pair Concatenate corpus,
then replay the existing 315-pair Mid/executor suite against this checkout. Keep
the new receipts separate from the historical Mid receipt. The retained suite
contains 74 Mid AML, 42 Mid retirement, 79 integer, 55 generic, 22 pipeline and
43 ToInteger pairs; its original assertions and changed-expectation controls are
reused unchanged. Generation of all 315 pairs has passed, but execution against
the Concatenate extension is still pending.

```sh
python3 tools/ports/acpi/interpreter/concat-execution/check.py \
  --record /tmp/cathedral-concat-execution-full110.json
python3 tools/ports/acpi/interpreter/concat-execution/check.py \
  --verify /tmp/cathedral-concat-execution-full110.json
python3 tools/ports/acpi/interpreter/mid-execution/check.py \
  --batch-size 1000 --workers 1 \
  --record /tmp/cathedral-concat-regressions315.json
python3 tools/ports/acpi/interpreter/mid-execution/verify_record.py \
  /tmp/cathedral-concat-regressions315.json --require-binaries
```

The single regression worker limits contention with other active validations.
Both checkers bind the current production sources and binary before and after
execution. A changed source snapshot requires a fresh run; successful generation
or an earlier branch's receipt does not satisfy this replay.

See [the port contract](../../../../../source/libraries/acpi/interpreter/execution/concat.PORT.md)
for source pin, primary rules, allocation atomicity, Field continuation and
remaining opcode/description boundaries.

The separate public Rust probe completed all 24 encoded AML cases: 16 value/state
agreements and eight retained primary/pin conversion differences. It recovers the
exact initialized AML arrays from the checked fixture generator, uses the unchanged
service-trap harness and pinned crate, and records every result. All runs returned
with zero forbidden callbacks; 86 direct canonical-state cases are explicit
omissions from this public probe.

```sh
python3 tools/ports/acpi/interpreter/concat-execution/public/check.py --acpi-source /path/to/pinned/acpi
python3 tools/ports/acpi/interpreter/concat-execution/public/check.py --verify --require-binary
```

These are public Rust observations, not proof that the Omega assertions passed.
