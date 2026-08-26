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
  (.contract.service_reach.interface == "internal_inferred" or
   .contract.service_reach.interface == "published_ceiling") and
  (.contract.service_reach.services // []) == [] and
  (.contract.synchronous_invocation.interface == "internal_inferred" or
   .contract.synchronous_invocation.interface == "published_ceiling") and
  .contract.synchronous_invocation.targets == [] and
  .implementation.checked_service_reach == [] and
  .implementation.checked_synchronous_invocations == [] and
  .implementation.checked_may_suspend == false and
  .implementation.checked_may_block == false and
  .implementation.checked_crash_sites == [] and
  .implementation.checked_termination.kind == "terminates";

.[0] as $typed
| .[1] as $contracts
| typed_named($typed; "X86BootstrapSingleEntryPageImageCandidate"; "members") as $candidate_types
| typed_named($typed; "X86BootstrapSingleEntryPageImageCheck"; "members") as $check_types
| typed_named($typed; "x86_page_table_entries_are_equal"; "states") as $equality_helpers
| typed_named($typed; "x86_bootstrap_single_entry_page_slot_is_consistent"; "states") as $slot_helpers
| typed_named($typed; "x86_validate_bootstrap_single_entry_page_image"; "states") as $validators
| typed_named($typed; "x86_scan_bootstrap_single_entry_page_image"; "states") as $scanners
| [
    $contracts.machines[]
    | select(.machine as $name | [
        "x86_page_table_entries_are_equal",
        "x86_bootstrap_single_entry_page_slot_is_consistent",
        "x86_validate_bootstrap_single_entry_page_image",
        "x86_scan_bootstrap_single_entry_page_image"
      ] | index($name))
  ] as $machine_contracts
| require(($candidate_types | length) == 1 and
          ($check_types | length) == 1 and
          ($equality_helpers | length) == 1 and
          ($slot_helpers | length) == 1 and
          ($validators | length) == 1 and
          ($scanners | length) == 1 and
          ($machine_contracts | length) == 4;
    "expected one single-entry page-image validation frontier")
| $candidate_types[0] as $candidate_type
| $check_types[0] as $check_type
| $equality_helpers[0] as $equality
| $slot_helpers[0] as $slot_helper
| $validators[0] as $validator
| $scanners[0] as $scanner
| require([
      $candidate_type.members[]
      | {kind, name, relevance, type_name: .type_reference.name}
    ] == [
      {
        kind: "field",
        name: "step",
        relevance: "relevant",
        type_name: "X86BootstrapPageTableWalkStep"
      },
      {
        kind: "field",
        name: "page",
        relevance: "relevant",
        type_name: "X86PageTablePageCandidate"
      }
    ];
    "page-image candidate lost its exact step and complete page")
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
        name: "ImageConsistent",
        payload: [{
          identity: null,
          name: "candidate",
          relevance: "relevant",
          type_reference: {
            kind: "named",
            name: "X86BootstrapSingleEntryPageImageCandidate"
          }
        }],
        retired_payload_identities: []
      }
    ];
    "page-image result no longer rejects or returns the complete candidate")
| [
    $equality.states[0].statements[0].value
    | ..
    | objects
    | select(.kind? == "member")
    | expression_path
  ] | unique as $equality_paths
| [
    "present",
    "writable",
    "user",
    "write_through",
    "cache_disable",
    "accessed",
    "dirty",
    "page_size_or_pat",
    "global",
    "software_low",
    "frame_number",
    "software_high",
    "protection_key",
    "no_execute"
  ] as $entry_fields
| [$entry_fields[] as $field | ["left", $field]] as $left_paths
| [$entry_fields[] as $field | ["right", $field]] as $right_paths
| [
    $equality.states[0].statements[0].value
    | ..
    | objects
    | select(.kind? == "binary")
    | .operator
  ] as $equality_operators
| require(($equality.states | map(.name)) == ["entry"] and
          ($equality_paths - ($left_paths + $right_paths)) == [] and
          (($left_paths + $right_paths) - $equality_paths) == [] and
          ($equality_operators | map(select(. == "==")) | length) == 14 and
          ($equality_operators | map(select(. == "&&")) | length) == 13;
    "selected-entry equality must compare every schema field exactly once")
| require(($slot_helper.states | map(.name)) == [
      "entry",
      "compare_selected_bootstrap_page_entry",
      "compare_zero_bootstrap_page_entry"
    ] and
    $slot_helper.states[0].statements[0].target.path ==
      ["compare_selected_bootstrap_page_entry"] and
    $slot_helper.states[0].statements[0].guard.value.left == {
      kind: "name",
      path: ["selected"]
    } and
    $slot_helper.states[0].statements[1].target.path ==
      ["compare_zero_bootstrap_page_entry"] and
    $slot_helper.states[0].statements[1].guard.kind == "always";
    "slot policy must branch selected equality versus unselected exact zero")
