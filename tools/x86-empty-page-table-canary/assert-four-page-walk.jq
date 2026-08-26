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
  (.contract.service_reach.services // []) == [] and
  (.contract.synchronous_invocation.targets // []) == [] and
  (.contract.suspension.may_suspend // false) == false and
  (.contract.blocking.may_block // false) == false and
  (.contract.crashes.buckets // []) == [] and
  .implementation.checked_service_reach == [] and
  .implementation.checked_synchronous_invocations == [] and
  .implementation.checked_may_suspend == false and
  .implementation.checked_may_block == false and
  .implementation.checked_crash_sites == [] and
  .implementation.checked_termination.kind == "terminates";

def named($path): {kind: "name", path: $path};

def member($receiver; $name):
  {kind: "member", receiver: $receiver, member: $name};

.[0] as $typed
| .[1] as $contracts
| typed_named($typed; "X86BootstrapFourPageWalkImageCandidate"; "members") as $candidate_types
| typed_named($typed; "X86BootstrapFourPageWalkImageCheck"; "members") as $check_types
| typed_named($typed; "x86_bootstrap_page_table_walk_steps_are_equal"; "states") as $step_equalities
| typed_named($typed; "x86_validate_bootstrap_four_page_walk_images"; "states") as $validators
| [
    $contracts.machines[]
    | select(.machine == "x86_bootstrap_page_table_walk_steps_are_equal" or
             .machine == "x86_validate_bootstrap_four_page_walk_images")
  ] as $machine_contracts
| require(($candidate_types | length) == 1 and
          ($check_types | length) == 1 and
          ($step_equalities | length) == 1 and
          ($validators | length) == 1 and
          ($machine_contracts | length) == 2;
    "expected one checked four-page walk-image aggregation frontier")
| $candidate_types[0] as $candidate_type
| $check_types[0] as $check_type
| $step_equalities[0] as $step_equality
| $validators[0] as $validator
| require([
      $candidate_type.members[]
      | {kind, name, relevance, type_name: .type_reference.name}
    ] == [
      {
        kind: "field",
        name: "endpoints",
        relevance: "relevant",
        type_name: "X86BootstrapFourKilobyteWalkEndpoints"
      },
      {
        kind: "field",
        name: "pml4",
        relevance: "relevant",
        type_name: "X86BootstrapSingleEntryPageImageCandidate"
      },
      {
        kind: "field",
        name: "pdpt",
        relevance: "relevant",
        type_name: "X86BootstrapSingleEntryPageImageCandidate"
      },
      {
        kind: "field",
        name: "pd",
        relevance: "relevant",
        type_name: "X86BootstrapSingleEntryPageImageCandidate"
      },
      {
        kind: "field",
        name: "pt",
        relevance: "relevant",
        type_name: "X86BootstrapSingleEntryPageImageCandidate"
      }
    ];
    "four-page carrier must retain exact endpoints and four named page images")
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
        name: "ImagesConsistent",
        payload: [{
          identity: null,
          name: "candidate",
          relevance: "relevant",
          type_reference: {
            kind: "named",
            name: "X86BootstrapFourPageWalkImageCandidate"
          }
        }],
        retired_payload_identities: []
      }
    ];
    "four-page result must reject or return the complete exact carrier")
| require(($step_equality.states | map(.name)) == ["entry"];
    "walk-step equality must remain one pure checked expression")
| [
    $step_equality.states[0].statements[0].value
    | ..
    | objects
    | select(.kind? == "member")
    | expression_path
  ] | unique as $step_paths
| [
    ["left", "table_address"],
    ["left", "index"],
    ["left", "entry_address"],
    ["left", "entry"],
    ["left", "entry", "physical_address"],
    ["left", "entry", "entry"],
    ["right", "table_address"],
    ["right", "index"],
    ["right", "entry_address"],
    ["right", "entry"],
    ["right", "entry", "physical_address"],
    ["right", "entry", "entry"]
  ] as $expected_step_paths
| require(($step_paths - $expected_step_paths) == [] and
          ($expected_step_paths - $step_paths) == [];
    "walk-step equality changed its exact table/index/entry/target coordinates")
| [
    $step_equality.states[0].statements[0].value
    | ..
    | objects
    | select(.kind? == "call")
  ] as $entry_equality_calls
| require($entry_equality_calls == [{
      kind: "call",
      receiver: null,
      target: "x86_page_table_entries_are_equal",
      machine_arguments: [],
      arguments: [
        member(member(named(["left"]); "entry"); "entry"),
        member(member(named(["right"]); "entry"); "entry")
      ],
      acknowledgement_synthesized: false,
      acknowledges_suspend: false,
      acknowledges_block: false
    }];
    "walk-step equality must delegate all fourteen PTE fields to exact equality")
| require(($validator.states | map(.name)) == [
      "entry",
      "validate_bootstrap_pml4_page_image",
      "validate_bootstrap_pdpt_page_image",
      "validate_bootstrap_pd_page_image",
      "validate_bootstrap_pt_page_image",
      "validate_bootstrap_four_page_walk_correspondence",
      "accept_bootstrap_four_page_walk_images",
      "reject_bootstrap_four_page_walk_images"
    ];
    "four-page validator changed its exact checked validation ladder")
| [$validator.states[] | select(.name == "entry")][0] as $entry
| [
    $validator.states[]
    | select(.name | startswith("validate_bootstrap_") and endswith("_page_image"))
  ] as $image_states
| [$validator.states[] | select(.name == "validate_bootstrap_four_page_walk_correspondence")][0] as $correspondence
| [$validator.states[] | select(.name == "accept_bootstrap_four_page_walk_images")][0] as $accept
| [$validator.states[] | select(.name == "reject_bootstrap_four_page_walk_images")][0] as $reject
| require($entry.parameters[0].name == "candidate" and
          $entry.parameters[0].type_reference.name ==
            "X86BootstrapFourPageWalkImageCandidate" and
          $entry.return_type.name == "X86BootstrapFourPageWalkImageCheck" and
          [
            $entry.statements[0:4][]
            | {name, path: (.initial_value | expression_path)}
          ] == [
            {name: "pml4", path: ["candidate", "pml4"]},
            {name: "pdpt", path: ["candidate", "pdpt"]},
            {name: "pd", path: ["candidate", "pd"]},
            {name: "pt", path: ["candidate", "pt"]}
          ];
    "aggregation entry must preserve all four exact image members")
| [
    $entry.statements[]
    | ..
    | objects
    | select(.kind? == "call")
  ] as $endpoint_calls
| require(($endpoint_calls | length) == 1 and
          $endpoint_calls[0].target ==
            "x86_validate_bootstrap_four_kilobyte_walk_endpoints" and
          $endpoint_calls[0].arguments == [
            member(named(["candidate"]); "endpoints")
          ] and
          $entry.statements[6].guard.value.right.path == [
            "X86BootstrapFourKilobyteWalkEndpointCheck",
            "EndpointConsistent"
          ] and
          [$entry.statements[6].target.arguments[] | expression_path] == [
            ["endpoints_check", "candidate"],
            ["pml4"],
            ["pdpt"],
            ["pd"],
            ["pt"]
          ] and
          $entry.statements[7].target.path ==
            ["reject_bootstrap_four_page_walk_images"];
    "endpoint role consistency must be the sole first validation gate")
| require(($image_states | length) == 4 and
          [$image_states[].name] == [
            "validate_bootstrap_pml4_page_image",
            "validate_bootstrap_pdpt_page_image",
            "validate_bootstrap_pd_page_image",
            "validate_bootstrap_pt_page_image"
          ] and
          [
            $image_states[]
            | .statements[0].initial_value
            | {target, argument: .arguments[0].path[0]}
          ] == [
            {target: "x86_validate_bootstrap_single_entry_page_image", argument: "pml4"},
            {target: "x86_validate_bootstrap_single_entry_page_image", argument: "pdpt"},
            {target: "x86_validate_bootstrap_single_entry_page_image", argument: "pd"},
            {target: "x86_validate_bootstrap_single_entry_page_image", argument: "pt"}
          ] and
          all($image_states[];
            .statements[2].guard.value.right.path == [
              "X86BootstrapSingleEntryPageImageCheck",
              "ImageConsistent"
            ] and
            .statements[3].target.path ==
              ["reject_bootstrap_four_page_walk_images"]);
    "each named level must independently pass the existing complete image validator")
| [
    $correspondence.statements[]
    | select(.kind == "local_data")
    | .initial_value
  ] as $correspondence_calls
| require(($correspondence_calls | length) == 4 and
          all($correspondence_calls[];
            .kind == "call" and
            .target == "x86_bootstrap_page_table_walk_steps_are_equal") and
          [
            $correspondence_calls[]
            | [(.arguments[0] | expression_path),
               (.arguments[1] | expression_path)]
          ] == [
            [["pml4", "step"], ["endpoints", "descriptor", "pml4"]],
            [["pdpt", "step"], ["endpoints", "descriptor", "pdpt"]],
            [["pd", "step"], ["endpoints", "descriptor", "pd"]],
            [["pt", "step"], ["endpoints", "descriptor", "pt"]]
          ];
    "all four validated images must bind to their exact descriptor level")
| [
    $correspondence.statements[]
    | select(.kind == "transition")
  ] as $correspondence_transitions
| require(($correspondence_transitions | length) == 2 and
          $correspondence_transitions[0].target.path ==
            ["accept_bootstrap_four_page_walk_images"] and
          [$correspondence_transitions[0].target.arguments[] | .path[0]] ==
            ["endpoints", "pml4", "pdpt", "pd", "pt"] and
          $correspondence_transitions[1].target.path ==
            ["reject_bootstrap_four_page_walk_images"];
    "correspondence must return every exact retained value or reject")
| require($accept.statements[0].value.type_name ==
            "X86BootstrapFourPageWalkImageCheck" and
          $accept.statements[0].value.fields[0].name == "candidate" and
          $accept.statements[0].value.fields[0].value.type_name ==
            "X86BootstrapFourPageWalkImageCandidate" and
          [
            $accept.statements[0].value.fields[0].value.fields[]
            | {name, path: .value.path}
          ] == [
            {name: "endpoints", path: ["endpoints"]},
            {name: "pml4", path: ["pml4"]},
            {name: "pdpt", path: ["pdpt"]},
            {name: "pd", path: ["pd"]},
            {name: "pt", path: ["pt"]}
          ] and
          $reject.statements == [{
            kind: "expression",
            value: {
              kind: "name",
              path: ["X86BootstrapFourPageWalkImageCheck", "Rejected"]
            }
          }];
    "success must return exact endpoints/descriptor/images with no partial result")
| [$machine_contracts[] | select(.machine == "x86_bootstrap_page_table_walk_steps_are_equal")][0] as $step_contract
| [$machine_contracts[] | select(.machine == "x86_validate_bootstrap_four_page_walk_images")][0] as $validator_contract
| require(($step_contract | pure_contract) and
          ($validator_contract | pure_contract) and
          all($step_contract.implementation.checked_crash_calls[];
            .surviving_buckets == []) and
          all($validator_contract.implementation.checked_crash_calls[];
            .surviving_buckets == []) and
          all($step_contract.implementation.inferred_write_frames[];
            .completeness == "complete" and .paths == []) and
          all($validator_contract.implementation.inferred_write_frames[];
            .completeness == "complete") and
          ($validator_contract.implementation.checked_crash_calls | length) == 9 and
          ([
            $validator_contract.implementation.checked_crash_calls[]
            | .target_machine
          ] | group_by(.) | map({key: .[0], value: length}) | from_entries) == {
            x86_bootstrap_page_table_walk_steps_are_equal: 4,
            x86_validate_bootstrap_four_kilobyte_walk_endpoints: 1,
            x86_validate_bootstrap_single_entry_page_image: 4
          };
    "four-page aggregation gained authority, effects, crash survivors, or nontermination")
| true
