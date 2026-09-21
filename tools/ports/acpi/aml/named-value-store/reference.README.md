# Public named-value Store observations

These original fixtures invoke the pinned public Interpreter loader and evaluator
with `Store(Arg0, TARG); Return(Arg0)`. Nine fixtures instead select a namespace
alias of a distinct source. The public namespace supplies before/after target
observations and pointer identity. Every fixture creates a fresh interpreter and
process; all service callbacks are trapped except the inert constructor mutex.
No private method is called or mirrored, token fabricated, or device accessed.

```sh
python3 tools/ports/acpi/aml/named-value-store/reference.py
```

`--write` deliberately refreshes the receipt. `--smoke --write` writes a separate
ten-case smoke record. The default command reproduces every retained observation
and source/build binding. `verify_record.py` verifies both checked Omega execution
and this public record without rerunning them.

All 297 rows execute real Store opcodes. Integer destinations, String destinations
of lengths 0/3/256, and Buffer destinations of lengths 0/1/4/8/256 are paired with
16 Integer/String/Buffer inputs under both AML revisions, plus nine named-source
aliases. Inputs include high u64 bits, empty byte values, hexadecimal prefixes,
nonhexadecimal stopping, embedded Buffer NULs, non-ASCII Buffer bytes and the
85/86-byte Buffer-to-String formatting boundary. Source values remain unchanged;
destination type and identity are retained.

An independent primary-rule oracle classifies 68 agreements, 116 conversion-policy
differences, 97 observations outside the bounded profile and 16 primary-rule
rejections that succeed in the pin. The pin casts raw bytes: Integer sources always
supply eight bytes, String destinations decode UTF8 lossily and truncate at NUL,
and Buffer destinations resize to the source byte length. The primary component
instead normalizes integer width, parses/formats hexadecimal text, and preserves
admitted positive Buffer target extents. Empty numeric conversions and overcapacity
formatted Strings are explicit rejection differences. Zero-extent Buffer, empty
String-to-Buffer and Buffer-to-Buffer are local profile exclusions, not claimed
normative errors. Public observations of those cases do not expand Omega coverage.

The sequential Return observes the source separately; this fixture makes no claim
about the Store expression result. An earlier nested `Return(Store(...))` smoke
returned Uninitialized in the pin: its opcode dispatcher discards `do_store`'s
returned object. Exact diagnostic inputs and the first observation are retained
outside the release set at `/tmp/cathedral-named-store-nested-return-diagnostic`.
No failed diagnostic run is counted as verification.

Self-store is tested only by the safe Omega snapshot fixtures. Invoking it here
could overlap the pin's unsafe mutable target access with immutable source access.
The nine public alias fixtures always use a distinct source and verify that fact.
Only immutable observations occur after evaluation returns or unwinds.

The public receipt binds exact execution/probe roots, Cargo manifest text/hash,
isolated binary, four probe/lock inputs, production comparison hashes and all 27
pinned source/manifest/license hashes. Omega mappings identify comparison scope;
this Rust record does not claim to execute Omega. Pinned rust-osdev/acpi revision:
257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5, MIT OR Apache-2.0, copyright 2018 Isaac Woods.
Existing repository notices apply.
