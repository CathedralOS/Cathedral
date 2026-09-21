# Actual public AML interpreter observations

This host harness loads synthetic AML with the actual pinned
`Interpreter::load_table` and calls `Interpreter::evaluate`. It imports the
existing integer executor's original fixtures and selected licensed upstream
scenario adaptations; it does not copy firmware tables or an external test suite.
The pin is `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, MIT OR Apache-2.0.

All 79 Omega fixture rows have an explicit disposition: 49 run through public
Rust APIs, and 30 are omitted for specified host-service, nonterminating,
direct-storage-edit or caller-budget boundaries. Each observed row runs in a
fresh child process with a five-second deadline. The generated table, exact
bytes, outputs and source hashes are retained in `observations.json`.
Twenty-four cases agree on successful return values and all requested namespace
effects. The other 25 records include both corresponding errors and deliberate
or upstream differences; they are not labelled successful equivalence tests.

The Handler has no device implementation. Physical mappings, memory, I/O, PCI,
clock, sleep, synchronization, debug and fatal callbacks increment a counter and
panic. Construction alone allocates one synthetic mutex identity; acquiring it
is forbidden. Each completed observation requires zero forbidden calls and one
constructor identity. Fixed-register metadata uses the actual SystemIo GAS
constructor, whose pinned body stores data without mapping or I/O. No FACS,
physical pointer, ObjectToken forgery or interpreter source modification is used.
The source byte vector remains alive until after the interpreter is dropped.

## Observed differences and corresponding errors

| Cases | Public pin observation and Cathedral profile |
| --- | --- |
| `divide_targets` | Pin produces 43 by storing quotient before remainder. Cathedral produces 34 with AML's remainder-then-quotient target order. This correction was already documented. |
| Three `unary_*`, `from_bcd`, `to_bcd`, `invalid_bcd` | Pin dispatch collects a target but its retirement extractor expects only the value, returning InternalError. Cathedral executes valid unary/BCD forms and explicitly rejects malformed BCD. |
| `zero_division`, `malformed_package` | Pin panics; Cathedral returns its explicit bounded error. |
| `call_no_return_operand`, uninitialized local/argument | Pin returns an Uninitialized object. Cathedral permits void statement calls but rejects uninitialized value operands. |
| `store_null` | Pin discards the store; Cathedral requires a real Store SuperName target. Optional arithmetic targets remain distinct. |
| `truncated_expression`, `noop_operand` | Pin returns Uninitialized; Cathedral rejects malformed/incomplete operands. |
| `unsupported_buffer` | Pin returns a Buffer. The existing executor explicitly excludes generic Buffer execution. |
| `block_depth`, `expression_depth` | Pin succeeds; Cathedral enforces its declared finite stacks. |
| Argument count, CopyObject null, Break/Continue outside loop, orphan Else, truncated constant, missing name | Both reject; exact upstream error and Cathedral outcome are retained rather than conflating their error types. |

The primary [ACPI 6.6 operator specification](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#divide-integer-divide)
defines the Divide target order. Encoding/arity follows
[ACPI 6.6 AML grammar](https://uefi.org/specs/ACPI/6.6/20_AML_Specification.html).
These observations supplement the existing actual Omega checked-body tests;
they do not establish full AML compatibility, Omega native execution, hardware
behavior or newly translated source anchors.

```sh
python3 tools/ports/acpi/aml-public-execution/check.py
```

The command builds locked/offline with `nightly-2026-09-04`, reruns public calls
and rejects any recorded-input/output drift. Use `--write` only after reviewing
intentional fixture, probe or observation changes. Exact source receipt hashes
are retained; differing compiler/build paths can also change the binary hash.
