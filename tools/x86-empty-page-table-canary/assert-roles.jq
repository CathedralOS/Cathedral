def require($condition; $message):
  if $condition then . else error($message) end;

def typed_named($typed; $name; $member):
  [
    $typed
    | ..
    | objects
    | select(.name? == $name and has($member))
  ];

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
  .contract.service_reach == {
    interface: "published_ceiling",
    services: []
  } and
  .contract.synchronous_invocation == {
    interface: "published_ceiling",
    targets: []
  } and
  .contract.suspension == {
    interface: "published_ceiling",
    may_suspend: false
  } and
  .contract.blocking == {
    interface: "published_ceiling",
    may_block: false
  } and
  .contract.crashes == {
    interface: "published_ceiling",
    buckets: []
  } and
  .implementation.checked_service_reach == [] and
  .implementation.checked_synchronous_invocations == [] and
  .implementation.checked_may_suspend == false and
  .implementation.checked_may_block == false and
  .implementation.checked_crash_sites == [] and
  .implementation.checked_termination.kind == "terminates";

.[0] as $typed
| .[1] as $contracts
| typed_named($typed; "X86BootstrapFourLevelWalkRoleCheck"; "members") as $check_types
| typed_named($typed; "x86_validate_bootstrap_four_level_walk_roles"; "states") as $validators
| [
    $contracts.machines[]
    | select(.machine == "x86_validate_bootstrap_four_level_walk_roles")
  ] as $machine_contracts
| require(($check_types | length) == 1 and
          ($validators | length) == 1 and
          ($machine_contracts | length) == 1;
    "expected one four-level walk role-validation frontier")
| $check_types[0] as $check_type
| $validators[0] as $validator
| $machine_contracts[0] as $contract
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
        name: "RoleConsistent",
        payload: [{
          identity: null,
          name: "descriptor",
          relevance: "relevant",
          type_reference: {
            kind: "named",
            name: "X86BootstrapFourLevelWalkDescriptor"
          }
        }],
        retired_payload_identities: []
      }
    ];
    "role check no longer rejects or returns the exact walk descriptor")
| require(($validator.states | map(.name)) == [
      "entry",
      "validate_four_level_walk_roles",
      "accept_four_level_walk_roles",
      "reject_four_level_walk_roles"
    ];
    "role validator changed its exact checked-state interface")
| [$validator.states[] | select(.name == "entry")][0] as $entry
| [$validator.states[] | select(.name == "validate_four_level_walk_roles")][0] as $roles
| [$validator.states[] | select(.name == "accept_four_level_walk_roles")][0] as $accept
| [$validator.states[] | select(.name == "reject_four_level_walk_roles")][0] as $reject
| require($entry.parameters == [{
      name: "candidate",
      type_reference: {
        kind: "named",
        name: "X86BootstrapFourLevelWalkDescriptor"
      },
      is_const: false,
      is_mutable: false,
      is_self: false
    }] and
    $entry.return_type == {
      kind: "named",
      name: "X86BootstrapFourLevelWalkRoleCheck"
    };
    "role validator lost its exact descriptor input/result shape")
| [
    $entry.statements[]
    | ..
    | objects
    | select(.kind? == "call")
  ] as $numeric_calls
| require(($numeric_calls | length) == 1 and
          $numeric_calls[0].target ==
            "x86_validate_bootstrap_four_level_walk_descriptor" and
          $numeric_calls[0].arguments == [{kind: "name", path: ["candidate"]}];
    "role validation must first replay the exact numeric descriptor validator")
| [
    $entry.statements[]
    | select(.kind == "transition")
  ] as $entry_transitions
| require(($entry_transitions | length) == 2 and
          $entry_transitions[0].target.path ==
            ["validate_four_level_walk_roles"] and
          $entry_transitions[0].target.arguments == [{
            kind: "member",
            receiver: {kind: "name", path: ["numeric"]},
            member: "descriptor"
          }] and
          $entry_transitions[0].guard.value == {
            kind: "binary",
            left: {kind: "name", path: ["numeric"]},
            operator: "==",
            right: {
              kind: "name",
              path: [
                "X86BootstrapFourLevelWalkDescriptorCheck",
                "NumericallyConsistent"
              ]
            }
          } and
          $entry_transitions[1].target.path ==
            ["reject_four_level_walk_roles"] and
          $entry_transitions[1].target.arguments == [] and
          $entry_transitions[1].guard.kind == "always";
    "only a numerically consistent payload may reach role checking")
