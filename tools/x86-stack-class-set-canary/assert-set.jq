def require($condition; $message):
  if $condition then . else error($message) end;

def typed_named($typed; $name; $member):
  [
    $typed
    | ..
    | objects
    | select(.name? == $name and has($member))
  ];

def field_value($literal; $name):
  first($literal.fields[] | select(.name == $name) | .value);

def expression_path:
  if .kind == "name" then
    .path
  elif .kind == "member" then
    [(.receiver | expression_path)[], .member]
  else
    null
  end;

def pure_contract:
  .contract.supply == "checked_body" and
  .contract.service_reach.interface == "internal_inferred" and
  .contract.synchronous_invocation == {
    interface: "internal_inferred",
    targets: []
  } and
  .implementation.checked_service_reach == [] and
  .implementation.checked_synchronous_invocations == [] and
  .implementation.checked_may_suspend == false and
  .implementation.checked_may_block == false and
  .implementation.checked_crash_sites == [] and
  .implementation.checked_termination == {kind: "terminates", premises: []};

.[0] as $typed
| .[1] as $contracts
| typed_named($typed; "X86BootstrapStackClassSet"; "members") as $set_types
| typed_named($typed; "X86BootstrapStackClassSetPolicyCheck"; "members") as $check_types
| typed_named($typed; "x86_bootstrap_stack_class_set"; "states") as $set_machines
| typed_named($typed; "x86_validate_bootstrap_stack_class_set"; "states") as $validators
| [
    $contracts.machines[]
    | select(
        .machine == "x86_bootstrap_stack_class_set" or
        .machine == "x86_validate_bootstrap_stack_class_set"
      )
  ] as $machine_contracts
| require(($set_types | length) == 1 and
          ($check_types | length) == 1 and
          ($set_machines | length) == 1 and
          ($validators | length) == 1 and
          ($machine_contracts | length) == 2;
    "expected one complete stack set, check type, author, validator, and two contracts")
| $set_types[0] as $set_type
| $check_types[0] as $check_type
| $set_machines[0] as $set_machine
| $validators[0] as $validator
| require([
      $set_type.members[]
      | {kind, name, relevance, type: .type_reference.name}
    ] == [
      {kind: "field", name: "double_fault", relevance: "relevant", type: "X86IstStackClass"},
      {kind: "field", name: "nmi", relevance: "relevant", type: "X86IstStackClass"},
      {kind: "field", name: "machine_check", relevance: "relevant", type: "X86IstStackClass"},
      {kind: "field", name: "maskable_irq", relevance: "relevant", type: "X86IstStackClass"}
    ];
    "bootstrap stack-class set lost a required role or made one proof-only")
| require($check_type.members == [
      {
        kind: "variant",
        identity: null,
        name: "Rejected",
        payload: [],
        retired_payload_identities: []
      },
      {
        kind: "variant",
        identity: null,
        name: "PolicyConsistent",
        payload: [{
          identity: null,
          name: "candidate",
          relevance: "relevant",
          type_reference: {kind: "named", name: "X86BootstrapStackClassSet"}
        }],
        retired_payload_identities: []
      }
    ];
    "stack-class validation no longer returns the complete checked candidate")
| require(($set_machine.states | length) == 1 and
          $set_machine.states[0].name == "entry" and
          $set_machine.states[0].parameters == [] and
          $set_machine.states[0].return_type == {
            kind: "named",
            name: "X86BootstrapStackClassSet"
          } and
          ($set_machine.states[0].statements | length) == 1 and
          $set_machine.states[0].statements[0].kind == "expression";
    "bootstrap stack-class author is no longer one pure record-producing state")
| $set_machine.states[0].statements[0].value as $set_literal
| require($set_literal.type_name == "X86BootstrapStackClassSet" and
          [
            $set_literal.fields[]
            | {
                role: .name,
                stack_class: (field_value(.value; "stack_class").text | tonumber),
                ist_index: (field_value(.value; "ist_index").text | tonumber)
              }
          ] == [
            {role: "double_fault", stack_class: 1, ist_index: 1},
            {role: "nmi", stack_class: 2, ist_index: 2},
            {role: "machine_check", stack_class: 3, ist_index: 3},
            {role: "maskable_irq", stack_class: 4, ist_index: 4}
          ];
    "bootstrap stack roles no longer select the exact coupled class/IST records")
