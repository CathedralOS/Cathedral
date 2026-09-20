# pci_types translation — PCI-000

**Status: tested** (pure semantic evaluation and pinned Rust witnesses).

Source: [rust-osdev/pci_types](https://github.com/rust-osdev/pci_types/tree/eff856b6bab81adbe1bda0ab750009f3786cb8d3),
commit `eff856b6bab81adbe1bda0ab750009f3786cb8d3` (0.10.1), all six Rust source
files. Modified Omega translation under MIT OR Apache-2.0. The original notices
and exact license texts are preserved in
[THIRD_PARTY_NOTICES.md](../../../THIRD_PARTY_NOTICES.md) and
[licenses/rust-osdev/pci_types](../../../licenses/rust-osdev/pci_types).
The pinned crate has no authored `#[test]` cases; Cathedral supplies behavioral
fixtures and a Rust witness program instead of claiming to port nonexistent tests.

## Representation and authority

The package is `cathedral-pci`. `address` packs segment/bus/device/function into
a raw u32. `snapshot` decodes caller-supplied little-endian bytes (up to 4096)
with explicit extent checks. `headers` retains common, type-0 endpoint, and
type-1 bridge fields. `registers` retains all status/command bits.
`device_type` retains every upstream classification name and USB interface code;
unknown classification is 65536, while raw class/interface bytes remain in the
header. Class names alone do not validate a device's programming interface.

`bars` separates raw encoding, supplied probe response sizing, lookup, and
operation planning. `capabilities` returns one finite step per call, preserving
unknown IDs and reporting null entries as skipped; neither caller nor malformed
input can make an individual step loop. Conventional cursors accept aligned
0x40..0xfc entries, at most 48 visits. Extended cursors start at 0x100 and accept
aligned 0x100..0xffc entries, at most 960 visits. A 1024-slot visited bitmap
rejects cycles. Invalid pointers, exhausted budgets, short snapshots, and repeat
visits permanently terminate with malformed status. Null next pointers end;
zero/all-ones extended headers end. These are explicit parser policy choices.
Extended header id/version/next facts are a Cathedral extension beyond the pin.

Raw wrappers and parse results are ordinary Omega data, not C ABI declarations.
A raw unknown value is not an authority or validated device handle. Check `ok`
before using a result; failed records can retain diagnostic raw bits. Snapshot
BDF and `HeaderFacts.address` identify supplied data without proving freshness.
The type-2 CardBus header kind survives as 2; no CardBus body exists upstream,
so a CardBus body parser is outside this port's scope.

`ConfigRegionAccess`, including its reference implementation, is replaced by
`ConfigSnapshot`, bounded `snapshot_read`, and `ConfigOperation`/`ConfigPlan`.
This is an explicit translation to inert read/write descriptions, **not** an
implemented configuration accessor. Plans do not carry a BDF; the resource owner
must bind them to the same target whose snapshot/original words were validated.
There is no indirect callback, pointer dereference, assembly, or production
import. Actual ECAM/CF8/CFC access, transaction ordering, locking, freshness,
completion/failure semantics, ownership, DMA and interrupt allocation remain
unresolved authority contracts outside PCI-000's pure corpus.

Read operations have kind 1, writes kind 2; width is bytes, offset is bytes, and
value is the host integer representing the little-endian register. Read results
are returned in operation order. A plan's count is at most eight. All execution
must be serialized by its future owner. BAR probing additionally requires
quiesced decoding and exact restoration on *every* failure path before decoding
resumes. BAR writes require quiesced decoding. Message-address/data plans require
MSI interrupts disabled; no plan allocates vectors or validates a platform's
interrupt destination. Raw control write plans describe writes; they do not
assert that a caller's arbitrary control word is operationally valid.

## Deliberate changes from the pin

| Pinned behavior | Translated behavior and evidence |
|---|---|
| MSI MME getter/setter uses DWORD bits 6:4, overwriting the capability ID. | Helpers use control-word bits 6:4 and a 16-bit write at capability+2. A Rust witness observes `01860005 -> 01860025`; the Omega fixture requires control `01a6`. Intel's [MSI control register](https://edc.intel.com/content/www/jp/ja/design/publications/core-ultra-processors-for-edge-ps-series-ioe-p-i-o-registers/001/message-signaled-interrupt-message-control-msi-mctl-offset-82/) defines MME/ MMC fields. |
| Invalid MMC silently becomes Int1 and requests above MMC silently clamp. | Preserve the raw encoding and reject MMC>5, MME>MMC, or an excessive requested MME. This is checked-helper policy, not a claim of identical error behavior. |
| BAR probing restores a masked address, dropping original low flags; zero masks can lead to shifts by the carrier width. | Restore both exact original words, preserve flags, reject zero/ambiguous or noncontiguous sizing masks, and calculate in u64 without width-sized shifts. A Rust witness observes `80000008 -> 80000000` restoration. BAR sizing rules are in PCI Local Bus 3.0 §6.2.5.1, [PCI-SIG specification hosted by TI](https://e2e.ti.com/cfs-file.ashx/__key/communityserver-discussions-components-files/639/1016.PCI.Local.Bus.Specification.Revision.3.0.pdf). |
| `bar` returns absent for a zero current address and can panic on reserved encoding. | A zero address remains a valid *encoding*, not evidence of implementation. Supplied probe masks establish supported sizes. Reserved and obsolete below-1MiB encodings return `ok=false`. `endpoint_bar` rejects the upper slot of a 64-bit BAR and a 64-bit low word in slot5. Raw `bar_decode`/plan builders require the caller to identify a low slot, normally through `endpoint_bar`. |
| `write_bar` trusts alignment and truncates low flags. `unwrap_mem`/`unwrap_io` panic on wrong kind. | Plans reject misalignment/32-bit overflow and preserve flags. Checked projections return `ok`; memory projection accepts a separate validated size result and retains address, size, prefetchability. BAR-size alignment and resource allocation are still the owner’s responsibility. |
| Capability iterator accepts malformed pointers and can revisit cycles forever (including an infinite null-only loop). | Bounded, cycle-detecting snapshot cursor with explicit skipped/end/malformed results. Rust witness samples the pin's same self-cycle four times; Omega rejects the second visit. PCIe [extended-header bit fields](https://www.intel.com/content/www/us/en/docs/programmable/683140/22-4-8-0-0/tph-requester-enhanced-capability-header.html) define the added id/version/next extraction. |
| Command update writes a DWORD echoing status bits, some RW1C. | Emit only a 16-bit command write at 4. Rust witness records the pin's `80100005` write. Register widths are retained in upstream header documentation and [PCI standard register definitions](https://raw.githubusercontent.com/torvalds/linux/master/include/uapi/linux/pci_regs.h). |
| Interrupt update rewrites pin and line. | Permit an unchanged observed pin and emit a line-byte write; reject requests to change the pin. Generic pin routing/configuration is outside this library; [Intel endpoint interrupt properties](https://edc.intel.com/content/www/xl/es/design/publications/13th-generation-core-processor-datasheet-volume-2-of-2/interrupt-properties-intr-offset-3c/) illustrate RO pin/RW line. Device-specific writable pin exceptions require a separate owner contract. |
| MSI masking works only for 64-bit MSI; pending reads ignore the masking flag. Data writes cover a DWORD. | Use 32-bit mask/pending offsets +12/+16 or 64-bit +16/+20 only with PVM, and 16-bit message data. The [standard register definitions](https://raw.githubusercontent.com/torvalds/linux/master/include/uapi/linux/pci_regs.h) independently corroborate the numeric layouts. No Linux implementation code is translated or copied. |
| MSI-X exposes raw BIR/offset without BAR bounds validation. | Preserve facts and add BIR<6, table size1..2048, 16-byte entries, 64-vector/8-byte PBA blocks, and widened region bounds checks. No MMIO table mapping is supplied. |
| OtherMultimedia is mapped at0403, OtherMemory at0502; WorldFIP enum has no mapping; satellite TV/audio/voice/data use0f00..03. | Correct Other subclasses to0480/0580, add0205, correct satellite to0f01..04. 0403/0502 retain raw facts and are Unknown in this pin-sized taxonomy. PCI-SIG [Code and ID1.11 §§1.5–1.6](https://pcisig.com/sites/default/files/files/PCI_Code-ID_r_1_11__v24_Jan_2019.pdf), PCI Local Bus3.0 appendixD.16, and maintained [PCI ID class data](https://raw.githubusercontent.com/pciutils/pciids/master/pci.ids) support these numeric facts. Rust witnesses preserve the original wrong cases. |

Rust formatting/debug/error prose, derive machinery, generic closure syntax,
reexports and reference-accessor forwarding are not reproduced as language
machinery. Their semantic values/operations are mapped explicitly in
`inventory.json`; formatting entries are deliberately omitted. Rust scalar type
aliases become explicitly named header fields of the same width, because these
aliases carry no distinct representation or authority. All original command
bits, including unknown u16 bits admitted by the unnamed all-bits flag, survive.

## Verification and update audit

`tools/ports/pci-types/check.py` checks exact Git pin, all six source hashes,
and every lexical inventory anchor, then runs the pinned Rust witness harness
with Cargo.lock. `semantic-check.py` checks the actual Omega package and runs
10 authored behavior groups, then changes one expected behavior in each group
and requires the computed result to become1 and fail `requires value == 0`.
These controls test evaluation of fixture bodies, not merely the final assertion.

Validated with clean Omega `eaa7993`, binary SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
This is semantic evaluation of pure behavior. Native execution, foreign-layout
comparison, physical configuration access, DMA, and interrupt behavior are not
claimed. All Omega source passes current checking; no compiler/design blocker
is being substituted for unimplemented pure work.

On a pin update, regenerate the inventory snapshot, compare all six files and
Cargo.lock, re-audit every deliberate correction against the new source, rerun
Rust witnesses (upstream fixes should make old-defect witnesses fail), and rerun
all Omega positive/negative groups. Do not silently turn a newly fixed upstream
bug witness into a corrected-behavior witness without reviewing the port mapping.
