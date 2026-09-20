# UEFI scalar semantic tests

Run from the Cathedral root:

```sh
python3 tools/ports/uefi-scalars/check.py
```

`--omega /path/to/omega` selects a compiler; the runner records its binary
SHA-256. The default is the existing sibling Omega release binary, which the
runner does not rebuild. The fixture depends on the raw Cathedral contract and
pure UEFI library through their real package declarations.

`TEST_RESULT` evaluates the real Omega helper bodies at compile time. A checked
call requires that result to equal zero. A temporary negative fixture adds one
to the computed result and must reject specifically at that contract; unrelated
compiler failure cannot pass the negative control. No source package is edited
for the negative check.

This tests Boolean conversion/order/equality/hash normalization, status classes,
revision encoding/extraction/order, time range/equality behavior, and custom
memory-type endpoints. It is semantic-evaluator execution, not native execution
or a UEFI ABI measurement. Expected foreign layout/value vectors and the
complete source map are beside
[scalars.PORT.md](../../../source/contracts/uefi/raw/scalars.PORT.md).
