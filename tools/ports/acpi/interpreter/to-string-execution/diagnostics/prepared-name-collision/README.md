# Original Prepared-name diagnostic

Checkpoint `8772ff8` failed to compile the six selected AML pairs in `plan.json`.
The separate package retained here reproduces that failure with exit 2 and
unchanged inputs. `provenance.json` binds these files, the original adapter,
runner binary and Omega pin. `plan.json` retains the actual command and all 145
source/tool input hashes. No selected body evaluated.

The compiler reports an equality between `Outcome` and `ExecutionOutcome`.
The adapter's private `Prepared` has an ExecutionOutcome field, while the AML
harness imports `pipeline::program::Prepared` with an Outcome field. At the pin,
`psi/pipeline/symbol-resolved-trees-to-typed-trees/src/expressions/expression/table/structural_equality.rs`
uses `data_definition_by_name` for field comparisons; the implementation in
`expressions/equatable.rs` selects the first matching raw type name. This is
source evidence consistent with a collision, not a dump of this package's
resolved type bindings.

The candidate renames all 32 adapter references to `ToStringPrepared`; generated
fixture bodies are unchanged. Candidate execution is pending. The original
retirement package completed separately against frozen `8772ff8` inputs:
all eight positive/control pairs passed in 828.306 seconds. The mixed
`original-focused14.json` retains both the failed AML package and passing
retirement package; its overall exit is 1. The accompanying audit verifies all
145 inputs, runner, generated bodies, build text and sixteen retirement results.
This does not establish renamed-candidate or bytecode execution success.
The checker now prints failed-batch diagnostics as soon as a batch completes.
That reporting change does not alter the selected bodies or expected outcomes.
