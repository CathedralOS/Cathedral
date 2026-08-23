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
  .implementation.checked_termination.kind == "terminates";

.[0] as $typed
| .[1] as $contracts
| typed_named($typed; "X86BootstrapPageTableWalkStep"; "members") as $step_types
| typed_named($typed; "X86BootstrapFourLevelWalkDescriptor"; "members") as $descriptor_types
| typed_named($typed; "X86BootstrapFourLevelWalkDescriptorCheck"; "members") as $check_types
| typed_named($typed; "x86_bootstrap_page_table_walk_step_is_consistent"; "states") as $step_helpers
| typed_named($typed; "x86_validate_bootstrap_four_level_walk_descriptor"; "states") as $validators
| [
    $contracts.machines[]
    | select(.machine == "x86_bootstrap_page_table_walk_step_is_consistent" or
             .machine == "x86_validate_bootstrap_four_level_walk_descriptor")
  ] as $machine_contracts
| require(($step_types | length) == 1 and
          ($descriptor_types | length) == 1 and
          ($check_types | length) == 1 and
          ($step_helpers | length) == 1 and
          ($validators | length) == 1 and
          ($machine_contracts | length) == 2;
    "expected one four-level numeric walk descriptor frontier")
| $step_types[0] as $step_type
| $descriptor_types[0] as $descriptor_type
| $check_types[0] as $check_type
| $step_helpers[0] as $step_helper
| $validators[0] as $validator
| require([
      $step_type.members[]
      | {
          name,
          relevance,
          type_kind: .type_reference.kind,
          type_name: (.type_reference.name // .type_reference.base_type.name),
          minimum: (.type_reference.constraints[0].minimum.text // null),
          maximum: (.type_reference.constraints[0].maximum.text // null)
        }
    ] == [
      {
        name: "table_address",
        relevance: "relevant",
        type_kind: "named",
        type_name: "u64",
        minimum: null,
        maximum: null
      },
      {
        name: "index",
        relevance: "relevant",
        type_kind: "constrained",
        type_name: "u64",
        minimum: "0",
        maximum: "511"
      },
      {
        name: "entry_address",
        relevance: "relevant",
        type_kind: "named",
        type_name: "u64",
        minimum: null,
        maximum: null
      },
      {
        name: "entry",
        relevance: "relevant",
        type_kind: "named",
        type_name: "X86BootstrapAddressBoundPageTableEntryCandidate",
        minimum: null,
        maximum: null
      }
    ];
    "walk step lost its exact table/index/entry-address/address-bound-entry shape")
| require([
      $descriptor_type.members[]
      | {name, relevance, type_name: .type_reference.name}
    ] == [
      {name: "indices", relevance: "relevant", type_name: "X86BootstrapVirtualAddressIndices"},
      {name: "pml4", relevance: "relevant", type_name: "X86BootstrapPageTableWalkStep"},
      {name: "pdpt", relevance: "relevant", type_name: "X86BootstrapPageTableWalkStep"},
      {name: "pd", relevance: "relevant", type_name: "X86BootstrapPageTableWalkStep"},
      {name: "pt", relevance: "relevant", type_name: "X86BootstrapPageTableWalkStep"}
    ];
    "walk descriptor lost its exact indices and four named levels")
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
        name: "NumericallyConsistent",
        payload: [{
          identity: null,
          name: "descriptor",
          relevance: "relevant",
          type_reference: {kind: "named", name: "X86BootstrapFourLevelWalkDescriptor"}
        }],
        retired_payload_identities: []
      }
    ];
    "walk validation no longer rejects or returns the complete descriptor")
| require(($step_helper.states | map(.name)) == ["entry"] and
          ($validator.states | map(.name)) == [
            "entry",
            "accept_four_level_walk_descriptor",
            "reject_four_level_walk_descriptor"
          ];
    "walk validation changed its exact checked-state interface")
| [
    $validator.states[0]
    | ..
    | objects
    | expression_path
    | select(. != null)
  ] as $validator_paths
| require(any($validator_paths[]; . == ["candidate", "indices", "address"]) and
          any($validator_paths[]; . == ["candidate", "pml4", "entry", "physical_address"]) and
          any($validator_paths[]; . == ["candidate", "pdpt", "table_address"]) and
          any($validator_paths[]; . == ["candidate", "pdpt", "entry", "physical_address"]) and
          any($validator_paths[]; . == ["candidate", "pd", "table_address"]) and
          any($validator_paths[]; . == ["candidate", "pd", "entry", "physical_address"]) and
          any($validator_paths[]; . == ["candidate", "pt", "table_address"]);
    "walk validation lost canonical-address or numeric inter-level linkage inputs")
| [$machine_contracts[] | select(.machine == "x86_bootstrap_page_table_walk_step_is_consistent")][0] as $step_contract
| [$machine_contracts[] | select(.machine == "x86_validate_bootstrap_four_level_walk_descriptor")][0] as $validator_contract
| require(($step_contract | pure_contract) and
          $step_contract.implementation.checked_crash_calls == [] and
          [$step_contract.implementation.inferred_write_frames[] | {
            state,
            completeness,
            paths
          }] == [{state: "entry", completeness: "complete", paths: []}];
    "walk-step validation gained effects, calls, writes, crashes, or nontermination")
| require(($validator_contract | pure_contract) and
          ($validator_contract.implementation.checked_crash_calls | length) == 4 and
          all($validator_contract.implementation.checked_crash_calls[];
              .target_machine == "x86_bootstrap_page_table_walk_step_is_consistent" and
              .surviving_buckets == []) and
          [$validator_contract.implementation.inferred_write_frames[] | {
            state,
            completeness,
            paths
          }] == [
            {state: "entry", completeness: "complete", paths: []},
            {state: "accept_four_level_walk_descriptor", completeness: "complete", paths: []},
            {state: "reject_four_level_walk_descriptor", completeness: "complete", paths: []}
          ];
    "walk descriptor validation gained effects or lost its exact four-step check")
| true
