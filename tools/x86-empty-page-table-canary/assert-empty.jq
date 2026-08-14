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
| typed_named($typed; "X86PageTablePageCandidate"; "members") as $page_types
| typed_named($typed; "X86EmptyPageTablePagePolicyCheck"; "members") as $check_types
| typed_named($typed; "x86_page_table_entry_is_zero"; "states") as $zero_helpers
| typed_named($typed; "x86_validate_empty_page_table_page"; "states") as $validators
| typed_named($typed; "x86_scan_empty_page_table_page"; "states") as $scanners
| [
    $contracts.machines[]
    | select(.machine as $name | [
        "x86_page_table_entry_is_zero",
        "x86_validate_empty_page_table_page",
        "x86_scan_empty_page_table_page"
      ] | index($name))
  ] as $machine_contracts
| require(($page_types | length) == 1 and
          ($check_types | length) == 1 and
          ($zero_helpers | length) == 1 and
          ($validators | length) == 1 and
          ($scanners | length) == 1 and
          ($machine_contracts | length) == 3;
    "expected one empty-page candidate, check, helper, wrapper, scanner, and three contracts")
| $page_types[0] as $page_type
| $check_types[0] as $check_type
| $zero_helpers[0] as $zero_helper
| $validators[0] as $validator
| $scanners[0] as $scanner
| require($page_type.members == [{
      kind: "field",
      identity: null,
      name: "entries",
      relevance: "relevant",
      type_reference: {
        kind: "fixed_array",
        element_type: {kind: "named", name: "X86PageTableEntry"},
        length: "512"
      }
    }];
    "page-table page is no longer exactly 512 relevant PTE values")
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
        name: "ZeroConsistent",
        payload: [{
          identity: null,
          name: "candidate",
          relevance: "relevant",
          type_reference: {kind: "named", name: "X86PageTablePageCandidate"}
        }],
        retired_payload_identities: []
      }
    ];
    "empty-page validation no longer returns the complete checked candidate")
| require(($zero_helper.states | length) == 1 and
          $zero_helper.states[0].name == "entry" and
          $zero_helper.states[0].return_type == {kind: "named", name: "bool"} and
          ($zero_helper.states[0].statements | length) == 1 and
          $zero_helper.states[0].statements[0].kind == "expression";
    "zero-entry helper is no longer one pure Boolean state")
| $zero_helper.states[0].statements[0].value as $zero_expression
| [
    $zero_expression
    | ..
    | objects
    | select(.kind? == "unary" and .operator? == "!")
    | (.operand | expression_path)
  ] as $false_fields
| [
    $zero_expression
    | ..
    | objects
    | select(.kind? == "binary" and .operator? == "==" and .right.text? == "0")
    | (.left | expression_path)
  ] as $zero_fields
| require($false_fields == [
      ["entry", "present"],
      ["entry", "writable"],
      ["entry", "user"],
      ["entry", "write_through"],
      ["entry", "cache_disable"],
      ["entry", "accessed"],
      ["entry", "dirty"],
      ["entry", "page_size_or_pat"],
      ["entry", "global"],
      ["entry", "no_execute"]
    ] and
    $zero_fields == [
      ["entry", "software_low"],
      ["entry", "frame_number"],
      ["entry", "software_high"],
      ["entry", "protection_key"]
    ];
    "zero-entry helper no longer rejects every nonzero PTE field")
| require(($validator.states | length) == 1 and
          $validator.states[0].name == "entry" and
          $validator.states[0].return_type == {
            kind: "named",
            name: "X86EmptyPageTablePagePolicyCheck"
          } and
          $validator.states[0].statements[0].initial_value.target ==
            "x86_scan_empty_page_table_page" and
          $validator.states[0].statements[0].initial_value.arguments == [
            {kind: "name", path: ["candidate"]},
            {kind: "integer", text: "512"},
            {kind: "boolean", value: true}
          ];
    "empty-page wrapper no longer starts one complete 512-slot scan")
| require(($scanner.states | map(.name)) == [
      "entry",
      "settle_empty_page_table_page",
      "accept_empty_page_table_page",
      "reject_empty_page_table_page"
    ] and
    $scanner.termination == {interface: "internal_derived"} and
    $scanner.states[0].parameters[1].type_reference == {
      kind: "constrained",
      base_type: {kind: "named", name: "u64"},
      constraints: [{
        kind: "range",
        minimum: {kind: "integer", text: "1"},
        maximum: {kind: "integer", text: "512"}
      }]
    };
    "empty-page scanner lost its exact four-state 512-step decreasing shape")
