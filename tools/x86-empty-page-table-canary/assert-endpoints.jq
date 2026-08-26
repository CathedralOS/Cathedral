def require($condition; $message):
  if $condition then . else error($message) end;

def typed_named($typed; $name; $member):
  [
    $typed
    | ..
    | objects
    | select(.name? == $name and has($member))
  ];

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

def named_parameter($name; $type_name):
  {
    name: $name,
    type_reference: {kind: "named", name: $type_name},
    is_const: false,
    is_mutable: false,
    is_self: false
  };

.[0] as $typed
| .[1] as $contracts
| typed_named($typed; "X86BootstrapFourKilobyteWalkEndpoints"; "members") as $endpoint_types
| typed_named($typed; "X86BootstrapFourKilobyteWalkEndpointCheck"; "members") as $check_types
| typed_named($typed; "x86_validate_bootstrap_four_kilobyte_walk_endpoints"; "states") as $validators
| [
    $contracts.machines[]
    | select(.machine ==
        "x86_validate_bootstrap_four_kilobyte_walk_endpoints")
  ] as $machine_contracts
| require(($endpoint_types | length) == 1 and
          ($check_types | length) == 1 and
          ($validators | length) == 1 and
          ($machine_contracts | length) == 1;
    "expected one four-kilobyte walk endpoint-validation frontier")
| $endpoint_types[0] as $endpoint_type
| $check_types[0] as $check_type
| $validators[0] as $validator
| $machine_contracts[0] as $contract
| require([
      $endpoint_type.members[]
      | {kind, name, relevance, type_name: .type_reference.name}
    ] == [
      {
        kind: "field",
        name: "virtual_page_address",
        relevance: "relevant",
        type_name: "u64"
      },
      {
        kind: "field",
        name: "physical_frame_address",
        relevance: "relevant",
        type_name: "u64"
      },
      {
        kind: "field",
        name: "descriptor",
        relevance: "relevant",
        type_name: "X86BootstrapFourLevelWalkDescriptor"
      }
    ];
    "endpoint carrier lost its exact virtual/physical/descriptor custody")
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
        name: "EndpointConsistent",
        payload: [{
          identity: null,
          name: "candidate",
          relevance: "relevant",
          type_reference: {
            kind: "named",
            name: "X86BootstrapFourKilobyteWalkEndpoints"
          }
        }],
        retired_payload_identities: []
      }
    ];
    "endpoint check no longer rejects or returns the complete candidate")
| require(($validator.states | map(.name)) == [
      "entry",
      "validate_four_kilobyte_walk_endpoints",
      "accept_four_kilobyte_walk_endpoints",
      "reject_four_kilobyte_walk_endpoints"
    ];
    "endpoint validator changed its exact checked-state interface")
| [$validator.states[] | select(.name == "entry")][0] as $entry
| [$validator.states[] | select(.name == "validate_four_kilobyte_walk_endpoints")][0] as $endpoints
| [$validator.states[] | select(.name == "accept_four_kilobyte_walk_endpoints")][0] as $accept
| [$validator.states[] | select(.name == "reject_four_kilobyte_walk_endpoints")][0] as $reject
| require($entry.parameters == [
      named_parameter(
        "candidate";
        "X86BootstrapFourKilobyteWalkEndpoints"
      )
    ] and
    $entry.return_type == {
      kind: "named",
      name: "X86BootstrapFourKilobyteWalkEndpointCheck"
    } and
    $endpoints.parameters == [
      named_parameter("virtual_page_address"; "u64"),
      named_parameter("physical_frame_address"; "u64"),
      named_parameter(
        "descriptor";
        "X86BootstrapFourLevelWalkDescriptor"
      )
    ];
    "endpoint validator lost its exact candidate or retained-value parameters")
| [
    $entry.statements[]
    | ..
    | objects
    | select(.kind? == "call")
  ] as $role_calls
| require(($role_calls | length) == 1 and
          $role_calls[0].target ==
            "x86_validate_bootstrap_four_level_walk_roles" and
          $role_calls[0].arguments == [{
            kind: "member",
            receiver: {kind: "name", path: ["candidate"]},
            member: "descriptor"
          }];
    "endpoint validation must first replay the exact role validator")
| [
    $entry.statements[]
    | select(.kind == "transition")
  ] as $entry_transitions
| require(($entry_transitions | length) == 2 and
          $entry_transitions[0].target.path ==
            ["validate_four_kilobyte_walk_endpoints"] and
          $entry_transitions[0].target.arguments == [
            {kind: "name", path: ["virtual_page_address"]},
            {kind: "name", path: ["physical_frame_address"]},
            {
              kind: "member",
              receiver: {kind: "name", path: ["roles"]},
              member: "descriptor"
            }
          ] and
          $entry_transitions[0].guard.value == {
            kind: "binary",
            left: {kind: "name", path: ["roles"]},
            operator: "==",
            right: {
              kind: "name",
              path: [
                "X86BootstrapFourLevelWalkRoleCheck",
                "RoleConsistent"
              ]
            }
          } and
          $entry_transitions[1].target.path ==
            ["reject_four_kilobyte_walk_endpoints"] and
          $entry_transitions[1].target.arguments == [] and
          $entry_transitions[1].guard.kind == "always";
    "only the exact RoleConsistent payload may reach endpoint checking")
