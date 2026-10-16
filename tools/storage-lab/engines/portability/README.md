# no_std engine portability and allocation measurement

The library checks redb's experimental backend interface on bare-metal targets.
The `measure-runtime` host executable uses the same no_std engine with a normal
host file backend and a metered System allocator; std belongs to the harness.

Run inside this directory, choosing a new output path (creation refuses overwrite):

```powershell
cargo run --locked --release --features measure --bin measure-runtime -- ../../../../build/redb-runtime-new.db 32768
```

The CSV reports create, 32-byte commit, 16 KiB commit, close and verified clean
reopen. On 2026-10-16 the 32 KiB cache trace reported:

| Phase | Live bytes at end | Peak requested live bytes | Largest request |
| --- | ---: | ---: | ---: |
| Create | 34,608 | 39,016 | 9,432 |
| Small commit | 48,288 | 54,804 | 4,112 |
| Large commit | 59,448 | 100,067 | 32,784 |
| Close | 0 | 78,676 | 4,416 |
| Reopen and close | 0 | 95,335 | 32,784 |

These are requested bytes, not RSS, allocator overhead, stack use, fragmentation,
or an upper bound on larger workloads. Realloc accounting uses net requested bytes,
not temporary allocator-internal coexistence of old and new buffers. This trace
uses clean close; the [guest runtime proof](../../../../source-rs/distribution/lab/database/README.md)
checks unclean recovery separately and supplies a 256 KiB fixed heap. No production
cache sizing or performance claim follows from this small trace.
