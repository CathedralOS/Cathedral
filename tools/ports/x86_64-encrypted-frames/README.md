# Explicit-profile frame checks

This harness observes actual public `PhysAddr`, `PhysFrame`, `PhysFrameRange`
and `PhysFrameRangeInclusive` operations from pinned x86_64 with its
`memory_encryption` feature. The Rust profile is release, 64-bit `usize`. These are host arithmetic calls;
64-bit/x86-64 numeric profile labels do not claim x86-64 target code or instruction
execution.
Each of 13 configuration sequences runs in a fresh process before constructing
addresses. This does not inspect or configure the host CPU. The 7,008 public
calls include current/old/low/outside-52 bits, repeated 47→48→47 configuration,
all three page sizes, PFN/multiplication boundaries, inclusive/exclusive
selection and actual iterator cursor/panic behavior. Forty additional Omega
cases exercise the explicit closed-size policy. Rust release multiplication
wrapping is retained in `rust_*` fields; `ok/value` express the documented
checked-multiplication policy where different.

From the Cathedral root:

```sh
python3 tools/ports/x86_64-encrypted-frames/check.py --host-only
python3 tools/ports/x86_64-encrypted-frames/map_inventory.py --check
python3 tools/ports/x86_64-encrypted-frames/check.py --record tools/ports/x86_64-encrypted-frames/verification.json
python3 tools/ports/x86_64-encrypted-frames/check_const.py --record tools/ports/x86_64-encrypted-frames/const-verification.json
python3 tools/ports/x86_64-encrypted-frames/verify_record.py
```

`check.py` compiles the actual authored Omega closure with the checked runner
from `tools/ports/acpi/interpreter/execution`, then executes 156 positive grouped
fixture bodies and 156 bodies with a changed expected acceptance/failure
condition. Each group stores up to 48 initialized rows and executes a shared bounded comparison loop; controls are
per group, not per individual case. A 10,000,000-step harness limit applies.
`--match` selects group-name substrings for diagnostics. The runner is built
from clean Omega revision `eaa7993a23623cd8fabf45350340479c5c9c7879`.

`check_const.py` independently checks six representative single-case fixtures
and their changed expectation controls using the current compiler at
`/tmp/cathedral-omega-eaa7993/release/omega` (SHA256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`).
Successful controls must report the evaluated `1 == 0` contract failure.
Checked-interpreter execution, constant evaluation and native execution are
different stages. No native or hardware execution is claimed. The JSON records
retain exact source/tool hashes and diagnostics; `verify_record.py` rejects
stale evidence. Source coverage is separate from these test results.
