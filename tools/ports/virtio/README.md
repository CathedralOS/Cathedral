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
