# Recursive mapper numeric witnesses

See [the port record](../../../source/libraries/x86_64/recursive-routes.PORT.md).
The reference copies pinned branch bodies into free functions over ordinary
borrowed initialized snapshots. Instrumented wrappers execute actual PTE setters
and PageTable::zero. It never constructs a RecursivePageTable or dereferences a
recursive address. Allocation inputs are a safe numeric observation queue.

`python3 tools/ports/x86_64-recursive-routes/check.py --omega /path/to/omega`
checks source bindings, fresh adapted-body Rust observations, 39 Omega route
batches, three translation batches, added input/capture cases and nine body
mutations. `--host-only`, `--positive-only`, `--controls-only` and
`--route-batch N` provide bounded reruns. No live mapping or native ABI claim.
