# Detached tables and captured translation

This X86-001/002 slice extends the [PTE codecs](page-entries.PORT.md) over the
existing canonical entry schema. `page_tables.omg` provides initialized 512-entry
array construction, checked word reads/writes, complete zeroing and zero-word
scanning. `translation.omg` provides numeric translation of captured entry words
and indexed reads from four borrowed table snapshots. Existing core table/walk
candidate types and admission policy remain canonical and unchanged.

Modified algorithms come from x86_64 `cc35c876d3badb57df54a66e22f7768a52be95f2`,
`page_table.rs`, `mapper/mod.rs` and `mapper/mapped_page_table.rs`. Retained
[MIT OR Apache-2.0 notices](../../../licenses/rust-osdev/x86_64/) apply.
Omega ownership, case-data and bounded-ranking contracts, the existing
[reconciliation](RECONCILIATION.md), pinned implementation and Intel's
[SDM Volume 3A](https://cdrdv2-public.intel.com/874249/253668-090-sdm-vol-3a.pdf)
chapters 4–5 were consulted before implementation.

## Data and ownership

The generic helpers borrow `[X86PageTableEntry; 512]`; there is no competing
PageTable record or dependency on privileged core. Callers own the array and any
index progression. Checked reads return an encoded numeric word; writes replace
one owned semantic element. Invalid indices fail without changing storage. Empty
means every complete word is zero, including non-present software metadata.
The scanner reaches the last entry, and zeroing visits all 512 entries under a
bounded ranking. These are ordinary initialized arrays, not placed 4096-aligned
hardware pages or reconstructed references to physical frames.

`walk_captured` computes the four indices from a canonical 48-bit virtual address,
reads the corresponding initialized snapshots, and checks supplied next-table
identities against parent-link address bits whenever traversal needs that table.
It reports a semantic `MismatchedTable` case for a mismatch. All four snapshots
must exist as ordinary inputs, even when a huge-page result stops traversal
early. Reading extra snapshots has no hardware effects. Numeric identity equality
does not prove the snapshots came from those physical addresses or remain current.

`Translation` has semantic cases for not mapped, invalid virtual address,
invalid root huge-page flag and mapped geometry. The mapped case carries frame
base, size, offset and leaf flags. No sum layout is asserted to be a hardware ABI.
`translated_address` revalidates size, frame alignment/physical envelope and
offset before combining a caller-supplied mapped case; forged numeric case
payloads therefore cannot bypass those geometry checks.

## Preserved pin behavior

The numeric algorithm preserves two upstream distinctions that must not become
hardware-admission claims:

- Parent links require PRESENT and reject/handle HUGE_PAGE in level order.
  A root huge-page flag becomes an explicit error instead of a Rust panic.
  At the PT leaf the pin checks the entire word for zero, rather than PRESENT;
  nonzero non-present software metadata can therefore produce a numeric result.
- Huge-page frames use `containing_address`, truncating low address bits to the
  2 MiB/1 GiB boundary. Returned flags are only the leaf entry's flags. Parent
  permissions are not folded in, and reserved bits, PAT interpretation,
  processor features and actual memory ownership are not validated.

All those distinctions have actual pinned mapper witnesses. Cathedral's stricter
core role/admission validators remain separate; this package does not install
or approve a mapping. The default physical mask makes the pin's 4 KiB
InvalidFrameAddress branch unreachable after address extraction; no invented
reachable failure is presented as a tested case. Encryption and LA57 profiles
remain outside this slice.

## Evidence and pending work

[tables-inventory.json](tables-inventory.json) binds all three source files,
173 lexical anchors: 75 translated, 18 deliberate omissions, 80 pending mapper
interfaces/operations. It supersedes the 16 full-table pending anchors in the
earlier entry-only inventory. Generic map/unmap/update/cleanup algorithms,
recursive/offset adapters and their ownership/flush integration remain work;
they are not marked language-blocked because this slice does not implement them.

The Rust harness executes 42 calls to the actual pinned `MappedPageTable`
implementation. Its frame registry owns three stable, aligned, initialized
PageTable allocations for the complete mapper lifetime; unknown IDs panic
instead of returning fabricated pointers. Only read-only translation runs.
The same harness exercises actual construction, all-512-entry mutable iteration,
zeroing, complete zero verification and non-present nonzero state.

Omega executes 43 captured-word scenarios across low/high canonical addresses,
all three page sizes, absent levels, malformed root flags, non-present PT words
and huge-page truncation. Six separate table/address fixtures exercise bounds,
full scans, last-slot metadata, clearing, all four captured indices, identity
mismatch and revalidation of constructed result payloads. Separate constants
keep repeated full-array scans within the evaluator work budget; exceeding the
budget in the former aggregate fixture was resolved by partitioning tests.
Six body-mutating controls require computed failure at the unchanged final
success contract: word identity, last-slot scan, zeroing, table-link identity,
numeric address and non-present leaf flags.

Fresh compiler: `eaa7993a23623cd8fabf45350340479c5c9c7879`, SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
The bodies and controls pass semantic evaluation. No native ABI measurement,
live table access, root publication, mapping mutation or TLB operation is claimed.

```sh
python3 tools/ports/x86_64-page-tables/check.py --omega /path/to/omega
python3 tools/ports/x86_64-page-tables/check.py --host-only
```

On pin updates, review complete implementations and the intentionally preserved
non-present/huge-page behavior before regenerating expected cases.
