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

def constrained_u64_field:
  .type_reference.kind == "constrained" and
  .type_reference.base_type == {kind: "named", name: "u64"} and
  .type_reference.constraints[0].kind == "range" and
  .type_reference.constraints[0].minimum == {kind: "integer", text: "0"};

.[0] as $typed
| .[1] as $contracts
| typed_named($typed; "X86BootstrapAddressSpaceProfile"; "members") as $profile_types
| typed_named($typed; "x86_bootstrap_address_space_profile"; "states") as $profile_machines
| typed_named($typed; "X86BootstrapVirtualAddressIndices"; "members") as $address_index_types
| typed_named($typed; "X86BootstrapVirtualAddressCheck"; "members") as $address_check_types
| typed_named($typed; "x86_decompose_bootstrap_virtual_address"; "states") as $address_machines
| typed_named($typed; "X86BootstrapPhysicalFrameGeometry"; "members") as $frame_geometry_types
| typed_named($typed; "X86BootstrapPhysicalFrameGeometryCheck"; "members") as $frame_check_types
| typed_named($typed; "x86_validate_bootstrap_physical_frame_geometry"; "states") as $frame_machines
| typed_named($typed; "X86PageTablePageCandidate"; "members") as $page_types
| typed_named($typed; "X86EmptyPageTablePagePolicyCheck"; "members") as $check_types
| typed_named($typed; "x86_page_table_entry_is_zero"; "states") as $zero_helpers
| typed_named($typed; "x86_validate_empty_page_table_page"; "states") as $validators
| typed_named($typed; "x86_scan_empty_page_table_page"; "states") as $scanners
| [
    $contracts.machines[]
    | select(.machine as $name | [
        "x86_page_table_entry_is_zero",
        "x86_bootstrap_address_space_profile",
        "x86_decompose_bootstrap_virtual_address",
        "x86_validate_bootstrap_physical_frame_geometry",
        "x86_validate_empty_page_table_page",
        "x86_scan_empty_page_table_page"
      ] | index($name))
  ] as $machine_contracts
| require(($profile_types | length) == 1 and
          ($profile_machines | length) == 1 and
          ($address_index_types | length) == 1 and
          ($address_check_types | length) == 1 and
          ($address_machines | length) == 1 and
          ($frame_geometry_types | length) == 1 and
          ($frame_check_types | length) == 1 and
          ($frame_machines | length) == 1 and
          ($page_types | length) == 1 and
          ($check_types | length) == 1 and
          ($zero_helpers | length) == 1 and
          ($validators | length) == 1 and
          ($scanners | length) == 1 and
          ($machine_contracts | length) == 6;
    "expected one bootstrap profile, address/frame geometry, and empty-page frontier")
| $profile_types[0] as $profile_type
| $profile_machines[0] as $profile_machine
| $address_index_types[0] as $address_index_type
| $address_check_types[0] as $address_check_type
| $address_machines[0] as $address_machine
| $frame_geometry_types[0] as $frame_geometry_type
| $frame_check_types[0] as $frame_check_type
| $frame_machines[0] as $frame_machine
| $page_types[0] as $page_type
| $check_types[0] as $check_type
| $zero_helpers[0] as $zero_helper
| $validators[0] as $validator
| $scanners[0] as $scanner
| require($profile_type.members == [
      {
        kind: "field",
        identity: null,
        name: "canonical_virtual_address_bits",
        relevance: "relevant",
        type_reference: {kind: "named", name: "u8"}
      },
      {
        kind: "field",
        identity: null,
        name: "translation_level_count",
        relevance: "relevant",
        type_reference: {kind: "named", name: "u8"}
      },
      {
        kind: "field",
        identity: null,
        name: "index_bits_per_level",
        relevance: "relevant",
        type_reference: {kind: "named", name: "u8"}
      },
      {
        kind: "field",
        identity: null,
        name: "entries_per_table",
        relevance: "relevant",
        type_reference: {kind: "named", name: "u64"}
      },
      {
        kind: "field",
        identity: null,
        name: "page_shift",
        relevance: "relevant",
        type_reference: {kind: "named", name: "u8"}
      },
      {
        kind: "field",
        identity: null,
        name: "page_bytes",
        relevance: "relevant",
        type_reference: {kind: "named", name: "u64"}
      },
      {
        kind: "field",
        identity: null,
        name: "level_shifts",
        relevance: "relevant",
        type_reference: {
          kind: "fixed_array",
          element_type: {kind: "named", name: "u8"},
          length: "4"
        }
      },
      {
        kind: "field",
        identity: null,
        name: "la57_enabled",
        relevance: "relevant",
        type_reference: {kind: "named", name: "bool"}
      }
    ];
    "bootstrap address-space profile changed shape")
