# Historical generic-execution inputs

`scratch-success.tar.gz` retains the complete successful scratch bundle with its
original receipts, exact source/tool inputs and SHA manifest. These are scratch
results, not canonical-path replay. Its original execution root is recorded in
the archived handoff; no receipt has been rewritten to claim canonical execution.

`pre-generic-canonical.tar.gz` retains the prepublication canonical files plus
four original execution/pipeline receipts and their exact recorded source inputs
under `recorded-inputs/`. Inputs differing from the prepublication tree were
recovered from Git by matching each recorded SHA-256. All46/33/51/45 source inputs
were recovered. These receipts remain historical; they are not current generic
runtime proof.

The canonical replay was committed at
`c4a8b03e2ff50b8e0eb69231f80e31554511291f`: 259 checked scenario/control pairs and
one constant-evaluator pair. Its exact inputs and receipts are preserved in Git.
The top-level verifier deliberately requires the recorded canonical root and
matching current inputs. After later source changes, verify the committed
milestone without rewriting its receipts:

```
python3 tools/ports/acpi/interpreter/generic-execution/history/verify_checkpoint.py
```

This checks the committed source, fixture, driver, document and receipt hashes.
It does not rerun the compiler or claim that the milestone tests exercised later
source changes. No native or hardware execution is claimed by these records.
