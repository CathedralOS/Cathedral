# Encryption-profile recursive translation

Run `python3 tools/ports/x86_64-encrypted-recursive-translation/check.py`.
`--omega` overrides the pinned isolated compiler; `--start`/`--end` select
half-open batch bounds, and `--host-only` checks fresh reference output/freshness.

`generate_reference.py` extracts the complete pinned generic recursive translate
body, substituting safe initialized borrows for topology resolution and restoring
Page type inference. No RecursivePageTable constructor, pointer dereference or
CPU operation exists. The reference runs actual PTE/physical APIs in eleven
isolated memory_encryption configurations; 330 observations feed 55 Omega batches.

The six leaf-flag mutations, invalid-address mutation and leaf-HUGE mutation
must compute one under an unchanged zero-result contract. Source errors do not
count. See the PORT for measured status, profile and authority limits.
