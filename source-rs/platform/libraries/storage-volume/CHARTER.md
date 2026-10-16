# Experimental volume boundary

Start at [lib.rs](lib.rs): inspect two envelope copies through an explicitly assigned
block region and validate the selected geometry. [header.rs](header.rs) owns the
same codec used by the [host volume lab](../../../../tools/storage-lab/engines/volume/README.md).
[region.rs](region.rs) owns checked byte-to-sector translation. This library has no
allocator, runtime dependency, kernel imports or production disk-format commitment.

`BlockRegion` exclusively borrows a `contracts::block::Device`. Its start and length
must be nonempty, sector aligned and fit the reported device extent without overflow.
It validates each entire IO before the first device operation. Byte writes crossing
sector boundaries use read/modify/write for partial sectors; aligned full sectors
avoid the read. This preserves bytes outside each requested range, with constant
512-byte working storage. The assigned range is a trusted provider boundary, not a
kernel-enforced subregion capability. Its owner still holds the whole controller grant.

Successful `write` is not an acknowledgement of durability. `flush` delegates to the
device barrier; success means all preceding successful writes have persisted under
the device contract. An IO error during a write (including a read/modify/write read)
or flush poisons the region: later reads, writes and flushes are refused without IO.
The owner must recover the transport and reopen durable state; dropping and rebuilding
the wrapper alone does not resolve an uncertain outcome. No multi-sector atomicity
is implied. Bounds errors occur before IO and do not poison the region.

`inspect` reads and selects envelope metadata only. It neither repairs copies nor
opens/initializes the database. A returned formatting state is not a ready database;
the caller must check readiness and validate its engine separately. The host formatter
and redb logical-length adapter remain in the host lab. The envelope encoding and
failure rules are documented there, including refusal of intact unsupported versions.

The [guest probe](../../../distribution/lab/volume/main.rs) runs over the existing
[ATA driver](../../drivers/ata-pio/CHARTER.md), whose flush issues ATA FLUSH CACHE and
waits for completion. The driver owns controller sequencing; this library owns range
translation; the distribution fixture owns what is tested. Neither the kernel nor
the ordinary storage service has gained a database-format dependency.

Run the proof from the repository root:

```powershell
python tools/boot-harness-rs/volume_test.py --release
python tools/boot-harness-rs/volume_test.py
```

The harness formats a new 4 MiB redb volume on the host, surrounds it with guards and
a separate scratch region, then makes four QEMU boots. It checks initial byte writes
and flush completion, cold reopening, one damaged envelope copy, and refusal of two
damaged copies. The guest compares an opaque engine prefix against the host fixture;
the host compares **every image byte** after each boot and finally reopens the intact
volume with redb. Writes include a 1,031-byte range starting at byte 509 and the final
byte of the scratch region, so both sector crossing and untouched padding are checked.
The volume's actual engine bytes remain unchanged; redb transactions run only on host.

Each run retains unique images, boot logs and source fingerprints under
`build/boot-harness-rs/volume/`. The ordinary session disk is never used. The profile
alone is build-only in the general harness; use the proof runner to supply its image.
QEMU termination/restart is not a simulation of host-cache loss or hardware power
failure. Host tests inject partial writes and failed flushes; the existing 939-cut
engine/envelope sweep remains separate evidence.

Recorded runs on 2026-10-16: debug `20261016T224117532205Z`, release
`20261016T224229165626Z` under the volume artifact directory; the shared-codec
939-cut regression is `build/storage-volume/20261016T223858135952Z`.

Five host tests cover the codec, arbitrary byte spans and preservation, pre-IO bounds,
every device error in the partial-write/flush trace, and read-only envelope inspection.
They run with `cargo test --locked` in `source-rs/`. The library also compiles for both
bare-metal CPU targets; the actual ATA/QEMU proof is x86-64 only.

The dev profile optimizes only BLAKE3 for size: the unoptimized dependency produced
a 292-page guest executable, exceeding the existing 80-page image bound. With that
dependency optimized the debug executable uses 77 pages (release: 62); guest code
still has debug assertions and symbols. The probe sequences buffer use after envelope
decoding returns so the debug call chain fits the existing 16 KiB stack. The ARM64 bare-metal build uses BLAKE3's
`pure` feature to avoid requiring a separate NEON C cross-compiler. This is a lab
build choice, not a selection of future hashing acceleration policy.

Next: a funded userspace allocator and sufficient memory/image admission for a guest
database runtime, then a redb backend over this block transport. The present user
runtime has no allocator and caps dynamically granted private memory at 16 pages
(64 KiB). This milestone does not raise those limits, install an engine, format from
the guest, or change the private catalog service's durable format.