| require($profile_machine.states == [{
      name: "entry",
      parameters: [],
      return_type: {kind: "named", name: "X86BootstrapAddressSpaceProfile"},
      contracts: [],
      statements: [{
        kind: "expression",
        value: {
          kind: "struct_literal",
          type_name: "X86BootstrapAddressSpaceProfile",
          fields: [
            {name: "canonical_virtual_address_bits", value: {kind: "integer", text: "48"}},
            {name: "translation_level_count", value: {kind: "integer", text: "4"}},
            {name: "index_bits_per_level", value: {kind: "integer", text: "9"}},
            {name: "entries_per_table", value: {kind: "integer", text: "512"}},
            {name: "page_shift", value: {kind: "integer", text: "12"}},
            {name: "page_bytes", value: {kind: "integer", text: "4096"}},
            {
              name: "level_shifts",
              value: {
                kind: "array_literal",
                values: [
                  {kind: "integer", text: "39"},
                  {kind: "integer", text: "30"},
                  {kind: "integer", text: "21"},
                  {kind: "integer", text: "12"}
                ]
              }
            },
            {name: "la57_enabled", value: {kind: "boolean", value: false}}
          ]
        }
      }]
    }];
    "bootstrap address-space policy is no longer exact four-level 48-bit x86-64")
| require($address_index_type.members[0] == {
      kind: "field",
      identity: null,
      name: "address",
      relevance: "relevant",
      type_reference: {kind: "named", name: "u64"}
    } and
    all($address_index_type.members[1:][]; constrained_u64_field) and
    [
      $address_index_type.members[1:][]
      | {
          name,
          relevance,
          maximum: .type_reference.constraints[0].maximum.text
        }
    ] == [
      {name: "pml4_index", relevance: "relevant", maximum: "511"},
      {name: "pdpt_index", relevance: "relevant", maximum: "511"},
      {name: "pd_index", relevance: "relevant", maximum: "511"},
      {name: "pt_index", relevance: "relevant", maximum: "511"},
      {name: "page_offset", relevance: "relevant", maximum: "4095"}
    ];
    "bootstrap virtual-address decomposition lost its exact bounded fields")
| require($address_check_type.members == [
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
        name: "Canonical",
        payload: [{
          identity: null,
          name: "indices",
          relevance: "relevant",
          type_reference: {kind: "named", name: "X86BootstrapVirtualAddressIndices"}
        }],
        retired_payload_identities: []
      }
    ];
    "bootstrap virtual-address check no longer returns rejection or exact indices")
| require(($address_machine.states | map(.name)) == ["entry", "decompose", "reject"] and
          [$address_machine.states[].return_type] == [
            {kind: "named", name: "X86BootstrapVirtualAddressCheck"},
            {kind: "named", name: "X86BootstrapVirtualAddressCheck"},
            {kind: "named", name: "X86BootstrapVirtualAddressCheck"}
          ] and
          $address_machine.states[0].parameters[0].name == "address" and
          $address_machine.states[0].parameters[0].type_reference == {
            kind: "named",
            name: "u64"
          };
    "bootstrap virtual-address check changed its exact three-state interface")
| $address_machine.states[0].statements as $address_entry
| require($address_entry[0] == {
      kind: "local_data",
      name: "low_canonical_max",
      type_reference: {kind: "named", name: "u64"},
      initial_value: {kind: "integer", text: "0x7fffffffffff"}
    } and
    $address_entry[1] == {
      kind: "local_data",
      name: "high_canonical_min",
      type_reference: {kind: "named", name: "u64"},
      initial_value: {kind: "integer", text: "0xffff800000000000"}
    } and
    $address_entry[2].target.path == ["decompose"] and
    $address_entry[2].target.arguments == [{kind: "name", path: ["address"]}] and
    $address_entry[2].guard.value.left.left == {
      kind: "binary",
      left: {kind: "name", path: ["address"]},
      operator: "<=",
      right: {kind: "name", path: ["low_canonical_max"]}
    } and
    $address_entry[2].guard.value.left.operator == "||" and
    $address_entry[2].guard.value.left.right == {
      kind: "binary",
      left: {kind: "name", path: ["address"]},
      operator: ">=",
      right: {kind: "name", path: ["high_canonical_min"]}
    } and
    $address_entry[3].target.path == ["reject"] and
    $address_entry[3].guard == {kind: "always"};
    "bootstrap virtual-address check no longer rejects the noncanonical 48-bit hole")
