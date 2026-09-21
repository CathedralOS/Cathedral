# Historical pre-scalar named Store evidence

`pre-scalar-canonical.tar.gz` retains the exact 305 checked/three constant pair input closure, the 297 public Rust probe input/production mapping closure, and all 27 pinned upstream Rust source hashes before the scalar extension changed named_value_store.omg. Repository inputs come from commit c4a8b03; manifest.json records its full identity and every archived file hash. The archive retains both original receipts and their literal execution root/build text.

```sh
python3 tools/ports/acpi/aml/named-value-store-scalar/history/verify.py
```

This checks historical input integrity only. It does not rerun either suite or claim the old receipt hashes describe the extended production source. The current scalar suite separately re-executes all 305 unchanged object-source fixtures within its 504 pairs. Original named-value-store tools/receipts remain untouched. Recorded compiler/runner/probe binary identities remain in their original receipts; binary artifacts are not embedded in this source archive.
