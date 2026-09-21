# Bounded ASL textual names

Status: **published and verified at the final repository paths**. All 150 checked
positives and 150 changed-body controls passed in 25.568 seconds; three constant
positives and three rejecting body controls passed in 44.071 seconds. Both stages
bind the same 21 explicit input hashes. Current retained-record verification and
all 84 actual public Rust observations reproduce successfully. `name_text.omg`
converts initialized textual names to canonical `model::Path` and back. It adds
no namespace lookup, evaluation, opcode dispatch, object reference or authority.

The modified source derives from rust-osdev/acpi
[`namespace.rs` at 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/namespace.rs),
MIT OR Apache-2.0, copyright 2018 Isaac Woods. Exact upstream notices and licenses
remain in Cathedral's `THIRD_PARTY_NOTICES.md` and `licenses/rust-osdev/acpi/`.
`name-text-inventory.json` binds the whole upstream file, marks eight relevant
anchors translated and two Rust formatting hooks omitted, and leaves the 42
other anchors outside this slice. It is not a whole-namespace completion claim.

## Primary grammar and deliberate policy

The profile follows [ACPI 6.6 §19.2.2](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#asl-name-and-pathname-terms):
leading root or parent prefixes, one-to-four-character segments and dotted path
separators. Root-only and parent-only names are valid; an empty unprefixed name
is not. Prefix characters cannot appear inside a path.
[§19.3.1](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#asl-names)
explicitly defines ASL names as case-insensitive and converted to uppercase.
This module therefore accepts ASCII lowercase and folds it into the existing
uppercase canonical segment representation.

This is textual ASL-name conversion. It consumes already-decoded text bytes,
not the source spelling of a quoted String literal; no quote removal or escape
processing occurs. Nor is it the AML wire NameString decoder, which already
belongs to `names.omg`. [§19.6.30, DerefOf](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#derefof-dereference-an-object-reference)
uses ASL names for String operands, but this module performs none of the required
scope resolution or object evaluation. No claim of a completed DerefOf operation
follows from successful parsing.

## API and results

`parse(input:&[u8;256], length:u64) -> PathResult` uses canonical `Outcome`, `Path`
and `PathResult`. Logical length excludes any String terminator; embedded NUL is
invalid. Each admitted segment is packed little-endian into a u32 and padded
with underscores to four characters. The fixed profile permits 16 segments and
16 leading parent prefixes. These are Cathedral limits, not ACPI maxima.
There is no extra work-budget parameter: every scan is bounded by 256 bytes.

Success sets `next=length`, `offset=0` and the complete canonical path. All unused
segment slots are initialized to zero. Failure publishes a default empty path
and `next=0`; it never leaks a partially parsed prefix. Outcomes are:

| Outcome | Condition and offset |
| --- | --- |
| `Truncated` | Logical length exceeds 256; offset zero. |
| `InvalidName` | Empty input; forbidden byte; invalid leading digit; segment over four characters; dotted empty segment; root/parent misuse. Offset identifies the offending byte, or logical end for a trailing dot. |
| `Capacity` | Seventeenth parent prefix, or completion of a seventeenth segment. Segment capacity is reported at its separator or logical end, after that segment's character checks. |

The parser rejects NUL, whitespace, non-ASCII bytes, interior prefixes, combined
root/parent prefixes and empty dotted components. It does not validate whether a
relative path's parents exist in a namespace. Prefixes remain ordinary path data.

`format(path:Path) -> TextResult` returns the semantic alternatives
`Text {length, bytes:[u8;256]}` or `Failure {error}`. It first validates typed local
count/parent bounds before invoking canonical `path_valid`, then rejects an
empty relative path. Malformed caller-built paths fail without a text payload.
Valid segments emit all four padded uppercase bytes. Separators appear only
between segments; prefixes are leading only. Logical length excludes a terminator
and **all remaining bytes through index 255 are zero**. Maximum output is 95 bytes:
16 parents, 16 four-byte segments and 15 separators. Absolute paths need at most
80 bytes. Existing canonical path/character helpers are reused; no new Value,
String storage or nominal NameSeg type is introduced.

The explicit numeric bounds state prevents malformed `count=u64::MAX` from
reaching a long canonical validation loop on the pinned evaluator. Helper calls
for uppercasing and final segment insertion use explicit local results to satisfy
the pinned constant evaluator's exact-owner rules. Neither change alters the
intended grammar or depends on compiler modifications.

## Pinned-source map and differences

| Pinned anchor | Bounded implementation |
| --- | --- |
| `AmlName::from_str`, line 586 | `name_text::parse`, leading-prefix grammar, fixed capacity and explicit errors. |
| `NameSeg::from_str`, line 651 | Embedded character validation and underscore padding in `text_step`; canonical u32 segment. |
| `AmlName::as_string`, line 473 | `name_text::format`, initialized output instead of allocation. |
| `NameSeg::as_str`, line 642 | Canonical segment-byte emission, without an unsafe UTF-8 borrow. |
| `is_lead_name_char` / `is_name_char`, lines 679/683 | Existing `names::lead_char` / `name_char`, preceded by ASL lowercase folding. |
| `FromStr::Err`, lines 584/649 | Existing canonical `Outcome` and `PathResult`. |
| Display/Debug `fmt`, lines 617/688 | Rust formatter integration omitted; ordinary text output is the intended interface. |

The pin accepts only uppercase segment characters. Some short invalid inputs
panic while constructing its diagnostic array. It accepts interior parent
prefixes and root-plus-parent combinations, and can normalize such values later;
the primary grammar profile rejects them at parse time. The pin rejects
parent-only names, whereas the primary grammar admits them. It has no 16-element
capacity restriction. Both representations pad short valid segments with `_`.
These differences are documented adaptations, not a hidden compatibility mode.

## Evidence and integration

The public Rust harness performs actual `AmlName::from_str`, `NameSeg::from_str`,
text output, normalization, parent and scope-resolution calls. It uses no copied
private bodies, forged object tokens or Interpreter/Handler. Each of its 84
original inputs runs in a fresh process; 29 executions catch a pin panic. A
panic may occur after `parsed` was emitted by a later observation, so the record
retains each operation instead of equating all panics with parser rejection.

The comparison classifies 26 accepted-text agreements, 40 inputs rejected by
both profiles, 13 primary acceptance corrections and five strict grammar or
capacity rejections. The prospective strict values are independently exercised
by actual Omega tests; the comparison does not pretend Rust implements the
strict profile. Host inputs are original UTF-8 strings; separate Omega cases
exercise non-UTF-8 bytes as invalid text.

The Omega suite contains 150 positives and changed-body controls, including 35
focused canonical name-guard regressions. It checks exact
parse outcomes/offsets, all 16 path slots, valid parse/format/parse round trips,
malformed ordinary formatter inputs, and all 256 formatter bytes. Direct format
cases use distinct segments to expose indexing errors. Their controls mutate only
byte 255 in the expected zero tail, so checking logical text alone cannot pass.
Three representative constant-evaluation pairs cover mixed-case multi-segment
round trips, the maximal 95-byte output and the canonical MAX-count guard. The maximum checked fuel usage was 32,630 per body. Exact counts/hashes belong
to the retained verification records; native Omega and live namespace execution
are outside this evidence.

The focused guard cases include counts and parents 17, 2^63 and u64::MAX through
path_valid/path_equal/parent/resolve and out-of-range physical segment access.
Path equality deliberately retains its prior structural semantics: equal
parent-only values with parents greater than 16 can compare equal even though
path_valid rejects them. The separately published canonical guard fix changes
expression typing, not these semantics.

The published slice contains `name_text.omg`, this report, its inventory and
the owned name-text tools. The canonical name/namespace guard correction landed
separately. Test receipts now bind only the used text-conversion import closure
and package build recipes, instead of the earlier scratch whole-package snapshot.
