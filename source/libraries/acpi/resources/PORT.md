# Bounded resource descriptor parsing

Status: **tested first slice**. All 117 checked-interpreter positives and 117
changed-body controls pass; four representative pairs also pass constant semantic
evaluation. The actual public Rust probe records 102 observations, including 33
normalized supported-descriptor agreements. See the [verification record](../../../../tools/ports/acpi/resources/verification.json) for exact source, compiler and fixture hashes.

This independent child package implements a first resource-descriptor slice over
caller-initialized `[u8;4096]` bytes and an explicit logical length. It produces
ordinary data and inert byte spans. It supplies no AML execution, namespace
resolution, interrupt setup, DMA, I/O, memory mapping or allocator authority.
The 4096-byte envelope is an implementation profile, not an ACPI maximum.

The exact upstream is [rust-osdev/acpi 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/resource.rs),
crate 6.1.1, MIT OR Apache-2.0, copyright 2018 Isaac Woods. Existing exact notices
and licenses remain in `THIRD_PARTY_NOTICES.md` and `licenses/rust-osdev/acpi/`.
This is a modified bounded translation. The complete file and every lexical
anchor are bound by `inventory.json`; pending rows retain unfinished families.
The final audit has 45 translated, 34 pending and 8 deliberately omitted anchors.
The aggregate `Resource`, list parser and descriptor dispatcher remain pending
with explicit partial targets until GPIO and SerialBus/I2C semantics are covered.

## API and scope

`resource_parse::parse_one(input, length, at)` first establishes the complete
small or large descriptor envelope. It returns `Parsed` with a successful semantic
`Resource` case or an error with `Resource::None`. Offset addition occurs only
after subtraction-based bounds checks; a declared body cannot exceed the logical
length or initialized array. Failed results carry no parsed descriptor. Exact
lengths are required for IRQ, DMA, I/O, FixedMemory32 and EndTag; address and
ExtendedIRQ descriptors validate their fixed prefix, declared array and optional
tail. Direct calls to the byte readers are also bounded.

Supported cases are small IRQ/DMA/I/O; large FixedMemory32; Word/DWord/QWord
address descriptors for memory, I/O and bus numbers; and ExtendedIRQ. IRQ masks
retain their 16-bit form. ExtendedIRQ retains a checked table span and count up
to the wire's 255 entries; `resource_irq::irq_at` returns the selected number.
It revalidates a caller-constructed span/count before reading. The caller must
retain the same input bytes while using spans. Spans neither own bytes nor resolve
objects. No heap container is required.

Address values, granularity, translation and length remain raw numeric facts.
Address allocation constraints, arithmetic such as base+length, contextual
`_CRS`/`_PRS`/`_SRS` requirements, and resource arbitration are not validated.
Type-specific address flags are preserved raw. In particular, a successful
address parse does not establish a usable region or valid allocation. Small IRQ
`is_consumer = false` preserves the pin's explicit producer assumption; no new
hardware observation is inferred.

Optional source tails retain their index and a span excluding the final NUL.
The byte profile requires an index and final NUL, with nonzero 7-bit bytes before
that terminator. Empty NUL-terminated names are retained as bytes. No AML pathname
grammar, namespace lookup, negotiated `_OSC` capability, or source-index policy
is guessed. The stricter byte profile is independent of the pin, which discards
these tails for the supported address/interrupt families.

Other defined descriptor tags return `Resource::Unsupported` with the entire
header+payload span and original tag. This is successful framing only: their
payload semantics and their internal length requirements remain unvalidated.
GPIO, serial buses/I2C, dependent functions, FixedIO/FixedDMA, vendor descriptors,
other memory forms, ExtendedAddress, generic registers, pins and ClockInput are
not decoded in this slice. Reserved tags return `BadEncoding`.

`resource_template::validate_template` performs a bounded scan of the logical
buffer. It counts descriptors and unsupported descriptors, and requires an exact
one-byte EndTag payload at the logical end. `Success` means the implemented
encoding checks and strict termination/checksum profile passed; `unsupported > 0`
explicitly prevents claiming every descriptor was semantically understood. Even
with zero unsupported descriptors, contextual allocation policy remains separate.
Partial counts on failure describe only the observed prefix. This scanner does
not construct or install an AML object list.

