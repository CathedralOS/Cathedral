def require($condition; $message):
  if $condition then . else error($message) end;

def field_value($literal; $name):
  first($literal.fields[] | select(.name == $name) | .value);

.[0] as $typed
| .[1] as $contracts
| [
    $typed
    | ..
    | objects
    | select(.name? == "x86_exception_vector_snapshot")
  ] as $machines
| [
    $contracts
    | ..
    | objects
    | select(.machine? == "x86_exception_vector_snapshot")
  ] as $machine_contracts
| require($machines | length == 1;
    "expected exactly one typed x86_exception_vector_snapshot machine")
| require($machine_contracts | length == 1;
    "expected exactly one x86_exception_vector_snapshot contract")
| $machines[0] as $machine
| $machine_contracts[0] as $machine_contract
| require(($machine.states | length) == 1 and
          $machine.states[0].name == "entry" and
          $machine.states[0].return_type == { kind: "named", name: "X86ExceptionVectorSnapshot" } and
          ($machine.states[0].statements | length) == 1 and
          $machine.states[0].statements[0].kind == "expression" and
          $machine.states[0].statements[0].value.type_name == "X86ExceptionVectorSnapshot";
    "exception-vector snapshot no longer has one pure record-producing state")
| $machine.states[0].statements[0].value as $snapshot
| field_value($snapshot; "vector_count") as $vector_count
| field_value($snapshot; "vectors") as $vectors
| require($vector_count.kind == "integer" and $vector_count.text == "32";
    "x86 exception-vector table cardinality changed")
| require($vectors.kind == "array_literal";
    "x86 exception-vector identities are no longer emitted as one fixed table")
| [$vectors.values[] | .text] as $actual_vectors
| [range(0; 32) | tostring] as $expected_vectors
| require($actual_vectors == $expected_vectors;
    "x86 exception-vector names no longer cover their exact architectural slots 0 through 31")
| require(($machine_contract.implementation.checked_may_suspend == false) and
          ($machine_contract.implementation.checked_may_block == false) and
          ($machine_contract.implementation.checked_service_reach == []) and
          ($machine_contract.implementation.checked_synchronous_invocations == []) and
          ($machine_contract.implementation.checked_crash_sites == []) and
          ($machine_contract.implementation.checked_crash_calls == []) and
          ($machine_contract.implementation.checked_termination.kind == "terminates") and
          ($machine_contract.implementation.inferred_write_frames | length) == 1 and
          ($machine_contract.implementation.inferred_write_frames[0].completeness == "complete") and
          ($machine_contract.implementation.inferred_write_frames[0].paths == []);
    "pure exception-vector evaluation gained effects, writes, calls, crashes, or nontermination")
| true
