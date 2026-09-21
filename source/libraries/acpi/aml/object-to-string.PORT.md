# Direct canonical Buffer ToString

Status: tested. All 195 actual checked behavior/control pairs, three constant
expression pairs and 44 public observations passed from final repository paths.

`object_to_string::to_string(input, length, unit, store, object, maximum)` accepts
a direct canonical Buffer slot and returns `StringResult::Failure(reason)` or
`StringResult::String(length, bytes[256])`. Failure is the first case and its default
reason is canonical `ConversionFailure::InvalidState`. `maximum` is an already
evaluated unsigned Integer value; this detached boundary performs no implicit
conversion or 32-bit normalization. `length` instead describes the initialized
source-input extent. A successful logical String excludes the NUL terminator;
every byte after its logical extent is zero, including an empty result's full tail.

[ACPI 6.6 §19.6.143](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#tostring-convert-buffer-to-string)
specifies copying Buffer bytes until the requested length or NUL, and an empty
String for an empty Buffer. The adapter reuses `conversions::buffer_to_string` and
its explicit bounded ASCII profile. Bytes 1–127 are admitted; bytes 128–255 before
the stopping point fail Encoding. A NUL is never part of the logical String.
Encoding after NUL or the supplied maximum is deliberately irrelevant.

The object count is staged as unsigned and checked against 64 before direct ID
lookup. Only Value::Buffer proceeds: String, Integer, NameReference, all Reference
kinds, BufferField, region fields and service values are not implicitly evaluated.
`byte_storage::read_bytes` validates the complete current backing first. Source
unit/bounds, the 1024-byte initialized input bound, both initializer/declared Buffer
extents, and Owned owner/initialization/256-byte extent are checked even when
maximum is zero. Source Buffer virtual zero padding and initializer-dominant size
reuse canonical storage semantics. For an Owned Buffer, irrelevant source input
metadata and unused block tails are ignored. After backing admission, the existing
conversion scans only the selected prefix and writes into initialized zero storage.
No error exposes a partially assembled result.

This is a component of `Interpreter::do_to_string` at upstream
`src/aml/mod.rs:2068`, pinned rust-osdev/acpi
257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5, copyright 2018 Isaac Woods,
MIT OR Apache-2.0. Notices remain in `licenses/rust-osdev/acpi` and
`THIRD_PARTY_NOTICES.md`. The pin's `split_inclusive` retains a found NUL and its
Rust UTF-8 decoder admits valid multibyte text; these are two distinct recorded
differences from this profile. No compatibility branch reproduces either behavior.
The source mapping retains the aggregate method as pending: reference evaluation,
Object allocation, optional target Store, contribution and opcode retirement are
not implemented by this ordinary-data adapter.

The 195 original cases cover Source/Owned storage, NUL at start/middle/end,
selected-prefix encoding versus ignored suffix, empty/full capacity, maximum 0,
1, 2, 255, 2^32 and u64::MAX, virtual zeros, initializer-dominant length, all
reference kinds, unsupported objects, malformed owners/metadata/counts and default
failure. Every success checks all 256 output bytes; each body control changes an
expected byte (including an otherwise zero tail), or changes an expected failure.
Three representative constant-expression pairs independently require the actual
helper body result to equal zero and reject the changed expectation.

The 44 host observations use actual public Interpreter::new/load_table/evaluate
with original synthetic AML, not copied private bodies. All hardware/service calls
are trapped; only inert mutex-handle creation is permitted. They retain 33 matching
outcomes and 11 raw differences: seven NUL-only differences, three UTF-8-only
differences, and one with both.
These public probes exercise the pinned opcode under revision 2. They are separate
from the 195 Omega body cases and do not claim all host inputs run in Omega. Host
error diagnostics remain raw. No native Omega execution, provider authority,
namespace mutation, allocation or installed AML result is claimed.

Reproduction and retained-receipt validation are in
[the tool README](../../../../../tools/ports/acpi/aml/object-to-string/README.md).