A zero checksum byte bypasses the sum check. A nonzero checksum requires the sum
of all bytes from the template start through EndTag to be zero modulo 256.
Missing EndTag, bad checksum and trailing bytes have distinct outcomes. Trailing
bytes are rejected before checksum evaluation. Requiring
EndTag to consume the entire logical buffer is this package's strict template
profile; `parse_one` remains usable on a descriptor within a larger buffer.

## Primary-spec corrections and preserved differences

The primary reference is [ACPI 6.6, section 6.4](https://uefi.org/specs/ACPI/6.6/06_Device_Configuration.html#resource-data-types-for-acpi).
These choices are explicit changes relative to the pin:

| Evidence | Bounded behavior and difference |
| --- | --- |
| Tables 6.26 and 6.38, descriptor envelopes | Check header and complete body before dispatch. The pin can panic on short headers and out-of-range `split_at`. |
| Table 6.28, small IRQ | Default edge/high when information is absent. Reject bits 7:6 and combinations other than high-edge or low-level, exactly as the table requires. The pin accepts the invalid combinations. Ignored bits 2:1 are not rejected. |
| Table 6.29, DMA | Reject reserved bit 7 and transfer code 3. The pin panics on code 3. Ignored bits 4:3 remain accepted. |
| Table 6.33, I/O | Reject reserved information bits 7:1; the pin discards them. |
| Table 6.43, FixedMemory32 | Preserve ignored information bits and decode write permission/base/length, as the pin does. |
| Tables 6.45–6.47, address descriptors | Reject reserved general flag bits; retain raw type flags and source bytes. Type 10 (PCC) and vendor kinds return explicit unsupported envelopes. The pin rejects PCC and panics for vendor kinds. |
| Table 6.52, ExtendedIRQ | Check count and full number table, require a nonzero count and reject reserved flags 7:5. Bit 2 means active-low when set, matching the actual Rust body; its preceding comment states the opposite. Contextual single-entry `_CRS`/`_SRS` enforcement is pending. |
| Table 6.37, EndTag | Require payload length 1 and honor zero-checksum bypass/nonzero modulo sum. The pin ignores checksum and returns immediately even with trailing bytes; it also accepts a list without EndTag. |
| Table 6.39, large tags | Tag 0x03 is reserved and rejected; the pin skips it. ClockInput 0x13 is defined and retained unsupported; the pin rejects it as reserved. |

No correction rewrites the pinned source. Public reference observations retain
both accepted results and panics so the divergences remain reviewable.

## Validation and pin updates

The host probe calls the **actual public** pinned
`resource_descriptor_list(Object::Buffer(bytes).wrap())`. No private body mirror
or `Handler` implementation is involved. Supported results are normalized into
numeric facts and compared with independently authored fixture expectations.
`catch_unwind` records pin panics for malformed synthetic inputs; Omega cases
require explicit failure or unsupported outcomes instead. Source-tail and strict
EndTag behavior are separately grounded in the primary tables and declared
profile. Fixtures are original synthetic bytes, not firmware dump transcriptions.

The checked-interpreter runner checks the authored package and dependencies once,
then executes actual parser/template/IRQ-access machines and a changed expected
behavior inside every test body as its negative control. Four representative
pairs are also retained under constant semantic evaluation via `omega --check`. Its
source hash includes this package plus the reused parent `fixed_bytes`/`bytes`
helpers and package builds. No native ABI, production import or live firmware
claim follows from these semantic checks. Exact counts and hashes are retained in the tools verification records.

On a pin change, review the full resource file and all anchor dispositions,
licenses, actual public probe observations, reserved/defined tag tables, optional
source syntax and checksum rules. Regenerate the corpus and rerun all actual
Omega positives and body controls. Preserve explicit pending GPIO/I2C and other
family semantics rather than treating their envelopes as completed parsers.
