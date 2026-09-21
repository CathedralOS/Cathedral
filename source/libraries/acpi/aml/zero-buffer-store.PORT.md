# Existing zero-length Buffer conversion

Status: 86 checked behavior/control pairs and six public observations pass in
the isolated `/private/tmp/cathedral-acpi-buffer-store` worktree; see the
[receipt and commands](../../../../tools/ports/acpi/aml/zero-buffer-store/README.md).
This extends the existing canonical preparation and named Store components.

An existing admitted zero-length Buffer now accepts an Integer scalar, an
allocated Integer, or an allocated String of any admitted length. The result is
an owned zero-length Buffer at the same destination ID, with all 256 backing bytes
zeroed. Source identity, object links, namespace metadata and other objects/blocks
are unchanged. No temporary object allocation is introduced. Both Integer widths
and Source/Owned backing use the existing conversion and private publisher.

Destination admission still precedes source admission. The preparation helper
checks ID/count, maximum extent, source kind and complete String backing/ASCII
before copying, even when the requested extent is zero. Invalid source IDs,
backing owners, capacity, bounds or encoding are errors, and preserve the entire
store. Unsupported wrappers now reach UnsupportedValue at zero extent instead of
the former blanket Bounds exclusion. The scalar API has no source ID to validate.

## Primary evidence and remaining decisions

[ACPI 6.6 Table 19.7](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules)
expressly fits Integer/String conversion to an existing Buffer's extent;
[Buffer declarations](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#buffer-declare-buffer-object)
permit zero-length objects. Applying those rules truncates to zero for an
already-present zero-length target. The empty-String special case also produces
zero length here, so it gives the same result.

For an empty String and a **positive-length** existing Buffer, the same conversion
row's special zero-length result and its general existing-target padding rule
need a precedence decision. That case remains the local Empty exclusion after
complete source admission. Explicit ToBuffer replacement does not decide named
Store destination policy. Historical
[ACPI 3.0a Table 17-8, printed page 446](https://www.intel.com/content/dam/www/public/us/en/documents/articles/acpi-config-power-interface-spec.pdf)
contains the existing-extent wording, before the explicit empty-String sentence;
it does not resolve the current precedence question. This narrow design decision
leaves other ACPI-005 work available.

[Unequal same-type Buffer extents](named-buffer-store.PORT.md#extent-decision-still-open)
remain a separate compatibility decision. Equal Buffer copies, including empty
self-store, retain the preceding milestone's behavior.

## Scope and evidence

The focused suite has 86 behavior/control pairs: 75 use the existing full-store
comparator, and 11 call preparation directly with complete byte comparisons.
Cases cover both widths, zero/high-bit/MAX Integers, empty/1/256-byte Strings,
Source/Owned storage, malformed empty sources, final-byte ASCII errors,
destination/source precedence, source aliases, a full arena and one-object scalar
store, empty Buffer self-store, retained unequal-extent exclusions, and positive
conversion regressions. Every store check compares all 16,384 arena bytes plus
all object/namespace metadata. Controls change expected results, byte tails,
identity or links. The new receipt binds the exact production and reused fixture
closure, generated bodies, execution root and pinned checked runner.

The checked run completed in 142.439 seconds with two workers. Maximum
observed evaluator fuel was 471,789, below the unchanged 10,000,000 ceiling.
Source and runner hashes remained unchanged throughout execution; both new
receipt verifiers and the source inventory audit pass.

Six actual public pinned Object replacement observations document five resize
disagreements and one empty-String agreement. They are implementation observations,
not a normative oracle. No interpreter, fabricated ObjectToken or provider is used.
The pin resizes zero-length targets. ACPICA also grows zero-length targets in its
[Buffer store implementation](https://github.com/acpica/acpica/blob/master/source/components/executer/exstorob.c);
this component follows primary existing-target Integer/String conversion geometry.

The licensed partial mapping is rust-osdev/acpi
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, `src/aml/mod.rs:2406 do_store` and
`src/aml/object.rs:317 replace_with_implicit_casting`, MIT OR Apache-2.0,
copyright 2018 Isaac Woods. Notices remain in `licenses/rust-osdev/acpi` and
`THIRD_PARTY_NOTICES.md`. The [inventory](zero-buffer-store-inventory.json) keeps
all 158 aggregate anchors pending. No aggregate Store, target dispatch, native
execution or new constant-evaluation completion is claimed.

Earlier source-bound receipts remain unchanged historical evidence. In particular,
the 97/504 equal-extent milestone receipts describe checkpoint `9e45adc`, whose
production sources predate this change; their current-input verifiers correctly
reject the changed closure. This bounded extension does not claim a fresh replay
of those 601 pairs or the original preparation/scalar corpora.