| $scanner.states[0].statements as $scan_statements
| require($scan_statements[0].initial_value == {
      kind: "binary",
      left: {kind: "integer", text: "512"},
      operator: "-",
      right: {kind: "name", path: ["remaining"]}
    } and
    $scan_statements[1].initial_value == {
      kind: "indexed",
      collection: {
        kind: "member",
        receiver: {kind: "name", path: ["candidate"]},
        member: "entries"
      },
      index: {kind: "name", path: ["index"]}
    } and
    $scan_statements[2].initial_value.kind == "binary" and
    $scan_statements[2].initial_value.operator == "&&" and
    $scan_statements[2].initial_value.left.path == ["zero_so_far"] and
    $scan_statements[2].initial_value.right.target == "x86_page_table_entry_is_zero" and
    $scan_statements[3].target.path == ["x86_scan_empty_page_table_page"] and
    $scan_statements[3].target.arguments[1] == {
      kind: "binary",
      left: {kind: "name", path: ["remaining"]},
      operator: "-",
      right: {kind: "integer", text: "1"}
    } and
    $scan_statements[4].target.path == ["settle_empty_page_table_page"];
    "empty-page scanner no longer checks each derived index and carries its verdict")
| require($scanner.states[2].statements[0].value.type_name ==
            "X86EmptyPageTablePagePolicyCheck" and
          $scanner.states[2].statements[0].value.fields[0].value.path == ["candidate"] and
          $scanner.states[3].statements[0].value.path ==
            ["X86EmptyPageTablePagePolicyCheck", "Rejected"];
    "empty-page scanner no longer returns only the whole candidate or rejection")
| [$machine_contracts[] | select(.machine == "x86_page_table_entry_is_zero")][0] as $helper_contract
| [$machine_contracts[] | select(.machine == "x86_validate_empty_page_table_page")][0] as $validator_contract
| [$machine_contracts[] | select(.machine == "x86_scan_empty_page_table_page")][0] as $scanner_contract
| require(($helper_contract | pure_contract) and
          $helper_contract.implementation.checked_crash_calls == [] and
          [$helper_contract.implementation.inferred_write_frames[] | {
            state,
            completeness,
            paths
          }] == [{state: "entry", completeness: "complete", paths: []}];
    "zero-entry helper gained effects, calls, writes, crashes, or nontermination")
| require(($validator_contract | pure_contract) and
          ($validator_contract.implementation.checked_crash_calls | length) == 1 and
          $validator_contract.implementation.checked_crash_calls[0].target_machine ==
            "x86_scan_empty_page_table_page" and
          $validator_contract.implementation.checked_crash_calls[0].surviving_buckets == [] and
          [$validator_contract.implementation.inferred_write_frames[] | {
            state,
            completeness,
            paths
          }] == [{
            state: "entry",
            completeness: "complete",
            paths: ["$P0", "self"]
          }];
    "empty-page wrapper gained effects or changed its exact by-value transfer frame")
| require(($scanner_contract | pure_contract) and
          ($scanner_contract.implementation.checked_crash_calls | length) == 1 and
          $scanner_contract.implementation.checked_crash_calls[0].target_machine ==
            "x86_page_table_entry_is_zero" and
          $scanner_contract.implementation.checked_crash_calls[0].surviving_buckets == [] and
          $scanner_contract.implementation.resolved_ranking_view == "Nat::Descending" and
          $scanner_contract.implementation.ranking_witness == {
            subjects: ["remaining"],
            view: "Nat::Descending",
            view_arguments: []
          } and
          [$scanner_contract.implementation.inferred_write_frames[] | {
            state,
            completeness,
            paths
          }] == [
            {state: "entry", completeness: "opaque", paths: []},
            {state: "settle_empty_page_table_page", completeness: "complete", paths: []},
            {state: "accept_empty_page_table_page", completeness: "complete", paths: []},
            {state: "reject_empty_page_table_page", completeness: "complete", paths: []}
          ];
    "empty-page scanner lost its ranking evidence or gained effects, writes, or crashes")
| true
