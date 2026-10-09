# Canonical owned AML byte storage

This is a finite ordinary-data storage layer, integrated into the existing AML
`Value` and single-source `Program`. It is not generic Store/CopyObject/Index
opcode dispatch, region access, allocator authority, or a completed AML executor.

Upstream is [rust-osdev/acpi](https://github.com/rust-osdev/acpi/tree/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5),
commit `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, `src/aml/object.rs` and
`src/aml/mod.rs`, copyright 2018 Isaac Woods, MIT OR Apache-2.0. Exact source,
manifest and license hashes are in `byte-storage-inventory.json`; applicable
texts and translation provenance remain in `licenses/rust-osdev/acpi` and
`THIRD_PARTY_NOTICES.md`. No firmware samples are included.

## Representation and ownership

`Value::String { string_storage }` uses `StringStorage::Source { string_source }`
or `Owned { string_owner }`. `Value::Buffer { buffer_storage }` uses
`BufferStorage::Source { declared_size, buffer_initializer }` or
`Owned { buffer_owner }`. Source cases retain the exact immutable `Span`; no
unit value encodes writable memory. `Value::BufferField` carries backing object
ID, bit offset and bit length. This extends the one canonical Value enum.

`ObjectStore` pairs the existing `Namespace` with `ByteArena`. Its 64 initialized
`ByteBlock`s each contain 256 initialized bytes, a logical length and an
initialization flag. An Owned payload is valid only when its owner ID equals its
containing live object slot, its block is initialized and its length is at most
256. Source spans require matching unit, ordered bounds and end within the
retained logical source length (at most 1024). Owned reads do not consult source
bytes. These are profile capacities, not ACPI maxima or storage-authority grants.

`Program.store` owns that pair alongside its existing source snapshot and
MethodDefinitions. Program and Prepared are affine; moving `prepared.program`
leaves its independent outcome field usable. The internal copyable ObjectStore
remains ordinary detached snapshot data: copying the whole store copies its
arena too. Numeric IDs are local to the particular store, not globally unique
handles or Omega borrows. No cross-Program reference transport is provided.

Object IDs are never recycled during this profile's Program lifetime. Replacing
a byte payload reuses its owner's block. Frame teardown cannot free this arena;
reference wrappers and fields retain stable IDs. The new lifetime fixture returns
an Index ID from an ordinary host helper and then mutates its retained backing.
It does not claim that generic AML Return/local/argument promotion is implemented.

## API and operation contracts

All APIs take initialized source bytes, logical source length/unit, and the same
borrowed ObjectStore. Shared reads and staged exclusive mutations use Omega's
ordinary array bounds, borrow and lifetime rules; numeric IDs confer no access.

| API | Behavior |
|---|---|
| `read_bytes`, `byte_length` | Follow Named/Local/Arg transparent references; validate type, owner, complete span and String encoding; return detached initialized data |
| `materialize` | Replace source metadata with its own owned block only after complete validation; already-owned valid data is preserved |
| `clone_bytes_into` | Deep-copy a byte source into an existing destination ID; copy the complete logical bytes and zero unused destination tail |
| `create_buffer_field` | Allocate one field identity after complete nonzero bit-range validation |
| `make_byte_index` | Allocate field plus RefOf identities atomically; validate index before checking space for both slots |
| `read_field_integer` | Follow reference wrappers to a field, then re-resolve its backing; return numeric bits only when the field fits the selected 32/64-bit width |
| `write_field_integer` | Stage normalized 4/8-byte integer bits, zero extend if the field is wider, preserve every outside bit and initialized tail byte, then commit |

ByteResult/ByteRead/FieldRead outcomes are semantic cases: Success, InvalidState,
Capacity, Bounds, Encoding, UnsupportedValue, ReferenceCycle, WorkLimit. Consumers
must inspect outcome before interpreting the default payload of a failed result.
`ByteResult.object` is the resolved byte owner for read/materialize/clone/write,
the new identity for creation, and the resolved backing owner for numeric field
reads. Its `length` is bytes for byte operations (including field writes), and
bits for field creation, Index construction and numeric field reads; only numeric
reads populate `value`. These units are tied to the called API, never inferred
from an untagged source span.
Read snapshots include initialized capacity, but only bytes below logical length
belong to the value. Source snapshots have zero padding/tail; existing Owned
snapshots retain their initialized unused tail. No length-plus-offset overflow
is formed before bounds checks, including u64::MAX inputs.

Clone destinations may be Uninitialized, Integer, Package, Reference, String or
Buffer. This is a direct destination-ID kernel, not Store's reference-target
selection. Existing byte destinations must themselves validate before replacement.
Methods, permanent BufferFields and service-bearing values are rejected. Object
IDs, namespace bindings, package sibling links and allocation counters are
preserved by cloning. MethodDefinitions are untouched; this API cannot make an
arbitrary Method copy executable. Generic copying of other values remains separate.

Shallow identity sharing uses the same object ID. Namespace-only `copy_value`
continues to share Reference/Package/BufferField edges and immutable Source spans,
but rejects either an Owned source or destination because it cannot update the
paired arena. Deep byte copying must use `clone_bytes_into`. Self-copy is legal
and produces the same logical bytes with a zeroed unused tail.

Every field access resolves its backing object again and checks the current type
and logical range. A field sees replacement of that object's payload; namespace
name rebinding does not retarget it. A replacement with a shorter value can make
an old field out of bounds. String is accepted as internal field backing because
String Index produces a byte field; this kernel is not AML CreateField dispatch,
which requires Buffer. Full arbitrary byte-input field writes and wide Buffer
read results remain pending.

All mutating operations stage validation, scratch bytes and required slot counts
before any observable write. Any error preserves the entire supplied namespace,
all byte blocks and allocation counters. Successful field writes materialize
source-backed bytes atomically; immutable input never changes. This is per-operation
atomicity, not whole-method rollback. String results must remain ASCII 1–127 without
embedded NUL; failure to preserve that profile returns Encoding before commit.

## Specification and pin differences

The primary references are ACPI 6.6 [§19.6.10 Buffer](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#buffer-declare-buffer-object),
§19.6.21 CreateField, §19.6.63 Index, and
[§20.2.3 / §20.2.5 data and expression encoding](https://uefi.org/specs/ACPI/6.6/20_AML_Specification.html).

Buffer materialization uses `max(declared size, initializer length)`: copy the
complete initializer and zero pad to that length. The pin instead takes a
shortened destination and full initializer in `copy_from_slice`, which can panic;
this corrects the earlier static port note that proposed truncation. An ASL
string initializer's encoded terminating zero is already part of the AML ByteList;
this layer does not synthesize another terminator. AML String spans exclude their
terminator and obey the explicit ASCII 1–127 grammar.

The pin compares BufferField bit length with `IntegerSize`'s byte-valued enum in
`read_buffer_field`, so it can return Buffer for fields fitting an Integer. This
explicit numeric API uses 32/64 bits and rejects wider fields rather than claiming
a general Object-valued read. The pin's write helper has a TODO for bounds and
mutates String storage through unsafe UTF-8 access; this layer validates complete
bounds and the ASCII/NUL-free postcondition before committing. Zero-width fields
return Bounds, corresponding to the primary CreateField rule without a host panic.

## Verification and migration

Current receipts pass **138 storage/composition scenarios and 138 changed-body
controls**, **306 regression pairs**, **eight const pairs**, and **21 Rust
observations** (13 public-read/clone observations and 8 exact private bit-write
mirrors). The own inventory retains 157 pending anchors and one translated
`copy_bits` dependency, with partial generic components explicitly mapped.
Records live under `tools/ports/acpi/aml/owned-bytes/`.
The runner checks actual bodies and then executes positive and changed-body
controls. The main corpus compares every namespace field, all 64 object payloads
and links, all 32 entries, and every byte/metadata field of all 64 arena blocks.
Controls change the expected final arena byte, exercising the complete comparison.
Extra fixtures cover byte length, typed copies, reference backing, source ownership,
affine Program movement and IDs retained across a host helper return.

`reference.py` executes actual pinned public Object clone/read/access operations;
its `copy_bits` write probe is an exact private-body mirror, separately labelled.
It never forges ObjectToken or claims to call public mutable WrappedObject APIs.
Root's independent public AML probe also records byte CopyObject/Index behavior;
those observations do not substitute for these kernel-body tests.

The model migration changes source hashes. Pre-storage parser, reference, field,
execution and pipeline verification JSON files are retained unchanged as historical
receipts bound to their original source. Fresh regression records are stored under
`owned-bytes/regressions/`; `verify_record.py` checks this milestone's current closure.
No native publication, ABI, live handler or firmware run is claimed.
