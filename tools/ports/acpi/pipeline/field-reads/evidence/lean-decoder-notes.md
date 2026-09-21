This is an unrun, isolated source variant. No checked result is claimed.

Only decode_generic_value's read-enabled selection is changed. Existing mode,
continuation bounds/contribution admission, deferred cursor publication, legacy
admission, ObjectType dispatch and all other decoder bodies remain byte-identical.

The original read_value returns has_object=false for Integer and Uninitialized,
which immediately routes to admit; the variant takes that same route directly.
For Object it calls the exact same unwrap_transparent(store.space, id, 64). Its
success condition plus object<64 exactly matches read_value's selected arm.
Only a resolved canonical FieldUnit reaches deferred, with the same object ID.
Transparent Named/Local/Arg reference handling and explicit RefOf/Index stopping
are therefore unchanged. No value is evaluated during the new inspection.

On unwrap failure or a non-Field value, both versions call the unchanged admit
function with the original Operand. The probe does not return the raw Outcome or
change error mapping/precedence. Admission still repeats ordinary validation and
retains existing ReferenceCycle/WorkLimit/capacity policies. A malformed success
with object>=64 also follows the same fallback. The success bounds precede the
new direct array access. Neither probe mutates Frame, ObjectStore or source.

The intended performance difference is avoiding a rich canonical ValueResult
return and state argument where only the resolved ID and Value variant matter.
This is a hypothesis, not a measured improvement. Frozen jobs retain the original
implementation. A future comparison should use the same pinned runner, source
closure, generated one-pair fixture and step budget, changing this file alone;
retain both exact hashes and independent elapsed results before adopting it.