| [
    $slot_helper.states[]
    | ..
    | objects
    | select(.kind? == "call")
    | {target, arguments}
  ] as $slot_calls
| require($slot_calls == [
      {
        target: "x86_page_table_entries_are_equal",
        arguments: [
          {kind: "name", path: ["entry"]},
          {kind: "name", path: ["expected"]}
        ]
      },
      {
        target: "x86_page_table_entry_is_zero",
        arguments: [{kind: "name", path: ["entry"]}]
      }
    ];
    "slot policy lost selected field equality or unselected zero validation")
| require(($validator.states | map(.name)) == [
      "entry",
      "reject_bootstrap_single_entry_page_image"
    ];
    "page-image validator changed its exact prerequisite interface")
| $validator.states[0].statements as $validator_entry
| $validator_entry[0].initial_value as $geometry_call
| $validator_entry[1].target as $scanner_target
| require($geometry_call.kind == "call" and
          $geometry_call.target ==
            "x86_bootstrap_page_table_walk_step_is_consistent" and
          $geometry_call.arguments == [
            {
              kind: "member",
              receiver: {kind: "name", path: ["candidate"]},
              member: "step"
            },
            {
              kind: "member",
              receiver: {
                kind: "member",
                receiver: {kind: "name", path: ["candidate"]},
                member: "step"
              },
              member: "index"
            }
          ] and
          $scanner_target.kind == "value" and
          $scanner_target.value.kind == "call" and
          $scanner_target.value.target ==
            "x86_scan_bootstrap_single_entry_page_image" and
          $scanner_target.value.arguments == [
            {kind: "name", path: ["candidate"]},
            {kind: "integer", text: "512"},
            {kind: "boolean", value: true}
          ] and
          $validator_entry[1].guard.value.left == {
            kind: "binary",
            left: {kind: "name", path: ["step_consistent"]},
            operator: "&&",
            right: {
              kind: "member",
              receiver: {
                kind: "member",
                receiver: {
                  kind: "member",
                  receiver: {
                    kind: "member",
                    receiver: {kind: "name", path: ["candidate"]},
                    member: "step"
                  },
                  member: "entry"
                },
                member: "entry"
              },
              member: "present"
            }
          } and
          $validator_entry[2].target.path ==
            ["reject_bootstrap_single_entry_page_image"] and
          $validator_entry[2].guard.kind == "always";
    "page-image validation must require exact step geometry and enter the scanner through an explicit value call")
| require(($scanner.states | map(.name)) == [
      "entry",
      "settle_bootstrap_single_entry_page_image",
      "accept_bootstrap_single_entry_page_image",
      "reject_bootstrap_single_entry_page_image"
    ] and
    $scanner.termination_witness == {
      subjects: ["remaining"],
      ranking_view: 1,
      view_path: "Nat::Descending",
      view_arguments: [],
      rank_range: null
    };
    "page-image scan lost its exact checked 512-step ranking")
| $scanner.states[0].statements as $scan
| require(($scan | length) == 7 and
          $scan[0].name == "index" and
          $scan[0].type_reference.constraints[0].minimum.text == "0" and
          $scan[0].type_reference.constraints[0].maximum.text == "511" and
          $scan[0].initial_value == {
            kind: "binary",
            left: {kind: "integer", text: "512"},
            operator: "-",
            right: {kind: "name", path: ["remaining"]}
          } and
          $scan[1].initial_value.collection == {
            kind: "member",
            receiver: {
              kind: "member",
              receiver: {kind: "name", path: ["candidate"]},
              member: "page"
            },
            member: "entries"
          } and
          $scan[1].initial_value.index == {kind: "name", path: ["index"]} and
          $scan[2].initial_value == {
            kind: "binary",
            left: {kind: "name", path: ["index"]},
            operator: "==",
            right: {
              kind: "member",
              receiver: {
                kind: "member",
                receiver: {kind: "name", path: ["candidate"]},
                member: "step"
              },
              member: "index"
            }
          } and
          $scan[3].initial_value.target ==
            "x86_bootstrap_single_entry_page_slot_is_consistent" and
          $scan[3].initial_value.arguments == [
            {kind: "name", path: ["entry"]},
            {
              kind: "member",
              receiver: {
                kind: "member",
                receiver: {
                  kind: "member",
                  receiver: {kind: "name", path: ["candidate"]},
                  member: "step"
                },
                member: "entry"
              },
              member: "entry"
            },
            {kind: "name", path: ["selected"]}
          ] and
          $scan[4].initial_value == {
            kind: "binary",
            left: {kind: "name", path: ["consistent_so_far"]},
            operator: "&&",
            right: {kind: "name", path: ["slot_consistent"]}
          } and
          $scan[5].target.path ==
            ["x86_scan_bootstrap_single_entry_page_image"] and
          $scan[5].target.arguments[1] == {
            kind: "binary",
            left: {kind: "name", path: ["remaining"]},
            operator: "-",
            right: {kind: "integer", text: "1"}
          } and
          $scan[6].target.path ==
            ["settle_bootstrap_single_entry_page_image"];
    "page-image scan changed its exact slot coverage or decreasing recursion")