| $address_machine.states[1].statements as $decomposition
| require([
      $decomposition[0:4][]
      | {
          name,
          maximum: .type_reference.constraints[0].maximum.text,
          shift: .initial_value.left.right.text,
          mask: .initial_value.right.text
        }
    ] == [
      {name: "pml4_index", maximum: "511", shift: "39", mask: "511"},
      {name: "pdpt_index", maximum: "511", shift: "30", mask: "511"},
      {name: "pd_index", maximum: "511", shift: "21", mask: "511"},
      {name: "pt_index", maximum: "511", shift: "12", mask: "511"}
    ] and
    $decomposition[4].name == "page_offset" and
    $decomposition[4].type_reference.constraints[0].maximum.text == "4095" and
    $decomposition[4].initial_value == {
      kind: "binary",
      left: {kind: "name", path: ["address"]},
      operator: "&",
      right: {kind: "integer", text: "4095"}
    };
    "bootstrap virtual-address indexes no longer use exact four-level shifts and masks")
| $decomposition[5].value.fields[0].value as $canonical_indices
| require($decomposition[5].kind == "expression" and
          $decomposition[5].value.type_name == "X86BootstrapVirtualAddressCheck" and
          $canonical_indices.type_name == "X86BootstrapVirtualAddressIndices" and
          [$canonical_indices.fields[] | {name, path: .value.path}] == [
            {name: "address", path: ["address"]},
            {name: "pml4_index", path: ["pml4_index"]},
            {name: "pdpt_index", path: ["pdpt_index"]},
            {name: "pd_index", path: ["pd_index"]},
            {name: "pt_index", path: ["pt_index"]},
            {name: "page_offset", path: ["page_offset"]}
          ] and
          $address_machine.states[2].statements == [{
            kind: "expression",
            value: {kind: "name", path: ["X86BootstrapVirtualAddressCheck", "Rejected"]}
          }];
    "bootstrap virtual-address check no longer retains the exact address or rejects cleanly")
| require($frame_geometry_type.members[0] == {
      kind: "field",
      identity: null,
      name: "address",
      relevance: "relevant",
      type_reference: {kind: "named", name: "u64"}
    } and
    ($frame_geometry_type.members[1] | constrained_u64_field) and
    $frame_geometry_type.members[1].name == "frame_number" and
    $frame_geometry_type.members[1].relevance == "relevant" and
    $frame_geometry_type.members[1].type_reference.constraints[0].maximum == {
      kind: "integer",
      text: "1099511627775"
    };
    "bootstrap physical-frame geometry lost its retained address or bounded 40-bit PFN")
| require($frame_check_type.members == [
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
        name: "GeometryConsistent",
        payload: [{
          identity: null,
          name: "frame",
          relevance: "relevant",
          type_reference: {kind: "named", name: "X86BootstrapPhysicalFrameGeometry"}
        }],
        retired_payload_identities: []
      }
    ];
    "bootstrap physical-frame check no longer returns rejection or exact numeric geometry")
| require(($frame_machine.states | map(.name)) == [
      "entry",
      "derive_frame_number",
      "reject_frame_geometry"
    ] and
    all($frame_machine.states[];
      .return_type == {kind: "named", name: "X86BootstrapPhysicalFrameGeometryCheck"});
    "bootstrap physical-frame check changed its exact three-state interface")
