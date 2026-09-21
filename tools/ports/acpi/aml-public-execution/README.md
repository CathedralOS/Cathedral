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

## Declaration and alias witnesses

`check_pipeline.py` reconstructs five existing pipeline fixtures directly from
their retained initialized input bytes, then uses the same public Rust loader
and evaluator. Addition, nested calls and method redeclaration agree. The two
cross-scope alias cases return 22 in the pin, resolving `X` from the alias's D1
scope; Cathedral's documented declaration observation retains D0 and returns 11.
The same difference survives original-name rebinding. These are explicit
differences, not added compatibility passes or new source translations.
No source replacement, hardware callback or direct namespace mutation is used.

```sh
python3 tools/ports/acpi/aml-public-execution/check_pipeline.py
```

`pipeline-observations.json` separately binds these five cases, the actual host
binary and original pipeline fixture generator. It does not replace the existing
22 Omega pipeline cases or their mutation controls.

## Generic object observations

`check_generic.py` adds 94 original finite bytecode cases. The probe's optional
generic renderer reads immutable public Buffer, String, Package, Reference and
BufferField payloads, retaining wrapper kinds and byte-exact hex strings. A
64-level/512-node observation budget and pointer-equality cycle detection bound
rendering; pointers are never forged, manually dereferenced or retained in the
record. The original 49 integer and five pipeline observations were rerun after
the probe change, with exactly the same results.

The new record contains 85 value results, six explicit evaluator errors and
three caught panics. It is host reference evidence for pending generic Omega
integration, with no new Omega compatibility pass or translated-anchor claim.

| Authored scenario | Actual pinned public result |
| --- | --- |
| Store Local holding RefOf(named Integer) | InvalidOperationOnObject on Reference; named value remains unchanged. CopyObject overwrites Local successfully. |
| Store/Copy Arg receiving plain named Integer or caller Local | Changes the caller's underlying object. |
| Store/Copy Arg receiving RefOf(named Integer) | Replaces the intermediate Named wrapper with Integer; original named object remains unchanged. |
| RefOf return / local escape | Retains RefOf→Named or RefOf→Local wrapper chains, including the escaped local's value. |
| Add(named value, method which replaces that value) | Reads the retained first operand at retirement: 22 + 1 produces 23. |
| Direct Store/Copy to package Index | StoreToInvalidReferenceType. Index saved in a Local followed by Store does mutate the member. |
| Copy package, then mutate member through saved Index | Both packages observe the changed child. This witness uses the pin's Local write-through behavior. |
| Copy Buffer, then mutate copied byte through saved Index | Source retains its bytes; destination changes independently. |
| Store String "12" into named Integer | Produces 12849 from raw bytes; CopyObject replaces the type with String. |
| Oversized Buffer initializer, Mid beyond remaining suffix | Panics are retained explicitly. |
| ToString across a NUL | Includes the NUL byte in the returned String. |

Other rows cover exact/padded/empty literals, both integer widths, out-of-bounds
Index, String and Buffer byte fields, explicit conversions, same/mixed-type
concatenation and Mid boundaries. These observations are deliberately not
treated as the specification. [ACPI 6.6 method calling conventions](https://uefi.org/specs/ACPI/6.6/05_ACPI_Software_Programming_Model.html#method-calling-convention)
separate shared incoming values from argument assignment. [Object storing and
copying rules](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#rules-for-storing-and-copying-objects)
require Local replacement and distinguish reference-valued Args and named
conversion. Production integration must retain explicit dispositions for these
pin differences rather than copying them as default behavior.

```sh
python3 tools/ports/acpi/aml-public-execution/check_generic.py
```

`generic-observations.json` binds authored bytes, exact public outputs, compiler,
binary, probe/generator and upstream source hashes. Each fresh child must finish
within five seconds, with no forbidden service call and only the constructor's
mutex identity. No firmware fixture or upstream private algorithm is copied.
