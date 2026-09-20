# Third-party notices

Cathedral preserves `MIT OR Apache-2.0` for translations of the seven pinned
rust-osdev projects below. Either license may be selected by recipients. This
record describes the upstream grants and the port lane; it does not assert
that every listed project has already been translated. Each package's `PORT.md`
identifies the files actually derived from upstream, their modifications, and
verification state. Independent Cathedral code and primary-specification facts
remain distinguished from those translations.

The exact license files, including their original copyright notices and
formatting, are retained under [licenses/rust-osdev](licenses/rust-osdev/).
[sources.json](licenses/rust-osdev/sources.json) records every upstream license
path, pinned revision, copied-file SHA-256, and the Cargo/README evidence hashes.
No Rust source vendor tree is included.

## Audited pins and retained licensing

| Project and exact revision | Upstream declaration and evidence | Preserved choice and local texts |
|---|---|---|
| [uefi-rs](https://github.com/rust-osdev/uefi-rs/tree/c0facddf9ba42b74906a37fca2869e6cdbc8da6a), `c0facddf9ba42b74906a37fca2869e6cdbc8da6a` | Root `Cargo.toml` workspace: `MIT OR Apache-2.0`; `uefi-raw/Cargo.toml` inherits it. Root and crate `LICENSE-MIT`, `LICENSE-APACHE` agree. | `MIT OR Apache-2.0`: [MIT](licenses/rust-osdev/uefi-rs/LICENSE-MIT), [Apache-2.0](licenses/rust-osdev/uefi-rs/LICENSE-APACHE); crate copies retained too. |
| [x86_64](https://github.com/rust-osdev/x86_64/tree/cc35c876d3badb57df54a66e22f7768a52be95f2), `cc35c876d3badb57df54a66e22f7768a52be95f2` | `Cargo.toml`: `MIT/Apache-2.0`; both root license files. | `MIT OR Apache-2.0`: [MIT](licenses/rust-osdev/x86_64/LICENSE-MIT), [Apache-2.0](licenses/rust-osdev/x86_64/LICENSE-APACHE). |
| [acpi](https://github.com/rust-osdev/acpi/tree/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5), `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5` | `Cargo.toml`: `MIT/Apache-2.0`; README dual-license statement; root files use British spelling, `LICENCE-MIT`, `LICENCE-APACHE`. | `MIT OR Apache-2.0`: [MIT](licenses/rust-osdev/acpi/LICENCE-MIT), [Apache-2.0](licenses/rust-osdev/acpi/LICENCE-APACHE). |
| [pci_types](https://github.com/rust-osdev/pci_types/tree/eff856b6bab81adbe1bda0ab750009f3786cb8d3), `eff856b6bab81adbe1bda0ab750009f3786cb8d3` | `Cargo.toml`: `MIT/Apache-2.0`; README dual-license statement; actual root filenames are `LICENSE-MIT`, `LICENSE-APACHE` despite README links spelling them `LICENCE`. | `MIT OR Apache-2.0`: [MIT](licenses/rust-osdev/pci_types/LICENSE-MIT), [Apache-2.0](licenses/rust-osdev/pci_types/LICENSE-APACHE). |
| [virtio-spec-rs](https://github.com/rust-osdev/virtio-spec-rs/tree/ad565bd701e93fa47bc288d45701e1ae53ac3c12), `ad565bd701e93fa47bc288d45701e1ae53ac3c12` | `Cargo.toml`: `MIT OR Apache-2.0`; README and both root license files. | `MIT OR Apache-2.0`: [MIT](licenses/rust-osdev/virtio-spec-rs/LICENSE-MIT), [Apache-2.0](licenses/rust-osdev/virtio-spec-rs/LICENSE-APACHE). |
| [uart_16550](https://github.com/rust-osdev/uart_16550/tree/653455f17e92c67a67e44b4c4b6eaba22411e57b), `653455f17e92c67a67e44b4c4b6eaba22411e57b` | `Cargo.toml`: `MIT OR Apache-2.0`; README and both root license files. | `MIT OR Apache-2.0`: [MIT](licenses/rust-osdev/uart_16550/LICENSE-MIT), [Apache-2.0](licenses/rust-osdev/uart_16550/LICENSE-APACHE). |
| [pic8259](https://github.com/rust-osdev/pic8259/tree/136052bcbf081b9382fc3d72ba04b83b30f664b4), `136052bcbf081b9382fc3d72ba04b83b30f664b4` | `Cargo.toml`: `Apache-2.0/MIT`; README explicitly offers either at the recipient's option. No license text files in the pin; see [retained licensing and attribution excerpts](licenses/rust-osdev/pic8259/UPSTREAM-NOTICES.md). | `MIT OR Apache-2.0`: supplied [MIT](licenses/rust-osdev/pic8259/LICENSE-MIT), [Apache-2.0](licenses/rust-osdev/pic8259/LICENSE-APACHE) terms; their distinct provenance is recorded. |

The slash expressions above are retained as upstream evidence; the port records
use the explicit `OR` spelling for the offered license choice. No project-level
contradiction to the queue's dual-license policy was found at these pins.

## Attribution and source-specific review

- uefi-rs: `Copyright (c) The uefi-rs contributors`.
- x86_64: `Copyright (c) 2018 Philipp Oppermann`, `Copyright (c) 2015 Gerd
  Zellweger`, and `Copyright (c) 2015 The libcpu Developers`. The upstream
  [AUTHORS](licenses/rust-osdev/x86_64/AUTHORS) file and the additional
  [interrupt descriptor source notice](licenses/rust-osdev/x86_64/SOURCE-NOTICES.md)
  are retained.
- acpi and pci_types: `Copyright (c) 2018 Isaac Woods`.
- uart_16550: `Copyright (c) 2025 Philipp Schuster`.
- virtio-spec-rs's MIT file contains no copyright header; it is preserved as
  supplied, without inventing a holder or year.
- pic8259's manifest names Eric Kidd as author; its README credits the
  `pic8259_simple` fork and OSDev Wiki PIC notes. Those attributions are retained
  in its [notice record](licenses/rust-osdev/pic8259/UPSTREAM-NOTICES.md). Its
  pinned files contain no explicit copyright line.

No file named `NOTICE` was found in any of the seven tracked upstream trees.
That does not remove the need to retain applicable notices embedded in files.
In particular, acpi's `tests/pc-bios_acpi-dsdt.asl` contains an Intel ACPI
Component Architecture disassembler header, including `Copyright (c) 2000 -
2018 Intel Corporation`, and identifies a disassembled firmware table. This
setup does not import that fixture or establish its original firmware's license.
Audit the ASL/AML fixture's own origin and terms before copying or translating
it; the crate-level declaration alone is not a provenance record for firmware
captures. Record a deliberate omission if that evidence is unavailable.

These records cover the named upstream projects, not all their dependencies or
external assets. A translation that derives dependency code, generated tables,
third-party test inputs, or documentation must inventory its actual sources and
retain their applicable notices separately.

## Modification provenance

Every substantially translated source file must carry a prominent notice such
as the following, adjusted to the actual upstream path and pin:

```text
SPDX-License-Identifier: MIT OR Apache-2.0
Derived from <upstream URL>, revision <exact pin>, <upstream path>.
Modified for Cathedral: translated from Rust to Omega; <actual deviations>.
See PORT.md and THIRD_PARTY_NOTICES.md for provenance and license texts.
```

Use the target format's comment syntax. For fixtures without comments, keep the
notice in their adjacent metadata and source map. Retain applicable upstream
copyright and attribution text in the translated file; this central record does
not replace a source-specific notice. `PORT.md` records the source-to-target
mapping, omitted code, changed representations, authority seams, and tests. A
translation is a modification, even when its algorithm is unchanged.

## Auditing a later pin update

1. Treat the new revision as a separate reviewed change. Fetch it into the
   gitignored reading room and verify its full commit ID; never update a pin
   incidentally while translating.
2. Compare old and new Cargo manifests, workspace inheritance, README grants,
   all `LICENSE*`, `LICENCE*`, `COPYING*`, `NOTICE*`, author lists, and per-file
   copyright/SPDX headers across the entire claimed slice. Inspect renamed and
   newly added files, generated inputs, dependencies actually copied, and test
   fixtures. Record any change in terms or ownership instead of assuming the
   existing project-level grant covers it.
3. Retain the exact applicable license and notice bytes. Keep old records where
   any checked-in translation still derives from the old revision. Add the new
   URL, pin, upstream paths, and SHA-256 evidence to `sources.json`, and explain
   missing upstream text files or supplied standard texts explicitly.
4. Update this table, affected `PORT.md` records, source maps, modified-file
   notices, inventory manifests, and test/vector provenance together. Separate
   newly translated behavior from a metadata-only pin update.
5. Recheck copied files against `git show <pin>:<path>` and verify local hashes;
   rerun the slice's inventory and available conformance checks. Record tests
   still blocked by Omega without marking them passed. Review and commit the
   complete audit with the pin change.