| $frame_machine.states[0].statements as $frame_entry
| require(($frame_entry[0] | constrained_u64_field) and
          $frame_entry[0].name == "page_offset" and
          $frame_entry[0].type_reference.constraints[0].maximum.text == "4095" and
          $frame_entry[0].initial_value == {
            kind: "binary",
            left: {kind: "name", path: ["address"]},
            operator: "&",
            right: {kind: "integer", text: "4095"}
          } and
          $frame_entry[1].target.path == ["derive_frame_number"] and
          $frame_entry[1].target.arguments == [{kind: "name", path: ["address"]}] and
          $frame_entry[1].guard.value.left == {
            kind: "binary",
            left: {
              kind: "binary",
              left: {kind: "name", path: ["page_offset"]},
              operator: "==",
              right: {kind: "integer", text: "0"}
            },
            operator: "&&",
            right: {
              kind: "binary",
              left: {kind: "name", path: ["address"]},
              operator: "<=",
              right: {kind: "integer", text: "0xffffffffff000"}
            }
          } and
          $frame_entry[2].target.path == ["reject_frame_geometry"] and
          $frame_entry[2].guard == {kind: "always"};
    "bootstrap physical-frame check no longer enforces 4-KiB alignment and the 52-bit envelope")
| $frame_machine.states[1].statements as $frame_derivation
| require(($frame_derivation[0] | constrained_u64_field) and
          $frame_derivation[0].name == "frame_number" and
          $frame_derivation[0].type_reference.constraints[0].maximum.text ==
            "1099511627775" and
          $frame_derivation[0].initial_value == {
            kind: "binary",
            left: {
              kind: "binary",
              left: {kind: "name", path: ["address"]},
              operator: ">>",
              right: {kind: "integer", text: "12"}
            },
            operator: "&",
            right: {kind: "integer", text: "1099511627775"}
          } and
          $frame_derivation[1].value.type_name ==
            "X86BootstrapPhysicalFrameGeometryCheck" and
          $frame_derivation[1].value.fields[0].name == "frame" and
          $frame_derivation[1].value.fields[0].value.type_name ==
            "X86BootstrapPhysicalFrameGeometry" and
          [$frame_derivation[1].value.fields[0].value.fields[] |
            {name, path: .value.path}] == [
              {name: "address", path: ["address"]},
              {name: "frame_number", path: ["frame_number"]}
            ] and
          $frame_machine.states[2].statements == [{
            kind: "expression",
            value: {
              kind: "name",
              path: ["X86BootstrapPhysicalFrameGeometryCheck", "Rejected"]
            }
          }];
    "bootstrap physical-frame check no longer derives and retains the exact bounded PFN")
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
| [$machine_contracts[] | select(.machine == "x86_bootstrap_address_space_profile")][0] as $profile_contract
| [$machine_contracts[] | select(.machine == "x86_decompose_bootstrap_virtual_address")][0] as $address_contract
| [$machine_contracts[] | select(.machine == "x86_validate_bootstrap_physical_frame_geometry")][0] as $frame_contract
| [$machine_contracts[] | select(.machine == "x86_validate_empty_page_table_page")][0] as $validator_contract
| [$machine_contracts[] | select(.machine == "x86_scan_empty_page_table_page")][0] as $scanner_contract
| require(($profile_contract | pure_contract) and
          $profile_contract.implementation.checked_crash_calls == [] and
          [$profile_contract.implementation.inferred_write_frames[] | {
            state,
            completeness,
            paths
          }] == [{state: "entry", completeness: "complete", paths: []}];
    "bootstrap address-space profile gained effects, calls, writes, crashes, or nontermination")
| require(($address_contract | pure_contract) and
          $address_contract.implementation.checked_crash_calls == [] and
          [$address_contract.implementation.inferred_write_frames[] | {
            state,
            completeness,
            paths
          }] == [
            {state: "entry", completeness: "complete", paths: []},
            {state: "decompose", completeness: "complete", paths: []},
            {state: "reject", completeness: "complete", paths: []}
          ];
    "bootstrap virtual-address decomposition gained effects, calls, writes, crashes, or nontermination")
| require(($frame_contract | pure_contract) and
          $frame_contract.implementation.checked_crash_calls == [] and
          [$frame_contract.implementation.inferred_write_frames[] | {
            state,
            completeness,
            paths
          }] == [
            {state: "entry", completeness: "complete", paths: []},
            {state: "derive_frame_number", completeness: "complete", paths: []},
            {state: "reject_frame_geometry", completeness: "complete", paths: []}
          ];
    "bootstrap physical-frame geometry gained effects, calls, writes, crashes, or nontermination")
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
