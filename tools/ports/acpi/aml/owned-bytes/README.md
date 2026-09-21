# Owned AML bytes: current-body verification

These original fixtures exercise the canonical Value/ObjectStore APIs, not a
parallel interpreter model. The main 128 scenarios each execute positive and
changed-body-control machines after checking the actual source closure. Every
comparison includes all namespace metadata, all 64 object payloads/links, and all
16,384 initialized arena bytes. Controls alter an expected byte inside the test
body. The 10 extra scenarios exercise composition and the affine Program wrapper;
three representative extras also run as const expressions with requires checking.

Reproduce from the Cathedral root:

```sh
python3 tools/ports/acpi/aml/owned-bytes/fixtures.py
python3 tools/ports/acpi/aml/owned-bytes/check.py --record tools/ports/acpi/aml/owned-bytes/verification.json
python3 tools/ports/acpi/aml/owned-bytes/check_extras.py
python3 tools/ports/acpi/aml/owned-bytes/check_extras.py --const
python3 tools/ports/acpi/aml/owned-bytes/reference.py
python3 tools/ports/acpi/aml/owned-bytes/check_regressions.py
python3 tools/ports/acpi/aml/owned-bytes/verify_record.py
```

The checked runner builds from the clean exact Omega revision in the adjacent
checkout with its retained lock. The default main path batches eight scenarios
at a time. `--match` accepts comma-separated name substrings. The named existing
parser, reference, integer executor, pipeline and field suites retain their
original controls; their new receipts go under `regressions/`. Older receipts
remain unchanged and historical. There is no native, ABI, firmware or live-handler
claim. Test work/output capacities are unrelated to provisioned native stack size.

`reference.py` probes actual pinned public Object reads/clones and separately
labels eight exact private `copy_bits` write-body mirrors. It never constructs an
ObjectToken or calls the private AML Index/Store machinery. The Array/owner limits,
ASCII validation and numeric-only wide-field outcomes are explicit bounded-policy
differences described in the source PORT.

The original development run encountered one shared runner replacement from a
parallel scratch build. `runner-incident.json` records the exact interval and
affected eight-case batch. The raw original receipt is retained; its end-of-run
runner hash is not used to attest that affected batch. Final composed evidence
uses the separately recorded canonical rerun for those same eight cases, with an
identical fixture/source hash. All other batches and the pipeline/extra processes
were launched outside the replacement interval. Avoid concurrent alternate-source
builds into that same shared target when reproducing these receipts.
