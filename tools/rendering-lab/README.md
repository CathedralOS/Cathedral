# Rendering lab runner

Run `python tools/rendering-lab/run.py` from the repository root. It builds and runs
the native release benchmark, verifies all outputs, and saves `measurements.csv`
and `context.json` under `build/rendering-lab/`. Use `--output PATH` for another
destination. Requires Python and the repository's Rust toolchain; no QEMU is
needed for this host comparison.

Follow the [investigation](../../source-rs/distribution/lab/rendering/README.md)
for strategy definitions, measured results, limitations and QEMU commands that
exercise actual mappings. Start at [run.py](run.py) for orchestration or
[bench.rs](../../source-rs/distribution/lab/rendering/bench.rs) for the workload.
