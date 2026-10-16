# Experimental guest engine adapter

[lib.rs](lib.rs) inspects an assigned volume, requires ready state, opens redb and
validates the engine's volume identity against its envelope. [backend.rs](backend.rs)
owns serialized IO, logical-length changes and permanent failure state. It consumes
an exclusively owned `cathedral_block_device::Device`; the [parent codec/region library](../CHARTER.md)
remains allocator-free. This is a separate package so Cargo feature unification
cannot impose an allocator on ordinary volume probes.

The raw region never grows. Logical growth zeroes new bytes, flushes them, then
publishes and flushes both envelope copies. Shrink updates length without changing
the assigned region. Every read/write is checked against logical length before IO.
Mutation failure poisons the backend, including when temporary BlockRegion wrappers
have been dropped. Bounds refusals do no IO and leave the backend usable.

This adapter uses redb's experimental no_std backend interface. It requires a funded
allocator and is not a stable engine selection, guest formatter, or kernel-enforced
subregion grant. The provider still owns its entire controller capability. Identity
validation may follow engine recovery; unlike envelope inspection, open can write.

Run `cargo test --locked -p cathedral-storage-engine --features host` from source-rs.
The tests check zeroed growth, shrink/regrowth, envelopes, neighboring guards and
each IO-failure position in growth. They do not model cache loss. See the
[guest proof](../../../../distribution/lab/database/README.md) for real ATA/QEMU
transactions and independent host verification.