| [
    $endpoints.statements[]
    | select(.kind == "transition")
  ] as $endpoint_transitions
| require(($endpoint_transitions | length) == 2 and
          $endpoint_transitions[0].target.path ==
            ["accept_four_kilobyte_walk_endpoints"] and
          $endpoint_transitions[0].target.arguments == [
            {kind: "name", path: ["virtual_page_address"]},
            {kind: "name", path: ["physical_frame_address"]},
            {kind: "name", path: ["descriptor"]}
          ] and
          $endpoint_transitions[1].target.path ==
            ["reject_four_kilobyte_walk_endpoints"] and
          $endpoint_transitions[1].guard.kind == "always";
    "endpoint checking must return all exact values or reject")
| $endpoint_transitions[0].guard.value as $normalized_guard
| require($normalized_guard.operator == "==" and
          $normalized_guard.right == {kind: "boolean", value: true} and
          $normalized_guard.left == {
            kind: "binary",
            left: {
              kind: "binary",
              left: {
                kind: "binary",
                left: {
                  kind: "name",
                  path: ["virtual_page_address"]
                },
                operator: "==",
                right: {
                  kind: "member",
                  receiver: {
                    kind: "member",
                    receiver: {kind: "name", path: ["descriptor"]},
                    member: "indices"
                  },
                  member: "address"
                }
              },
              operator: "&&",
              right: {
                kind: "binary",
                left: {
                  kind: "member",
                  receiver: {
                    kind: "member",
                    receiver: {kind: "name", path: ["descriptor"]},
                    member: "indices"
                  },
                  member: "page_offset"
                },
                operator: "==",
                right: {kind: "integer", text: "0"}
              }
            },
            operator: "&&",
            right: {
              kind: "binary",
              left: {
                kind: "name",
                path: ["physical_frame_address"]
              },
              operator: "==",
              right: {
                kind: "member",
                receiver: {
                  kind: "member",
                  receiver: {
                    kind: "member",
                    receiver: {kind: "name", path: ["descriptor"]},
                    member: "pt"
                  },
                  member: "entry"
                },
                member: "physical_address"
              }
            }
          };
    "endpoint validation changed its exact two-address and zero-offset comparisons")
| require($accept.statements == [{
      kind: "expression",
      value: {
        kind: "struct_literal",
        type_name: "X86BootstrapFourKilobyteWalkEndpointCheck",
        fields: [{
          name: "candidate",
          value: {
            kind: "struct_literal",
            type_name: "X86BootstrapFourKilobyteWalkEndpoints",
            fields: [
              {
                name: "virtual_page_address",
                value: {kind: "name", path: ["virtual_page_address"]}
              },
              {
                name: "physical_frame_address",
                value: {kind: "name", path: ["physical_frame_address"]}
              },
              {
                name: "descriptor",
                value: {kind: "name", path: ["descriptor"]}
              }
            ]
          }
        }]
      }
    }] and
    $reject.statements == [{
      kind: "expression",
      value: {
        kind: "name",
        path: [
          "X86BootstrapFourKilobyteWalkEndpointCheck",
          "Rejected"
        ]
      }
    }];
    "endpoint result must retain the exact values and expose no partial success")
| require(($contract | pure_contract) and
          ($contract.implementation.checked_crash_calls | length) == 1 and
          $contract.implementation.checked_crash_calls[0].state == "entry" and
          $contract.implementation.checked_crash_calls[0].statement_ordinal == 2 and
          $contract.implementation.checked_crash_calls[0].call_ordinal == 0 and
          $contract.implementation.checked_crash_calls[0].target_machine ==
            "x86_validate_bootstrap_four_level_walk_roles" and
          $contract.implementation.checked_crash_calls[0].target_state == "entry" and
          $contract.implementation.checked_crash_calls[0].path_guard_conjuncts == [] and
          $contract.implementation.checked_crash_calls[0].path_guard_consequences == [] and
          $contract.implementation.checked_crash_calls[0].surviving_buckets == [] and
          [$contract.implementation.inferred_write_frames[] | {
            state,
            completeness,
            paths
          }] == [
            {state: "entry", completeness: "complete", paths: []},
            {state: "validate_four_kilobyte_walk_endpoints", completeness: "complete", paths: []},
            {state: "accept_four_kilobyte_walk_endpoints", completeness: "complete", paths: []},
            {state: "reject_four_kilobyte_walk_endpoints", completeness: "complete", paths: []}
          ];
    "endpoint validation gained effects, writes, crash survivors, suspension, blocking, or nontermination")
| true
