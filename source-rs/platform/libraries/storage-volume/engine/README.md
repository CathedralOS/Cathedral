# Experimental guest engine adapter

[lib.rs](lib.rs) inspects an assigned volume, requires ready state, opens redb and
validates the engine's volume identity against its envelope. [backend.rs](backend.rs)
owns serialized IO, logical-length changes and permanent failure state. It consumes
an exclusively owned `cathedral_block_device::Device`; the [parent codec/region library](../CHARTER.md)
remains allocator-free. This is a separate package so Cargo feature unification
cannot impose an allocator on ordinary volume probes.

The engine owns the whole assigned region from format; the region never grows.
redb still keeps a layout length and requires the backend to report exactly the
length it stored, so the adapter mirrors that length in the envelope: a change
flushes earlier engine writes, then publishes and flushes both envelope copies,
and writes nothing else. No range is zeroed, at growth or anywhere: the engine
never reads space it has not written, and after an unclean stop it rebuilds its
allocators from its trees rather than trusting region headers. Every read/write
is checked against the current length before IO. Mutation failure poisons the
backend, including when temporary BlockRegion wrappers have been dropped. Bounds
refusals do no IO and leave the backend usable.

This adapter uses redb's experimental no_std backend interface. It requires a funded
allocator and is not a stable engine selection, guest formatter, or kernel-enforced
subregion grant. The provider still owns its entire controller capability. Identity
validation may follow engine recovery; unlike envelope inspection, open can write.

Run `cargo test --locked -p cathedral-storage-engine --features host` from source-rs.
The tests check envelope-only growth and shrink/regrowth with engine bytes left
untouched, neighboring guards and each IO-failure position in a length change. They do not model cache loss. See the
[guest proof](../../../../distribution/lab/database/README.md) for real ATA/QEMU
transactions and independent host verification.
