# HII raw protocol and IFR corpus

## Scope and status

UEFI-007, reviewed 2026-09-20 against `uefi-rs`
`c0facddf9ba42b74906a37fca2869e6cdbc8da6a` and Omega
`eaa7993a23623cd8fabf45350340479c5c9c7879`.

Overall **inventoried with explicit overlay/runtime-tail blockers**. All fixed
representable declarations, inert backing carriers, constants, and fixed policies
pass Omega source checking. Pure selectors/header helpers execute successfully
in semantic-evaluator fixtures, with an expected-failure negative control.
No typed union, runtime tail, native layout, firmware invocation or Cathedral
configuration integration is claimed. All nine pinned `protocol/hii/*.rs` files
are mapped; no file or lexical anchor remains pending.

## Upstream pin and licensing

[uefi-rs](https://github.com/rust-osdev/uefi-rs), crate `uefi-raw` 0.16.0; exact
revision `c0facddf9ba42b74906a37fca2869e6cdbc8da6a` in the optional root-relative
`reference_code/rust-osdev/uefi-rs` checkout. Retain **MIT OR Apache-2.0**,
verified in root/crate manifests and each upstream file's SPDX header.
Modified Omega/probe files carry attribution headers.

[MIT](../../../../licenses/rust-osdev/uefi-rs/uefi-raw/LICENSE-MIT),
[Apache-2.0](../../../../licenses/rust-osdev/uefi-rs/uefi-raw/LICENSE-APACHE),
[root notices](../../../../THIRD_PARTY_NOTICES.md),
[separate pin-change audit](../../../../licenses/rust-osdev/README.md).
Rust stays in the ignored reading room; no Rust vendor tree enters Cathedral.

## Source and public-symbol map

[hii-inventory.json](hii-inventory.json) binds every source hash and anchor:
nine files; 828 translated, 131 blocked, 13 omitted, zero pending.
Size-assertion macro invocation rows are not declarations in the lexical index;
the target probe still checks every recorded size.
[The schema](../../../../tools/ports/uefi-hii/schema.json) supplements the lexical
inventory with every complete field mapping. The corpus has 185 inert carriers,
391 named constants, and 185 authored fixed policies in
[hii_layouts.omg](hii_layouts.omg).

| Upstream under `uefi-raw/src/protocol/hii` | Cathedral module | Coverage |
| --- | --- | --- |
| `mod.rs` | [hii.omg](hii.omg) | Handles/IDs, package headers, keys/modifiers, date/time/ref; header tail separate. |
| `config.rs` | [hii_config.omg](hii_config.omg) | Keyword handler, configuration access/routing, action/error constants, GUIDs. |
| `database.rs` | [hii_database.omg](hii_database.omg) | Keyboard prefix, notify carrier/flags, complete database protocol/GUID. |
| `font.rs` | [hii_font.omg](hii_font.omg) | Glyph/font/display/row facts, flags, both protocol revisions/GUIDs; name tail separate. |
| `form_browser.rs` | [hii_form_browser.omg](hii_form_browser.omg) | Screen descriptor, action requests, complete FormBrowser2 protocol/GUID. |
| `ifr.rs` | [hii_ifr.omg](hii_ifr.omg) | Every opcode/flag/value and fixed opcode record; union-bearing values explicitly named WireStorage. |
| `image.rs` | [hii_image.omg](hii_image.omg) | Input/output facts, draw flags, both protocols/GUIDs; output union remains untyped storage. |
| `popup.rs` | [hii_popup.omg](hii_popup.omg) | Style/type/selection, revision, protocol/GUID. |
| `string.rs` | [hii_string.omg](hii_string.omg) | Complete string protocol/GUID. |
| Four selector methods from `ifr.rs` | [hii_helpers.omg](../../../libraries/uefi/hii_helpers.omg) | Date/time storage and numeric size/display extraction, retaining reserved values. |

Upstream compile-time assertions map to vectors. `Debug` formatting is omitted
as Rust tooling, not a firmware operation. Derived traits are not promised Omega
APIs. Original function-pointer signatures are preserved beside inert `addr`
slots, including IN/OUT pointer shape; none becomes a callable service.

## Primary specifications and representation vectors

Governing contracts are UEFI [§33, HII packages and IFR](https://uefi.org/specs/UEFI/2.10/33_Human_Interface_Infrastructure.html),
[§34, HII protocols](https://uefi.org/specs/UEFI/2.11/34_HII_Protocols.html), and
[§35, configuration/browser protocols](https://uefi.org/specs/UEFI/2.10/35_HII_Configuration_Processing_and_Browser_Protocol.html).
The package header in §33.3.1.1 packs a 24-bit length and 8-bit type; IFR's
opcode header carries a 7-bit length and scope bit. Pure helpers extract those
fields without gaining any service or configuration authority.

[hii.vectors.json](hii.vectors.json) uses `cathedral-port-vectors-v1`: 1131
size/alignment/offset/value/GUID-byte expectations. They were measured from the
pinned Rust types on `aarch64-apple-darwin`, then every numeric fact and GUID byte
was asserted by Rust compile-time evaluation for `x86_64-unknown-uefi`.
`measure.py` enforces identical vector/assertion coverage; it never labels this
an Omega observation. Target: little endian, 64-bit pointers and UINTN.

The checker separately compares every source field and constant and every
fixed policy entry with those vectors. Union storage bytes intentionally have
no union member projections; their explicit policy carries measured alignment.
Typed union validity is not part of this check.

Not every zero-array tail begins at `sizeof(prefix)`: `FontInfo.font_name`
begins at offset 6 while Rust's fixed record size is 8 due to final padding.
Vectors preserve the actual tail offset. Neither an array of fixed prefixes nor
a `sizeof` step describes variable records.

## Translated tests and fixtures

Pinned HII files have compile-time ABI assertions but no `#[test]` functions.
An authored Rust test checks all 256 inputs for each of the four upstream flag
selectors. Omega `main.omg` calls the real translated helpers in a compile-time
constant, then requires the result to equal zero; `negative.omg` adds one and
must reject. The fixture exercises ordinary selectors, reserved mask values,
zero/minimum/maximum package and opcode lengths, truncation, scope bits and
maximum available extent. No test reads an unadmitted pointer or union member.

Header bounds checks are locally authored, narrowly specified predicates.
They do not validate entire package sequences, IFR scope nesting, opcode payload,
configuration strings, fonts/images, language encodings or firmware state.

## Omega blockers and boundary seams

- `omega:programmable-overlays`: Omega's
  [layout contract](../../../../../Omega/wiki/spec/layouts/plans.md#placement-vocabulary)
  leaves source forms for programmable unions unspecified. The four upstream
  unions are `IfrTypeValue` (22 bytes/alignment 1), `VarstoreInfo` (2/1),
  `IfrNumericData` (24/1), and `ImageOutputDest` (8/8). Every member remains in
  source comments and vectors; no Omega type with the upstream union name is
  fabricated. `*WireStorage` types hold backing bytes only, and the suffix is
  propagated to 25 union-dependent carriers (including the four unions).
  Typed union construction/projection, including its typed default, is blocked.
- `omega:runtime-layout-strides`: eight tails require checked runtime extent and
  element-stride forms: `HiiPackageHeader.data`, `HiiKeyboardLayout.descriptors`,
  `FontInfo.font_name`, `IfrEqIdValList.value_list`, `IfrFormMap.methods`,
  `IfrFormSet.class_guid`, `IfrVarstore.name`, `IfrVarstoreEfi.name`. Fixed fields,
  tail offsets, and pure bounded byte/header work remain useful independently.

Both seams carry `PORT-BLOCKED[...]` markers in source and exact inventory rows.
Fixed packed geometry is authored ordinary policy work; it is not falsely
classified as blocked by the absence of a compiler feature.

Selecting all 185 fixed layouts together passes source checking (24 sources).
The separate HiiDate field-projection fixture fails with `selects private data
HiiDateX64Layout<HiiDate>::year`: `PORT-BLOCKED[omega:plan-laid-public-fields]`.
This is a compiler visibility limitation on synthesized fields, distinct from
the successful layout selection and pure-helper semantic tests.

Firmware methods need explicit live-provider, buffer-bound, alias, allocation,
retention and callback contracts. Database notification registration retains
foreign callback/context obligations; configuration callbacks may mutate
platform state. Parsing tags or finding a GUID establishes none of that authority.

## Deliberate deviations and primary-source disagreements

- Pointer/callback/handle slots use inert `addr`; ID, Boolean and character
  fields use their raw widths. Open enum/flag wrappers preserve unknown values.
- Three Rust `r#type` fields become `type_code`; schema and vectors retain their
  exact source correspondence. Rust modules become explicit `hii_*` modules.
- Typed union-bearing records become visibly named `WireStorage` records with
  `*_storage` fields. They are not valid typed union instances and do not select
  an active member by themselves. Packed records require the selected explicit
  layout; their plain default Omega layout is not asserted.
- Upstream corrects `HiiFormPackageHdr.header` to an inline package header where
  its specification comment identifies a mistaken pointer. This port preserves
  the pinned correction and four-byte fixed representation.
- Upstream notes HiiImageEx `new_image_ex.image` is IN OUT in UEFI 2.11 but uses
  a const pointer following edk2/non-extended behavior. The inert slot preserves
  upstream signature documentation; no foreign-write permission is inferred.
- Upstream HiiFontEx `get_glyph_ex.baseline` is a pointer while the cited 2.11
  prototype writes it by value and refers to the pointer-based GetGlyph meaning.
  Its comment also notes HiiFontEx is not defined in edk2. These are recorded
  upstream/spec differences awaiting full primary resolution before live calls.

## Cathedral integration and authority

The [contracts charter](../../CHARTER.md) owns this raw ABI corpus. Pure machines
live in `source/libraries/uefi`; the test application is isolated under tools.
No production boot root gains a dependency or provider. No HII UI, configuration
mutation, allocation, callback, image rendering or firmware execution is added.
The existing complete console pixel type is reused, avoiding a competing shape.

## Verification commands and results

Run from Cathedral root; Rust `1.94.0-nightly (f6a07efc8 2026-01-16)` and the
Omega release binary from the revision above:

| Command | Result and exact scope |
| --- | --- |
| `python3 tools/ports/inventory.py check source/contracts/uefi/raw/hii-inventory.json --checkout reference_code/rust-osdev/uefi-rs` | Nine files, no pending anchors; explicit blocked/omitted rows above. |
| `python3 tools/ports/uefi-hii/measure.py` | 1131 upstream measurements, every fact asserted for UEFI x64. |
| `python3 tools/ports/uefi-hii/check_transcription.py` | 185 inert carriers/fixed plans and 391 constants agree with upstream target facts. |
| `python3 tools/ports/vectors.py source/contracts/uefi/raw/hii.vectors.json` | Deterministic vector format validation; no Omega observation. |
| `cargo test --locked --manifest-path tools/ports/uefi-hii/Cargo.toml` | Exhaustive upstream selector test passes; not translated-code execution. |
| `python3 tools/ports/uefi-hii/check_omega.py` | All nine raw modules, hii_layouts and helper source pass; positive semantic fixture passes; negative control rejects computed `0 + 1 == 0`. |
| `../Omega/target/release/omega --check tools/ports/uefi-hii/layout_probe.omg` | All 185 selected fixed layouts pass (24 sources); no projection/execution claim. |
| `../Omega/target/release/omega --check tools/ports/uefi-hii/layout_projection.omg` | Expected compiler failure: selected HiiDate layout exposes private synthesized `year`. |
| Native/firmware/production canaries | Not run; no production reach or live adapter change. |

A missing/mutated upstream checkout fails clearly. No successful manifest or Rust
probe promotes blocked typed unions/tails to a complete translation.
