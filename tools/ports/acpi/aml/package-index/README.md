# Package Index construction evidence

See [port scope](../../../../../source/libraries/acpi/aml/package-index.PORT.md).
Status: final repository-path verification passed all 65 checked pairs, three
constant pairs and 19 public observations; retained receipts match current inputs.

From the repository root:

```sh
python3 tools/ports/acpi/aml/package-index/inventory.py --check
python3 tools/ports/acpi/aml/package-index/check.py
python3 tools/ports/acpi/aml/package-index/reference.py
python3 tools/ports/acpi/aml/package-index/verify_record.py
```

The driver executes 65 behavior/control pairs with complete store comparisons,
then three independent constant-evaluation pairs. Constant fixtures compare all
object/entry metadata and byte-block metadata plus each block's first/last bytes;
the checked stage retains every-byte comparison. The driver checkpoints completed
checked evidence before constant proofs and the verifier requires both stages. Every successful invocation checks
one fresh RefOf and exact object-count change; all old objects, all entries and
all 16,384 byte-arena bytes are compared. A second-call capacity failure preserves
the first allocation. Shared scalar-return assertions bound source-analysis cost.
The fixture generator constructs expected deltas independently of production;
changed expected kind, target, count or preserved bytes must return one.

Three independent temporary batches may execute concurrently; receipt order is
deterministic. The record binds the exact execution root and build text hash,
current used source/build closure, fixture inputs, generated program hashes,
immutable checked-runner/compiler hashes and pre/post input snapshots. Wall time
and sum of checked-batch elapsed time are distinct. The shared runner is never
rebuilt. The public fixture uses its own Cargo target and exact pinned sources.

Public probes execute actual Package Index opcodes. Their safe immutable pointer
comparisons observe identity without constructing ObjectToken or mirroring private
bodies. All service callbacks are trapped except construction of an inert mutex
handle required by Interpreter::new. The retained source-bound record includes
27 upstream source/manifest/license hashes and all probe/lockfile inputs. `--write`
refreshes public evidence deliberately; the default reproduces it exactly.

verify_record.py audits current inputs, every positive/control result and const
diagnostic, generated AML and public observations against the exact pin. It is a
receipt-validation utility, not retroactively claimed as an executed fixture input.
Neither aggregate Index completion nor native Omega/hardware execution is claimed.
