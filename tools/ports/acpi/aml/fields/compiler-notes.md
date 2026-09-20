# Source-form evidence

With Omega `eaa7993a23623cd8fabf45350340479c5c9c7879` (binary SHA-256 `2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`), the original large-bit-offset fixture returned `InvalidState` at its `bit_offset <= bit_limit` check when the limit was `u64` maximum. The same fixture with `u32` maximum succeeded; direct PkgLength decoding independently succeeded.

`reproduce-field-comparison.py` retains the isolated observation: typed scalar `0 <= u64` maximum passes, the equivalent direct record-field comparison computes false, and explicitly typed locals loaded from those fields pass. This is a witnessed semantic-evaluation mismatch, not a complete compiler root-cause diagnosis. The production field parser now stages its bit-limit comparison and subtraction operands through typed scalar locals. No Omega code or proof rule was modified, and maximum-`u64` behavior remains covered by the actual parser fixture and its changed-body control.

Other source corrections were ordinary proof work: read-width masks before narrowing casts, an immutable captured index for descriptor array writes, and exact expected initializer bounds in a fixture instead of unconstrained subtraction. They are not language blockers.
