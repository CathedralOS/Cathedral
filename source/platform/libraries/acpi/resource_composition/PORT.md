# Bounded resource-template concatenation — partial ACPI-005

Status: source check passes 18 files; all 25 checked-interpreter cases and 25
controls pass, together with four representative constant-evaluation pairs.
This child package composes initialized detached `[u8;4096]` resource buffers.
It reuses the resource parser's explicit outcomes and produces `Composition`
case data, with no generic AML Object, namespace, Store target or interpreter
service. The caller retains source/output storage and supplies logical lengths
and output capacity, each at most 4096.

The modified result-construction algorithm is from the `Opcode::ConcatRes`
branch in `src/aml/mod.rs` at rust-osdev/acpi 6.1.1 revision
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, copyright 2018 Isaac Woods,
MIT OR Apache-2.0. Exact notices/licenses remain under `licenses/rust-osdev/acpi/`.
All tests are original synthetic buffers, without firmware dumps.

[ACPI 6.6 section 19.6.13](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#concatenaterestemplate-concatenate-resource-templates)
specifies an empty buffer as an empty resource template and rejects a one-byte
input. `compose` applies those rules at this boundary. For other inputs it
requires the existing strict `resource_template::validate_template` profile:
complete descriptor envelopes, supported-family checks, final exact EndTag,
checksum-zero bypass or valid modulo sum, and no trailing data. That validation
is deliberately stronger than the pin's result block, which merely removes the
last two bytes if the penultimate byte equals 0x79 and otherwise appends them.
Malformed inputs accepted by that Rust block are retained as reference differences.

Validation order is capacity/input extents, complete left input, complete right
input, then combined output extent. Invalid input returns its side and parser
outcome. No write occurs until the entire preflight succeeds. Each original
EndTag is removed, the left and right descriptor prefixes are copied unchanged,
and a new `[0x79,0x00]` EndTag is appended. The zero checksum selects the defined
no-checksum form, following the pin; it is not a recalculated nonzero checksum.
The logical output length includes this EndTag. No bytes past the logical result
are changed. Capacity failure does not truncate or partially concatenate.

Defined but unsupported descriptors remain inert complete spans accepted by the
parser's framing policy. `Composed.unsupported` adds both source counts, so a
successful composition does not claim full descriptor semantic validation.
Neither an empty unsupported count nor copied descriptors prove valid resource
allocation, controller state, namespace resolution or hardware authority.

The reference generator extracts the exact private `ConcatRes` result block,
replacing only `Object::Buffer(buffer).wrap()` with the returned byte vector.
It does not call the actual public interpreter, execute opcodes, mutate targets,
or imply the pin handles one-byte and malformed inputs as Cathedral does.
Successful-profile bytes are compared with that clearly labelled mirror.

The checked Omega harness executes actual composition/parser/copy bodies and
changed full-output comparisons. Every one of 4096 output bytes is checked, including
unchanged tails and error atomicity. Full-capacity input and combined overflow,
empty inputs, existing nonzero checksums, side ordering and u64::MAX extent
rejections have dedicated scenarios. Representative constant-evaluation pairs
check complete small results, an eight-byte prefix and a distant tail sentinel;
the full-buffer checks belong to the separate checked execution stage. The
reference includes 23 private result-block mirrors and nine accepted-profile
byte agreements. Native Omega, firmware, complete AML dispatch and
resource acquisition remain outside this component.