| [
    $roles.statements[]
    | select(.kind == "transition")
  ] as $role_transitions
| require(($role_transitions | length) == 2 and
          $role_transitions[0].target.path ==
            ["accept_four_level_walk_roles"] and
          $role_transitions[0].target.arguments == [{
            kind: "name",
            path: ["descriptor"]
          }] and
          $role_transitions[1].target.path ==
            ["reject_four_level_walk_roles"];
    "role checking must return the exact descriptor or reject")
| [
    $role_transitions[0].guard
    | ..
    | objects
    | select(.kind? == "member")
    | expression_path
    | select(length == 5)
  ] | unique as $role_paths
| [
    ["descriptor", "pml4", "entry", "entry", "present"],
    ["descriptor", "pml4", "entry", "entry", "page_size_or_pat"],
    ["descriptor", "pdpt", "entry", "entry", "present"],
    ["descriptor", "pdpt", "entry", "entry", "page_size_or_pat"],
    ["descriptor", "pd", "entry", "entry", "present"],
    ["descriptor", "pd", "entry", "entry", "page_size_or_pat"],
    ["descriptor", "pt", "entry", "entry", "present"]
  ] as $expected_role_paths
| [
    $role_transitions[0].guard
    | ..
    | objects
    | select(.kind? == "unary" and .operator == "!")
    | .operand
    | expression_path
  ] | unique as $negated_role_paths
| [
    ["descriptor", "pml4", "entry", "entry", "page_size_or_pat"],
    ["descriptor", "pdpt", "entry", "entry", "page_size_or_pat"],
    ["descriptor", "pd", "entry", "entry", "page_size_or_pat"]
  ] as $expected_negated_role_paths
| require(($role_paths | length) == 7 and
          ($role_paths - $expected_role_paths) == [] and
          ($expected_role_paths - $role_paths) == [] and
          ($negated_role_paths - $expected_negated_role_paths) == [] and
          ($expected_negated_role_paths - $negated_role_paths) == [];
    "role validation must inspect only upper present/non-large-page and PT-present fields")
| require($accept.statements == [{
      kind: "expression",
      value: {
        kind: "struct_literal",
        type_name: "X86BootstrapFourLevelWalkRoleCheck",
        fields: [{
          name: "descriptor",
          value: {kind: "name", path: ["descriptor"]}
        }]
      }
    }] and
    $reject.statements == [{
      kind: "expression",
      value: {
        kind: "name",
        path: ["X86BootstrapFourLevelWalkRoleCheck", "Rejected"]
      }
    }];
    "role result must preserve the exact descriptor and expose no partial success")
| require(($contract | pure_contract) and
          $contract.implementation.checked_crash_calls == [{
            state: "entry",
            statement_ordinal: 0,
            call_ordinal: 0,
            target_machine:
              "x86_validate_bootstrap_four_level_walk_descriptor",
            target_callable_overload_identity:
              $contract.implementation.checked_crash_calls[0].target_callable_overload_identity,
            target_state: "entry",
            target_contract_fingerprint:
              $contract.implementation.checked_crash_calls[0].target_contract_fingerprint,
            path_guard_conjuncts: [],
            path_guard_consequences: [],
            surviving_buckets: []
          }] and
          [$contract.implementation.inferred_write_frames[] | {
            state,
            completeness,
            paths
          }] == [
            {state: "entry", completeness: "complete", paths: []},
            {state: "validate_four_level_walk_roles", completeness: "complete", paths: []},
            {state: "accept_four_level_walk_roles", completeness: "complete", paths: []},
            {state: "reject_four_level_walk_roles", completeness: "complete", paths: []}
          ];
    "role validation gained effects, writes, crash survivors, suspension, blocking, or nontermination")
| true
