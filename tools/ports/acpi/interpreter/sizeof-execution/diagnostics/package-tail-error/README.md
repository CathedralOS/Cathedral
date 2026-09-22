# Original SizeOf terminal evidence

The full 362-pair run at `b84d790` reached checked interpretation and evaluated
all 724 entries. It ended with 723 passing entries and one failed positive:
`SizeOfExecutionSuite::size_malformed_package_positive`, expected zero and
observed one, with no interpreter error. Its control passed. Thus 361 complete
pairs passed; the full run remains a failure.

The fixture creates Package object 0 with Integer children 1 and 2, then sets
child 2's `has_next` and `next=1`. The canonical package walk rejects an extra
link after the advertised last element as `Outcome::BadEncoding`, before it
would follow the link. The SizeOf adapter preserves that error. The fixture
incorrectly expected `ExecutionOutcome::InvalidState`. The existing
`package_tail_extra` metadata pair expects `BadEncoding` and passed in this
same run. The required correction is to the fixture expectation.

The old checker asserted the runner status before writing its normal receipt.
`terminal-output.json` preserves the complete tool result, including traceback
and every entry line. `failed-run.json` explicitly records the checker exit as
1 and leaves the lost runner status unknown; the assertion establishes only
that it was nonzero. This is recovered evidence, not a normal passing receipt.

The eight files in `generated/` were copied from the original Omega source
cache and compared byte for byte with regeneration from the unchanged 160
launch inputs. `audit.py` reconstructs those exact original files from Git in
a temporary directory, regenerates every module, checks all recorded hashes,
and verifies the complete ordered 724-entry observation list. It does not
execute Omega or change the original result.

```sh
python3 tools/ports/acpi/interpreter/sizeof-execution/diagnostics/package-tail-error/audit.py --require-binaries
```

The corrected candidate retains `correction-source-audit.json` and
`correction-body.diff`: only the positive assertion's expected error changes,
with production and all other generated entry bodies identical. The checker
and verifier additionally preserve/reject failed or changed-source receipts.
Their host transport tests use synthetic temporary results solely to exercise
receipt handling. `correction-public-replay.json` binds 76 actual observations
to the unchanged public binary. None of these records promotes the corrected
Omega candidate to a passing full run.
