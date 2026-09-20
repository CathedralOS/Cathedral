# Register operand witnesses

See [the port record](../../../source/libraries/x86_64/register-operands.PORT.md).
`generate_reference.py` extracts exact pure Rust fragments with CPU observations
supplied as values and instruction tails removed. Actual live register APIs are
never called. `cases.json` and `observations.json` retain 1,323 inputs/results.

Run `python3 tools/ports/x86_64-register-operands/check.py --omega /path/to/omega`.
The checker verifies source generators, inventory, fresh Rust observations,
14 Omega semantic batches, added malformed-value cases and seven body mutations.
`--host-only`, `--positive-only`, `--controls-only` and `--batch N` select bounded
parts. Rust runs in the default development profile with overflow checks, so
STAR addition panics are observed and compared with explicit Omega rejection.
No native ABI, register access or CPU-state validity is claimed.