| require($scanner.states[2].statements == [{
      kind: "expression",
      value: {
        kind: "struct_literal",
        type_name: "X86BootstrapSingleEntryPageImageCheck",
        fields: [{
          name: "candidate",
          value: {kind: "name", path: ["candidate"]}
        }]
      }
    }] and
    $scanner.states[3].statements == [{
      kind: "expression",
      value: {
        kind: "name",
        path: ["X86BootstrapSingleEntryPageImageCheck", "Rejected"]
      }
    }];
    "page-image validation must return the complete exact candidate or reject")
| [$machine_contracts[] | select(.machine == "x86_page_table_entries_are_equal")][0] as $equality_contract
| [$machine_contracts[] | select(.machine == "x86_bootstrap_single_entry_page_slot_is_consistent")][0] as $slot_contract
| [$machine_contracts[] | select(.machine == "x86_validate_bootstrap_single_entry_page_image")][0] as $validator_contract
| [$machine_contracts[] | select(.machine == "x86_scan_bootstrap_single_entry_page_image")][0] as $scanner_contract
| require(($equality_contract | pure_contract) and
          $equality_contract.implementation.checked_crash_calls == [] and
          [$equality_contract.implementation.inferred_write_frames[] | {
            state, completeness, paths
          }] == [{state: "entry", completeness: "complete", paths: []}];
    "entry equality gained effects, writes, calls, or nontermination")
| require(($slot_contract | pure_contract) and
          [$slot_contract.implementation.checked_crash_calls[] | {
            state, target_machine, surviving_buckets
          }] == [
            {
              state: "compare_selected_bootstrap_page_entry",
              target_machine: "x86_page_table_entries_are_equal",
              surviving_buckets: []
            },
            {
              state: "compare_zero_bootstrap_page_entry",
              target_machine: "x86_page_table_entry_is_zero",
              surviving_buckets: []
            }
          ] and
          all($slot_contract.implementation.inferred_write_frames[];
            .completeness == "complete" and .paths == []);
    "slot policy gained effects, writes, crash survivors, or nontermination")
| require(($validator_contract | pure_contract) and
          [$validator_contract.implementation.checked_crash_calls[] | {
            state, target_machine, surviving_buckets
          }] == [
            {
              state: "entry",
              target_machine:
                "x86_bootstrap_page_table_walk_step_is_consistent",
              surviving_buckets: []
            },
            {
              state: "entry",
              target_machine:
                "x86_scan_bootstrap_single_entry_page_image",
              surviving_buckets: []
            }
          ] and
          [$validator_contract.implementation.inferred_write_frames[] | {
            state, completeness, paths
          }] == [
            {
              state: "entry",
              completeness: "complete",
              paths: ["$P0", "self"]
            },
            {
              state: "reject_bootstrap_single_entry_page_image",
              completeness: "complete",
              paths: []
            }
          ];
    "page-image validation gained effects, external writes, crash survivors, or nontermination")
| require(($scanner_contract | pure_contract) and
          [$scanner_contract.implementation.checked_crash_calls[] | {
            state, target_machine, surviving_buckets
          }] == [{
            state: "entry",
            target_machine:
              "x86_bootstrap_single_entry_page_slot_is_consistent",
            surviving_buckets: []
          }] and
          $scanner_contract.implementation.resolved_ranking_view ==
            "Nat::Descending" and
          $scanner_contract.implementation.ranking_witness == {
            subjects: ["remaining"],
            view: "Nat::Descending",
            view_arguments: []
          } and
          all($scanner_contract.implementation.inferred_write_frames[];
            .paths == []);
    "page-image scanner lost ranking or gained effects, writes, or crash survivors")
| true
