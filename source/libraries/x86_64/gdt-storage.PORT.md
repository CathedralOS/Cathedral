# Detached owned GDT storage

Status: **tested**. All 16 Omega scenarios, the malformed-input/reset fixture
and four body mutations pass; the producer source check passes 16 files. This X86-001/002
slice completes ordinary storage mutation around the already tested semantic
[Descriptor cases and append plans](../../drivers/facts/x86_descriptors.PORT.md).
It modifies `src/structures/gdt.rs` at x86_64
`cc35c876d3badb57df54a66e22f7768a52be95f2`, retaining
[MIT OR Apache-2.0 notices](../../../licenses/rust-osdev/x86_64/).

The pinned constructor/append/entries bodies, Omega's data/default-domain and
borrowing specifications, and the library charter were reviewed before coding.
No native placement is requested. The storage consists of ordinary initialized
u64 words; the existing Descriptor and SegmentSelector remain canonical. A raw
word is the existing GdtEntry's numeric payload, without a second entry schema.

## Behavior

`Table` owns 8192 words, a checked logical capacity in 1..8192, and a length
including the mandatory null entry. This one backing profile replaces Rust's
const-generic physical array size; it costs 64 KiB even at capacity eight.
`initialize_default` selects capacity eight; `initialize` clears the complete
backing and establishes length one. Invalid capacity leaves the prior value
unchanged. Zero-initialized Table metadata is intentionally invalid until
initialization or import; ordinary construction is not a validation credential.

`import_words` accepts a borrowed initialized array and an explicit prefix
length. It checks capacity, nonempty length, length<=capacity and first word zero
before modifying the destination. It then clears the complete backing, copies
exactly the prefix, and installs the length. Both borrows are ordinary Omega
borrows; callers cannot supply overlapping input/output through numeric addresses.
Sixteen-word blocks and a bounded tail implement the copy. This keeps full-size
semantic tests within evaluator fuel while preserving the exact prefix behavior.
No interpretation or validity check is imposed on later raw descriptor words.

`append` validates metadata/null entry, calls the canonical append plan, then
writes one UserSegment word or two SystemSegment words in low/high order. Both
system slots must fit before either write. Failed appends preserve length and
storage. Added carries the existing SegmentSelector with the descriptor's DPL;
Full and InvalidTable are semantic alternatives. No invalid numeric descriptor
word-count state is introduced. This directly uses the case data requested for
semantic descriptor alternatives.

`entry` checks the current table geometry and returns a numeric word only inside
the live prefix. It replaces Rust's dynamically sized entries view with a checked
indexed projection. `limit` reuses the existing `(length*8)-1` helper, including
65535 at maximum length. Public fields remain editable ordinary data, so these
operations recheck current metadata; no setter grants hardware installation.

Rust panics on invalid constructors or insufficient room. The detached API
returns false or an explicit result case; rejected operations preserve the
existing destination. There is no concurrent atomic Entry access, physical
pointer reconstruction, GDTR construction, segment reload or GDT installation.
The owner must separately establish native layout, persistent backing and live
mutation rules before using any table with hardware.

## Evidence

The reference executable calls actual public pinned GlobalDescriptorTable APIs,
never private-body mirrors. Sixteen scenarios cover capacities 1, 2, 3, 8, 64 and
8192; mixed user/system entries; all descriptor privilege bits; import; full and
one-free-slot rejection; last-slot and last-pair appends; lengths around every
16-word copy boundary through 33; and full 8192-entry import. It inspects all Rust
live words and asserts failed operations leave them unchanged. Default creation
and five invalid constructors are also checked. No table is ever installed.

The generator records sparse representations of those exact observed arrays.
Omega fixtures execute the real storage bodies and compare selectors, lengths,
limits, every observed nonzero position, the first nine positions, the current
last/next positions and slot8191. This is selected-word semantic evidence, not
an assertion that Omega compared every zero position in each large array. The
nonzero 15/16/17/31/32/33-word inputs exercise every block assignment and tail.
Extra cases check invalid capacity/import/null/geometry, rejected projections,
unchanged sentinel storage, default reset and full-backing last-slot clearing.
Four negative controls alter expected high-word contents, the Full outcome,
selector DPL and preserved sentinel value while retaining the success contract.

The [inventory](gdt-storage-inventory.json) is a narrow whole-file overlay.
Earlier metadata-only mappings remain valid as component evidence; this slice
adds actual owned storage. It does not claim the full x86 package is complete.
Native Omega execution, layout and CPU integration remain unmeasured.

Compiler: Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`, binary SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.

```sh
python3 tools/ports/x86_64-gdt-storage/check.py --omega /path/to/omega
python3 tools/ports/x86_64-gdt-storage/check.py --host-only
```
