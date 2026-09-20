# Fixed ACPI tables and Generic Address Structure

Current stage: tested by Omega semantic evaluation. All 291 original scenarios and all three body-mutating negative controls pass with the fresh compiler. The complete authored fixture source also checks. Production integration, native execution and native ABI comparison have not run.

## Pin and licensing

Derivative translation of `rust-osdev/acpi` 6.1.1 at `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, under MIT OR Apache-2.0. The audit and unchanged upstream notices are in [PORT.md](PORT.md) and `licenses/rust-osdev/acpi/`. No firmware captures, AML fixture bodies, or externally licensed examples enter this slice. Test bytes are original synthetic records.

## Source map

[fixed-inventory.json](fixed-inventory.json) classifies every indexed symbol in `src/address.rs` and `src/sdt/{fadt,madt,mcfg,hpet}.rs`. It maps the complete pinned data records and pure operations independently from omitted mapping, I/O, formatting, and mapped constructors. `fixed_types.omg` carries owned logical records; `fixed_decode.omg` performs field reads; the named table modules validate their framing and implement pure queries. `madt_search.omg` finds a mailbox address as an inert integer.

`fixed_generate.py` and `madt_generate.py` reproduce the generated source against exact pinned field declarations. `fixed_model.py` compiles minimal Rust declarations to measure 236 size, alignment, and offset facts. These observations describe the pinned Rust declarations, not an Omega layout, a full upstream crate build, or a firmware mapping. [fixed.vectors.json](fixed.vectors.json) retains expected wire facts; [fixed-rust-observation.json](fixed-rust-observation.json) records the source hashes and rustc1.94.0-nightly host observation on aarch64-apple-darwin. The logical Omega records deliberately contain optional-presence flags and decoded variants, so their native memory layout is not the wire layout.

## Bounds and behavior

Inputs are initialized read-only `[u8; 4096]` plus an available length. The capacity is a local resource limit, not an ACPI table size limit. SDT signature, declared length, available length, and complete declared checksum are checked before fixed-table decoding. Addresses remain full-width numeric data, including zero and high-bit values; parsing grants no mapping or access authority. Raw reserved fields remain inert data.

FADT accepts revision 1 at 116 bytes, revision 2 at 132, revisions 3/4 at 244, revision 5 at 268, and revision 6 or later at 276. These strict profile minimums come from ACPICA's historical size constants; historical specification archive downloads were unavailable. Unknown later suffixes remain checksum-covered. Extensions require both their revision threshold and actual complete bytes. A revision-only `MaybeUninit` accessor is replaced by explicit optional values. Byte presence does not prove hardware support. In particular, the pure legacy register-selection helpers preserve the pin's descriptor selection and do not activate registers or override the ACPI hardware-reduced flag. A future platform summary must apply that flag before reporting usable legacy facilities.

MADT keeps all 17 pinned entry variants, unknown entry headers, and bounded raw entry-tail access. Each nonterminal successful step advances at least 2 bytes. Zero/one-byte entries, incomplete headers, known entries shorter than their typed prefix, and records crossing the declared end fail. The profile accepts GICC 80-byte prefixes before MADT revision 6 and 82-byte prefixes at revision 6 or later; older 76-byte GICC forms are outside this profile. Local SAPIC UID tails remain raw bytes, without an unchecked string claim. Wake mailbox data is declared without volatile access, synchronization, or ownership.

MCFG framing requires whole 16-byte records unless `allow_partial_tail` is explicitly true; that pin compatibility quirk reports discarded byte count. Each indexed record rejects an inverted bus interval. The parser does not reject multiple disjoint ranges in one segment. It does not compute an ECAM access or prove address usability. HPET validates its 56-byte prefix and revision 1, preserving unknown/reserved descriptive bits without a hardware compatibility claim.

## Primary facts and deviations

The [ACPI 6.6 software programming model](https://uefi.org/specs/ACPI/6.6/05_ACPI_Software_Programming_Model.html), particularly GAS §5.2.3.2, FADT §5.2.9, and MADT §5.2.12, supplies serialized format facts. [ACPICA's primary table header](https://raw.githubusercontent.com/acpica/acpica/master/source/include/actbl.h) supplies historical FADT size constants. [Intel HPET 1.0a](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/software-developers-hpet-spec-1-0a.pdf) supplies HPET §2.3.4 and§3.2.4. [Intel's EDK II MCFG declaration](https://github.com/tianocore/edk2/blob/master/MdePkg/Include/IndustryStandard/MemoryMappedConfigurationSpaceAccessTable.h) cross-checks MCFG byte offsets; [PCI-SIG's multiple-base-address ECN](https://pcisig.com/PCIFirmware/ECN/Firmware/EnablingMultipleBaseAddressesperPCISegmentGroup) permits disjoint ranges in a segment. Only numeric specification facts were taken from these additional sources.

- GAS adds primary-defined PRM address space 11 and permits OEM spaces 128–255; the pin starts OEM acceptance at 192. Unknown 12–126 still fail semantic classification but can be preserved by raw decoding.
- GAS and flag wrapper identities use explicit u8/u16/u32 carriers and named constants/getters. This is a documented type-identity deviation, not a new authority type.
- GAS width selection preserves the pin's heuristic, including native-width precedence and64-bit clamping. Addition widens before summing offset and width, avoiding pinned u8 overflow. The result is not a complete legal register-access plan.
- HPET's encoded comparator field is the last comparator index; reported count is therefore encoded + 1. The pin reports the encoded number directly. Both count and encoded field are retained here.
- Legacy register lengths above 31 bytes return an error instead of overflowing an 8-bit width. Invalid wake commands return an error instead of panicking.
- Bounded MADT framing replaces unchecked packed casts and borrowed iterators. Unknown entries remain observable rather than silently disappearing; malformed nonprogressing entries fail.
- Named MADT payload fields are distinct because this Omega checker confuses different variant payload types when they reuse the name `value`. This changes member spelling only.
- No Rust reference, `Pin`, flexible trailing slice, `usize` physical address cast, or mapped-table constructor is translated into fabricated Omega authority. `HpetInfo::new` remains omitted as a whole mapped lookup; its pure extraction is separately implemented by `hpet_info`.

## Errors

The explicit u8 error carrier uses 0 success; inherited header errors 1 capacity,2 truncation,3 signature,4 revision,5 declared/profile length,6 checksum;9 partial record;10 index/cursor;11 GAS/address/access-width classification;12 absent required table address;13 inverted bus interval;14 invalid MADT entry length;15 interrupt polarity;16 trigger mode;17 wake command;18 absent wake mailbox. Parser result APIs use these explicit numeric carriers. The original MadtError variants also remain as an inert enum, including the unused wake-timeout case. Neither form performs firmware recovery.

## Tests and commands

The five sliced upstream files contain no `#[test]` functions. The 291 original scenarios in `tools/ports/acpi/fixed-cases.json` cover raw fields, revision/length boundaries, address selection, flags, GAS spaces/access widths, HPET extraction, MCFG tails, MADT variants and malformed lengths, interrupt flags, and wake commands. The checker compiles every authored case and evaluates bounded groups using actual Omega machine bodies. Body-mutating negative controls change a decoded GAS address, HPET comparator count, and malformed MADT expectation; each must produce the computed `1 == 0` failure. These are semantic evaluation tests, not native execution or hardware access. Verification ran the original 237 cases, 43 additional helper/enum/mailbox cases, and 11 FADT wrapper-routing cases in bounded batches; the published runner reproduces the complete 291-case corpus.

Compiler: clean Omega source `eaa7993a23623cd8fabf45350340479c5c9c7879`; binary `/tmp/cathedral-omega-eaa7993/release/omega`, SHA-256 `2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.

```sh
python3 tools/ports/acpi/fixed_check.py --omega /tmp/cathedral-omega-eaa7993/release/omega
python3 tools/ports/acpi/fixed_evidence.py --check
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/acpi --require-transcribed source/libraries/acpi/fixed-inventory.json
python3 tools/ports/vectors.py source/libraries/acpi/fixed.vectors.json
```

No boot path imports this package. Physical mapping, register I/O, native overlays, interrupt routing activation, ECAM access, AML execution, and firmware-controlled memory ownership remain separate work.
