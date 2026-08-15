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
| [
    $typed
    | ..
    | objects
    | select(.name? == "X86ExceptionDeliveryShape" and has("members"))
  ] as $delivery_types
| [
    $typed
    | ..
    | objects
    | select(.name? == "x86_exception_delivery_shape" and has("states"))
  ] as $delivery_machines
| [
    $contracts
    | ..
    | objects
    | select(.machine? == "x86_exception_delivery_shape")
  ] as $delivery_contracts
| require($machines | length == 1;
    "expected exactly one typed x86_exception_vector_snapshot machine")
| require($machine_contracts | length == 1;
    "expected exactly one x86_exception_vector_snapshot contract")
| require(($delivery_types | length) == 1 and
          ($delivery_machines | length) == 1 and
          ($delivery_contracts | length) == 1;
    "expected exactly one x86 exception delivery-shape fact")
| $machines[0] as $machine
| $machine_contracts[0] as $machine_contract
| $delivery_types[0] as $delivery_type
| $delivery_machines[0] as $delivery_machine
| $delivery_contracts[0] as $delivery_contract
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
| require(($delivery_type.members | map({name, payload})) == [
      {name: "NoArchitecturalException", payload: []},
      {name: "WithoutErrorCode", payload: []},
      {name: "WithErrorCode", payload: []},
      {name: "CpuProfileRequired", payload: []}
    ];
    "x86 exception delivery-shape categories changed")
| require(($delivery_machine.states | map(.name)) == [
      "entry",
      "vendor_profile",
      "reserved",
      "no_architectural_exception",
      "without_error_code",
      "with_error_code",
      "cpu_profile_required"
    ] and
    $delivery_machine.states[0].parameters == [{
      name: "vector",
      type_reference: {
        kind: "constrained",
        base_type: {kind: "named", name: "u8"},
        constraints: [{
          kind: "range",
          minimum: {kind: "integer", text: "0"},
          maximum: {kind: "integer", text: "31"}
        }]
      },
      is_const: false,
      is_mutable: false,
      is_self: false
    }] and
    all($delivery_machine.states[];
      .return_type == {kind: "named", name: "X86ExceptionDeliveryShape"});
    "x86 exception delivery-shape machine changed its total seven-state interface")
| [
    $delivery_machine.states[0].statements[0].guard
    | ..
    | objects
    | select(.kind? == "binary" and .operator? == "==" and
        .left.path? == ["vector"] and .right.kind? == "integer")
    | .right.text
  ] as $error_code_vectors
| require($error_code_vectors == ["8", "10", "11", "12", "13", "14", "17", "21"] and
          $delivery_machine.states[0].statements[0].target.path == ["with_error_code"] and
          $delivery_machine.states[0].statements[1].target.path == ["vendor_profile"] and
          $delivery_machine.states[0].statements[1].guard == {kind: "always"};
    "x86 hardware-error-code exception set changed")
| [
    $delivery_machine.states[1].statements[0].guard
    | ..
    | objects
    | select(.kind? == "binary" and
        (.operator? == ">=" or .operator? == "<=") and
        .left.path? == ["vector"])
    | {operator, value: .right.text}
  ] as $vendor_bounds
| require($vendor_bounds == [
      {operator: ">=", value: "28"},
      {operator: "<=", value: "30"}
    ] and
    $delivery_machine.states[1].statements[0].target.path == ["cpu_profile_required"] and
    $delivery_machine.states[1].statements[1].target.path == ["reserved"] and
    $delivery_machine.states[1].statements[1].guard == {kind: "always"};
    "optional AMD exception slots no longer require an explicit CPU profile")
| [
    $delivery_machine.states[2].statements[0].guard
    | ..
    | objects
    | select(.kind? == "binary" and .operator? == "==" and
        .left.path? == ["vector"] and .right.kind? == "integer")
    | .right.text
  ] as $individual_reserved_vectors
| [
    $delivery_machine.states[2].statements[0].guard
    | ..
    | objects
    | select(.kind? == "binary" and
        (.operator? == ">=" or .operator? == "<=") and
        .left.path? == ["vector"])
    | {operator, value: .right.text}
  ] as $reserved_bounds
| require($individual_reserved_vectors == ["9", "15", "31"] and
          $reserved_bounds == [
            {operator: ">=", value: "22"},
            {operator: "<=", value: "27"}
          ] and
          $delivery_machine.states[2].statements[0].target.path ==
            ["no_architectural_exception"] and
          $delivery_machine.states[2].statements[1].target.path ==
            ["without_error_code"] and
          $delivery_machine.states[2].statements[1].guard == {kind: "always"};
    "reserved x86 exception slots changed classification")
| require([
      $delivery_machine.states[3:][]
      | .statements[0].value.path
    ] == [
      ["X86ExceptionDeliveryShape", "NoArchitecturalException"],
      ["X86ExceptionDeliveryShape", "WithoutErrorCode"],
      ["X86ExceptionDeliveryShape", "WithErrorCode"],
      ["X86ExceptionDeliveryShape", "CpuProfileRequired"]
    ];
    "x86 exception delivery-shape terminal categories changed")
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
| require(($delivery_contract.implementation.checked_may_suspend == false) and
          ($delivery_contract.implementation.checked_may_block == false) and
          ($delivery_contract.implementation.checked_service_reach == []) and
          ($delivery_contract.implementation.checked_synchronous_invocations == []) and
          ($delivery_contract.implementation.checked_crash_sites == []) and
          ($delivery_contract.implementation.checked_crash_calls == []) and
          ($delivery_contract.implementation.checked_termination.kind == "terminates") and
          ($delivery_contract.implementation.inferred_write_frames | length) == 7 and
          all($delivery_contract.implementation.inferred_write_frames[];
            .completeness == "complete" and .paths == []);
    "exception delivery-shape fact gained effects, writes, calls, crashes, or nontermination")
| true
