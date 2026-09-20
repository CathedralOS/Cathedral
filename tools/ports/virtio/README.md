# Pure VirtIO port checks

See `source/libraries/virtio/PORT.md` for provenance and stage limits.
`check.py` audits the full pin, private raw fields and enum widths, compares
cross-target Rust observations, and runs actual pinned ring allocation cases.
`semantic-check.py` checks eight pure Omega groups and eight body-mutating
negative controls. `layout-check.py` independently checks named policy source;
`--combined` reproduces a current trait-resolution composition diagnostic.

The Rust host executable only allocates ordinary process memory. No test maps
registers, performs DMA, activates a queue, or treats expected geometry as an
observed Omega ABI. Build outputs in `target/` are disposable and ignored.

Packed, transport and simulator additions have separate semantic runners:
`packed-semantic-check.py`, `transport-semantic-check.py`, and
`simulator-semantic-check.py`. Each checks its positive groups then mutates
expected behavior within each group and requires computed failure.
`packed-check.py` and `transport-check.py` observe actual pinned Rust geometry
and check bitfield/register metadata. `layout-check.py --module packed_layouts`
and `--module transport_layouts` check the additional policy declarations.
`reproduce-unsigned-max.py` records a narrow current compiler inference issue;
its inferred and explicit forms are compared, not treated as port acceptance.
