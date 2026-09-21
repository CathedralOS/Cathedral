# Canonical conversion preflight

object_conversions.omg adds no opcode, Store, reference resolution,
mutable target, object ID, model variant or runtime dependency. The exact imported source closure is bound to each final receipt. Result
cases carry detached Integer/Bytes or Failure; all 256 output bytes are verified.

Reproduce here:

```
python3 fixtures.py --check
python3 inventory.py --check
python3 reference.py --write
python3 check.py --record verification.json
python3 check_const.py
python3 verify_record.py
```

The tools resolve the Cathedral checkout from their repository location. The unchanged Rust checked runner is
built only under /tmp/cathedral-acpi-object-conversion-runner using canonical
checked_runner.rs and runner.Cargo.lock. It never rebuilds a scratch Rust source
into the shared execution target. Omega source is the clean pinned eaa7993 checkout.
The standalone Object probe uses its own isolated Cargo target and retained lock.
The public opcode probe uses the canonical public-execution binary/source/lock.

207 scenarios exercise complete authored Omega bodies, each with a changed
expected-value/length/failure-reason control returning 1 rather than 0. Every Bytes
result checks all 256 bytes, including zero tails copied from dirty Owned storage.
Representative scalar-width, owned-buffer and maximum-ID scenarios also run as
three constant-expression positive/negative pairs. Source checking alone is not
counted as behavior verification. The harness's 10-million evaluation limit is
separate from the finite API limits.

The receipt binds the exact 13-file imported Omega/build closure plus suite
scripts/cases, Rust Object-probe source/lock and checked runner source/lock.
Historical scratch receipts and their original canonical baseline remain under
history/prepublication with their original manifest. Unrelated source modules
are not claimed as exercised. No native execution, hardware, ABI or privileged
service behavior is claimed.

reference.py records 100 calls to actual public Object conversion methods and 66
separate actual public Interpreter explicit ToInteger/ToBuffer evaluations.
Object probes use already-materialized canonical logical bytes; opcode probes
retain original Buffer declared-size/initializer encodings. This distinction
preserves the pin's overlong-initializer panic instead of comparing unequal
Object payloads. The two APIs differ for String→Buffer termination; records keep
both observed results. They also retain raw 64 Integer behavior at 32-bit width,
empty-buffer zero, permissive numeric-prefix String parsing and wider-field
conversion. These are evidence for intentional profile differences, not hidden
value agreements. No private algorithm mirror or ObjectToken is used.

The source map keeps whole generic methods pending: explicit opcode context/
target behavior, implicit String→Integer and wider BufferField conversion remain
outside this adapter. PinnedObjectBytes is compatibility with Object::to_buffer,
not the ACPI implicit conversion rule. The default checked-interpreter run uses
disjoint batches of five cases and evaluates every positive/control pair.

Scratch public-probe caveat: 12 reference-labelled Object rows all construct
RefOf in object_reference.rs; labels identify Omega cases, not distinct Rust
ReferenceKind observations. The final probe passes exact ReferenceKinds and regenerates observations;
Omega fixtures already used their exact kind. The original raw receipt and
probe source remain immutable under history/prepublication.
