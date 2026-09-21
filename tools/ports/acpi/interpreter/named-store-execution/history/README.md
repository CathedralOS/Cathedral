# Named Store source checkpoint

Run `python3 tools/ports/acpi/interpreter/named-store-execution/history/verify_checkpoint.py --source-ref <commit>` with the Git commit containing the six completed named Store receipts and their recorded sources. The explicit source reference is required; the verifier does not silently select the current checkout.

The verified named Store checkpoint is `a75f0cc1f88831b4e868320552b57e8d2ab7a313` (44 execution, 30 bridge, 25 target-action, 55 generic, 79 integer and 22 pipeline pairs).

The verifier reads receipts and sources from that commit, verifies every recorded source hash, and extracts the archived generators into a temporary directory. It reproduces each fixture, build file and ordered behavior/control selection. All 255 pairs must have exact successful observations, and the recorded runner, compiler pin, runner source and dependency lock must agree with the retained toolchain metadata.

This checks historical evidence integrity. It does not invoke Omega, require the original binaries to remain installed, or claim execution of later source changes. New unrelated ACPI files do not invalidate a historical source checkpoint. Changes to a recorded source require a matching earlier checkpoint or fresh execution evidence.

The nested location keeps this verifier outside the flat Python-file snapshots used by live named Store and generic execution checks. Historical receipts remain at their original paths and are read from the selected commit.