| require(($validator.states | map(.name)) == [
      "entry",
      "accept_stack_class_set",
      "reject_stack_class_set"
    ] and
    all($validator.states[];
      .return_type == {
        kind: "named",
        name: "X86BootstrapStackClassSetPolicyCheck"
      }
    ) and
    $validator.states[0].statements[0].initial_value == {
      kind: "call",
      receiver: null,
      target: "x86_bootstrap_stack_class_set",
      machine_arguments: [],
      arguments: [],
      acknowledgement_synthesized: false,
      acknowledges_suspend: false,
      acknowledges_block: false
    };
    "stack-class validator no longer derives Cathedral's expected set internally")
| [
    $validator.states[0].statements[1].initial_value
    | ..
    | objects
    | select(.kind? == "binary" and .operator? == "==")
    | {left: (.left | expression_path), right: (.right | expression_path)}
  ] as $comparisons
| require($comparisons == [
      {left: ["candidate", "double_fault", "stack_class"], right: ["expected", "double_fault", "stack_class"]},
      {left: ["candidate", "double_fault", "ist_index"], right: ["expected", "double_fault", "ist_index"]},
      {left: ["candidate", "nmi", "stack_class"], right: ["expected", "nmi", "stack_class"]},
      {left: ["candidate", "nmi", "ist_index"], right: ["expected", "nmi", "ist_index"]},
      {left: ["candidate", "machine_check", "stack_class"], right: ["expected", "machine_check", "stack_class"]},
      {left: ["candidate", "machine_check", "ist_index"], right: ["expected", "machine_check", "ist_index"]},
      {left: ["candidate", "maskable_irq", "stack_class"], right: ["expected", "maskable_irq", "stack_class"]},
      {left: ["candidate", "maskable_irq", "ist_index"], right: ["expected", "maskable_irq", "ist_index"]}
    ];
    "stack-class validator no longer checks every role's class and IST index")
| require($validator.states[1].statements[0].value.type_name ==
            "X86BootstrapStackClassSetPolicyCheck" and
          field_value($validator.states[1].statements[0].value; "candidate").path ==
            ["candidate"] and
          $validator.states[2].statements[0].value.path ==
            ["X86BootstrapStackClassSetPolicyCheck", "Rejected"];
    "stack-class validator no longer returns only the complete candidate or rejection")
| [$machine_contracts[] | select(.machine == "x86_bootstrap_stack_class_set")][0] as $set_contract
| [$machine_contracts[] | select(.machine == "x86_validate_bootstrap_stack_class_set")][0] as $validator_contract
| require(($set_contract | pure_contract) and
          $set_contract.implementation.checked_crash_calls == [] and
          [$set_contract.implementation.inferred_write_frames[] | {
            state,
            completeness,
            paths
          }] == [{state: "entry", completeness: "complete", paths: []}];
    "stack-class author gained effects, calls, writes, blocking, crashes, or nontermination")
| require(($validator_contract | pure_contract) and
          ($validator_contract.implementation.checked_crash_calls | length) == 1 and
          $validator_contract.implementation.checked_crash_calls[0].state == "entry" and
          $validator_contract.implementation.checked_crash_calls[0].statement_ordinal == 0 and
          $validator_contract.implementation.checked_crash_calls[0].call_ordinal == 0 and
          $validator_contract.implementation.checked_crash_calls[0].target_machine ==
            "x86_bootstrap_stack_class_set" and
          $validator_contract.implementation.checked_crash_calls[0].surviving_buckets == [] and
          [$validator_contract.implementation.inferred_write_frames[] | {
            state,
            completeness,
            paths
          }] == [
            {state: "entry", completeness: "complete", paths: []},
            {state: "accept_stack_class_set", completeness: "complete", paths: []},
            {state: "reject_stack_class_set", completeness: "complete", paths: []}
          ];
    "stack-class validator gained effects, writes, blocking, crashes, or nontermination")
| true
