def require($condition; $message):
  if $condition then . else error($message) end;

def state($machine; $name):
  first($machine.states[] | select(.name == $name));

def field_value($literal; $name):
  first($literal.fields[] | select(.name == $name) | .value);

def expression_signature:
  if .kind == "name" then
    {kind: "path", path}
  elif .kind == "integer" then
    {kind: "integer", text}
  elif .kind == "binary" then
    {
      kind: "binary",
      left: (.left | expression_signature),
      operator,
      right: (.right | expression_signature)
    }
  elif .kind == "member" and .receiver.kind == "struct_literal" then
    .member as $member
    | {
        kind: "constant_member",
        type_name: .receiver.type_name,
        member: $member,
        value: (.receiver | field_value(.; $member) | .text)
      }
  elif .kind == "member" then
    {
      kind: "member",
      receiver: (.receiver | expression_signature),
      member
    }
  else
    {kind}
  end;

def conjunct_signatures:
  if .kind == "binary" and .operator == "&&" then
    [(.left | conjunct_signatures)[], (.right | conjunct_signatures)[]]
  else
    [{
      left: (.left | expression_signature),
      operator,
      right: (.right | expression_signature)
    }]
  end;

def transition_targets($state):
  [
    $state.statements[]
    | select(.kind == "transition")
    | {
        target: .target.path[0],
        arguments: [
          .target.arguments[]
          | if .kind == "name" then { kind, path }
            elif .kind == "integer" then { kind, text }
            else { kind }
            end
        ],
        guard: .guard.kind
      }
  ];

def has_granted_extent_parameter:
  any(.parameters[]?;
    any(.type_reference.constraints[]?;
      .kind == "domain" and .name == "Granted"
    )
  );

.[0] as $typed
| .[1] as $contracts
| .[2] as $qualification
| [
    $typed
    | ..
    | objects
    | select(.name? == "Main::own_machine")
  ] as $machines
| [
    $contracts
    | ..
    | objects
    | select(.machine? == "Main::own_machine")
  ] as $machine_contracts
| require(($machines | length) == 1;
    "expected exactly one typed Main::own_machine machine")
| require(($machine_contracts | length) == 1;
    "expected exactly one Main::own_machine contract")
| first(
    $typed
    | ..
    | objects
    | select(.name? == "Main" and has("members"))
  ) as $main_type
| first(
    $typed
    | ..
    | objects
    | select(.name? == "UefiMemoryMapStorage" and has("members"))
  ) as $map_storage_type
| $machines[0] as $machine
| $machine_contracts[0] as $machine_contract
| require(first($main_type.members[] | select(.name == "map_buf") | .type_reference) == {
      kind: "named",
      name: "UefiMemoryMapStorage"
    } and
    $map_storage_type.members == [
      {
        kind: "field",
        identity: null,
        name: "alignment_anchor",
        relevance: "relevant",
        type_reference: {kind: "named", name: "u64"}
      },
      {
        kind: "field",
        identity: null,
        name: "bytes",
        relevance: "relevant",
        type_reference: {
          kind: "fixed_array",
          element_type: {kind: "named", name: "u8"},
          length: "65536"
        }
      }
    ];
    "owned-machine map backing lost its 8-byte anchor or exact 64-KiB bytes")
| state($machine; "own_machine") as $entry
| state($machine; "validate_boot_services") as $validate_boot_services
| state($machine; "refresh_map") as $refresh_map
| state($machine; "get_map") as $get_map
| state($machine; "map_failed") as $map_failed
| state($machine; "walk") as $walk
| state($machine; "step") as $step
| state($machine; "validate_candidate") as $validate_candidate
| state($machine; "validate_candidate_end") as $validate_candidate_end
| state($machine; "prepare_audit") as $prepare_audit
| state($machine; "audit_descriptor") as $audit_descriptor
| state($machine; "audit_geometry") as $audit_geometry
| state($machine; "audit_geometry_end") as $audit_geometry_end
| state($machine; "audit_overlap") as $audit_overlap
| state($machine; "audit_classify") as $audit_classify
| state($machine; "audit_left") as $audit_left
| state($machine; "audit_right") as $audit_right
| state($machine; "audit_step") as $audit_step
| state($machine; "exit") as $exit
| state($machine; "exit_failed") as $exit_failed
| state($machine; "own") as $own
| state($machine; "serial_init") as $serial_init
| state($machine; "to_mib") as $to_mib
| state($machine; "owned_wait") as $owned_wait
| state($machine; "owned_wait_busy") as $owned_wait_busy
| state($machine; "owned_send") as $owned_send
| state($machine; "digits_wait") as $digits_wait
| state($machine; "digits_wait_busy") as $digits_wait_busy
| state($machine; "report_size") as $report_size
| state($machine; "oversized_mib") as $oversized_mib
| state($machine; "tail_send") as $tail_send
| state($machine; "idle") as $idle
| state($machine; "owned_idle") as $owned_idle
| require(
    (($entry.statements[0].guard.value.left | conjunct_signatures) == [
      {
        left: {
          kind: "member",
          receiver: {
            kind: "member",
            receiver: {kind: "path", path: ["table"]},
            member: "header"
          },
          member: "signature"
        },
        operator: "==",
        right: {kind: "integer", text: "0x5453595320494249"}
      },
      {
        left: {
          kind: "member",
          receiver: {
            kind: "member",
            receiver: {kind: "path", path: ["table"]},
            member: "header"
          },
          member: "revision"
        },
        operator: ">=",
        right: {kind: "integer", text: "0x20000"}
      },
      {
        left: {
          kind: "member",
          receiver: {
            kind: "member",
            receiver: {kind: "path", path: ["table"]},
            member: "header"
          },
          member: "header_size"
        },
        operator: ">=",
        right: {kind: "integer", text: "120"}
      },
      {
        left: {
          kind: "member",
          receiver: {
            kind: "member",
            receiver: {kind: "path", path: ["table"]},
            member: "header"
          },
          member: "reserved"
        },
        operator: "==",
        right: {kind: "integer", text: "0"}
      }
    ]) and
    (transition_targets($entry) == [
      {
        target: "validate_boot_services",
        arguments: [
          { kind: "name", path: ["handle"] },
          { kind: "name", path: ["table"] }
        ],
        guard: "when"
      },
      {
        target: "idle",
        arguments: [],
        guard: "always"
      }
    ]);
    "owned-machine entry no longer gates the firmware system-table prefix")
| require(
    ($validate_boot_services.statements[0].initial_value == {
      kind: "member",
      receiver: {kind: "name", path: ["table"]},
      member: "boot_services"
    }) and
    (($validate_boot_services.statements[1].guard.value.left | conjunct_signatures) == [
      {
        left: {
          kind: "member",
          receiver: {
            kind: "member",
            receiver: {kind: "path", path: ["bs"]},
            member: "hdr"
          },
          member: "signature"
        },
        operator: "==",
        right: {kind: "integer", text: "0x56524553544f4f42"}
      },
      {
        left: {
          kind: "member",
          receiver: {
            kind: "member",
            receiver: {kind: "path", path: ["bs"]},
            member: "hdr"
          },
          member: "revision"
        },
        operator: "==",
        right: {
          kind: "member",
          receiver: {
            kind: "member",
            receiver: {kind: "path", path: ["table"]},
            member: "header"
          },
          member: "revision"
        }
      },
      {
        left: {
          kind: "member",
          receiver: {
            kind: "member",
            receiver: {kind: "path", path: ["bs"]},
            member: "hdr"
          },
          member: "header_size"
        },
        operator: ">=",
        right: {kind: "integer", text: "240"}
      },
      {
        left: {
          kind: "member",
          receiver: {
            kind: "member",
            receiver: {kind: "path", path: ["bs"]},
            member: "hdr"
          },
          member: "reserved"
        },
        operator: "==",
        right: {kind: "integer", text: "0"}
      }
    ]) and
    (transition_targets($validate_boot_services) == [
      {
        target: "refresh_map",
        arguments: [
          {kind: "name", path: ["bs"]},
          {kind: "name", path: ["handle"]},
          {kind: "integer", text: "0"}
        ],
        guard: "when"
      },
      {
        target: "idle",
        arguments: [],
        guard: "always"
      }
    ]);
    "Boot Services projection no longer gates its required table prefix before dispatch")
| require(transition_targets($refresh_map) == [
      {
        target: "get_map",
        arguments: [
          { kind: "name", path: ["bs"] },
          { kind: "name", path: ["handle"] },
          { kind: "name", path: ["stale_retry_used"] },
          { kind: "integer", text: "16384" }
        ],
        guard: "always"
      }
    ];
    "fresh map/key transaction no longer starts at the 16-KiB capacity")
| first(
    $get_map.statements[]
    | select(.kind == "local_data" and .name == "status")
  ) as $map_call
| require(($map_call.initial_value.kind == "call") and
          ($map_call.initial_value.target == "get_memory_map") and
          ($get_map.parameters[2].name == "stale_retry_used") and
          ($get_map.parameters[2].type_reference.constraints[0] == {
            kind: "range",
            minimum: {kind: "integer", text: "0"},
            maximum: {kind: "integer", text: "1"}
          }) and
          ($get_map.parameters[3].name == "attempted_capacity") and
          ($get_map.parameters[3].type_reference.constraints[0] == {
            kind: "range",
            minimum: {kind: "integer", text: "16384"},
            maximum: {kind: "integer", text: "65536"}
          }) and
          ($get_map.statements[0].name == "map_size") and
          ($get_map.statements[0].initial_value == {
            kind: "name",
            path: ["attempted_capacity"]
          }) and
          ($map_call.initial_value.arguments[2] == {
            kind: "mutable",
            value: {
              kind: "member",
              receiver: {
                kind: "member",
                receiver: {kind: "name", path: ["self"]},
                member: "map_buf"
              },
              member: "bytes"
            }
          }) and
          ($get_map.statements[6].guard.value.left | conjunct_signatures) == [
            {
              left: {kind: "path", path: ["status_code"]},
              operator: "==",
              right: {kind: "integer", text: "0"}
            },
            {
              left: {kind: "path", path: ["desc_version"]},
              operator: "==",
              right: {kind: "integer", text: "1"}
            },
            {
              left: {kind: "path", path: ["desc_size"]},
              operator: ">=",
              right: {kind: "integer", text: "40"}
            },
            {
              left: {kind: "path", path: ["desc_size"]},
              operator: "<=",
              right: {kind: "integer", text: "65536"}
            },
            {
              left: {
                kind: "binary",
                left: {kind: "path", path: ["desc_size"]},
                operator: "%",
                right: {kind: "integer", text: "8"}
              },
              operator: "==",
              right: {kind: "integer", text: "0"}
            },
            {
              left: {kind: "path", path: ["map_size"]},
              operator: ">=",
              right: {kind: "path", path: ["desc_size"]}
            },
            {
              left: {kind: "path", path: ["map_size"]},
              operator: "<=",
              right: {kind: "path", path: ["attempted_capacity"]}
            },
            {
              left: {kind: "path", path: ["map_size"]},
              operator: "<=",
              right: {kind: "integer", text: "65536"}
            },
            {
              left: {
                kind: "binary",
                left: {kind: "path", path: ["map_size"]},
                operator: "%",
                right: {kind: "path", path: ["desc_size"]}
              },
              operator: "==",
              right: {kind: "integer", text: "0"}
            }
          ] and
          (transition_targets($get_map) == [
            {
              target: "walk",
              arguments: [
                { kind: "name", path: ["bs"] },
                { kind: "name", path: ["handle"] },
                { kind: "name", path: ["stale_retry_used"] },
                { kind: "name", path: ["map_size"] },
                { kind: "name", path: ["desc_size"] },
                { kind: "name", path: ["key"] },
                { kind: "integer", text: "0" },
                { kind: "integer", text: "0" },
                { kind: "integer", text: "0" },
                { kind: "integer", text: "0" }
              ],
              guard: "when"
            },
            {
              target: "map_failed",
              arguments: [
                { kind: "name", path: ["bs"] },
                { kind: "name", path: ["handle"] },
                { kind: "name", path: ["stale_retry_used"] },
                { kind: "name", path: ["attempted_capacity"] },
                { kind: "name", path: ["status_code"] },
                { kind: "name", path: ["map_size"] }
              ],
              guard: "always"
            }
          ]);
    "fresh map/key transaction no longer gates descriptor walking")
| require(($map_failed.statements[0].guard.value.left | conjunct_signatures) == [
      {
        left: {kind: "path", path: ["attempted_capacity"]},
        operator: "==",
        right: {kind: "integer", text: "16384"}
      },
      {
        left: {kind: "path", path: ["status_code"]},
        operator: "==",
        right: {
          kind: "constant_member",
          type_name: "EfiStatus",
          member: "code",
          value: "0x8000000000000005"
        }
      },
      {
        left: {kind: "path", path: ["required_size"]},
        operator: ">",
        right: {kind: "integer", text: "16384"}
      },
      {
        left: {kind: "path", path: ["required_size"]},
        operator: "<=",
        right: {kind: "integer", text: "65536"}
      }
    ] and
    (transition_targets($map_failed) == [
      {
        target: "get_map",
        arguments: [
          { kind: "name", path: ["bs"] },
          { kind: "name", path: ["handle"] },
          { kind: "name", path: ["stale_retry_used"] },
          { kind: "integer", text: "65536" }
        ],
        guard: "when"
      },
      { target: "idle", arguments: [], guard: "always" }
    ]);
    "map growth is no longer one exact bounded EFI_BUFFER_TOO_SMALL retry")
| first(
    $walk.statements[]
    | select(.kind == "local_data" and .name == "is_free")
  ) as $is_free
| [$walk.statements[] | select(.kind == "transition")] as $walk_edges
| require(all(
      $walk.statements[]
      | select(.kind == "transition");
      .target.path[0] == "step"
      and .target.arguments[2] == {kind: "name", path: ["stale_retry_used"]}
      and .target.arguments[5] == {kind: "name", path: ["key"]}
    ) and
    ($walk.statements[0].initial_value.value == {
      kind: "indexed",
      collection: {
        kind: "member",
        receiver: {
          kind: "member",
          receiver: {kind: "name", path: ["self"]},
          member: "map_buf"
        },
        member: "bytes"
      },
      index: {kind: "name", path: ["offset"]}
    }) and
    (($is_free.initial_value | conjunct_signatures) == [
      {
        left: {
          kind: "member",
          receiver: {kind: "path", path: ["d"]},
          member: "kind"
        },
        operator: "==",
        right: {kind: "integer", text: "7"}
      },
      {
        left: {
          kind: "binary",
          left: {
            kind: "member",
            receiver: {kind: "path", path: ["d"]},
            member: "attribute"
          },
          operator: "&",
          right: {
            kind: "constant_member",
            type_name: "EfiMemoryAttribute",
            member: "bits",
            value: "0x8000000000000000"
          }
        },
        operator: "==",
        right: {kind: "integer", text: "0"}
      },
      {
        left: {
          kind: "binary",
          left: {
            kind: "member",
            receiver: {kind: "path", path: ["d"]},
            member: "attribute"
          },
          operator: "&",
          right: {
            kind: "constant_member",
            type_name: "EfiMemoryAttribute",
            member: "bits",
            value: "0x100000"
          }
        },
        operator: "==",
        right: {kind: "integer", text: "0"}
      },
      {
        left: {
          kind: "binary",
          left: {
            kind: "member",
            receiver: {kind: "path", path: ["d"]},
            member: "attribute"
          },
          operator: "&",
          right: {
            kind: "constant_member",
            type_name: "EfiMemoryAttribute",
            member: "bits",
            value: "0x40000"
          }
        },
        operator: "==",
        right: {kind: "integer", text: "0"}
      }
    ]) and
    ($walk_edges[0].target.arguments[9] == {kind: "name", path: ["offset"]}) and
    ($walk_edges[1].target.arguments[9] == {kind: "name", path: ["best_offset"]}) and
    ($step.statements[0].target.path[0] == "walk") and
    ($step.statements[0].guard.value.left.right.right.text == "65496") and
    ($step.statements[0].target.arguments[2] == {kind: "name", path: ["stale_retry_used"]}) and
    ($step.statements[0].target.arguments[5] == {kind: "name", path: ["key"]}) and
    ($step.statements[1].target.path[0] == "validate_candidate") and
    ($step.statements[1].target.arguments == [
      {kind: "name", path: ["bs"]},
      {kind: "name", path: ["handle"]},
      {kind: "name", path: ["stale_retry_used"]},
      {kind: "name", path: ["map_size"]},
      {kind: "name", path: ["desc_size"]},
      {kind: "name", path: ["key"]},
      {kind: "name", path: ["best_start"]},
      {kind: "name", path: ["best_pages"]},
      {kind: "name", path: ["best_offset"]}
    ]);
    "descriptor walk no longer excludes runtime/hot-plug/specific-purpose memory or carries its exact key")
| require(($validate_candidate.statements[0].guard.value.left | conjunct_signatures) == [
      {
        left: {kind: "path", path: ["best_pages"]},
        operator: ">",
        right: {kind: "integer", text: "0"}
      },
      {
        left: {kind: "path", path: ["best_pages"]},
        operator: "<=",
        right: {kind: "integer", text: "4503599627370495"}
      },
      {
        left: {
          kind: "binary",
          left: {kind: "path", path: ["best_start"]},
          operator: "%",
          right: {kind: "integer", text: "4096"}
        },
        operator: "==",
        right: {kind: "integer", text: "0"}
      }
    ] and
    transition_targets($validate_candidate) == [
      {
        target: "validate_candidate_end",
        arguments: [
          {kind: "name", path: ["bs"]},
          {kind: "name", path: ["handle"]},
          {kind: "name", path: ["stale_retry_used"]},
          {kind: "name", path: ["map_size"]},
          {kind: "name", path: ["desc_size"]},
          {kind: "name", path: ["key"]},
          {kind: "name", path: ["best_start"]},
          {kind: "name", path: ["best_pages"]},
          {kind: "name", path: ["best_offset"]}
        ],
        guard: "when"
      },
      {target: "idle", arguments: [], guard: "always"}
    ];
    "empty, oversized, or unaligned firmware spans no longer fail closed")
| require(($validate_candidate_end.parameters[7].type_reference == {
      kind: "constrained",
      base_type: {kind: "named", name: "u64"},
      constraints: [{
        kind: "range",
        minimum: {kind: "integer", text: "1"},
        maximum: {kind: "integer", text: "4503599627370495"}
      }]
    }) and
    ($validate_candidate_end.statements[1].initial_value == {
      kind: "binary",
      left: {kind: "name", path: ["checked_pages"]},
      operator: "*",
      right: {kind: "integer", text: "4096"}
    }) and
    ($validate_candidate_end.statements[3].initial_value == {
      kind: "integer",
      text: "0xffffffffffffffff"
    }) and
    ($validate_candidate_end.statements[4].guard.value.left | conjunct_signatures) == [
      {
        left: {kind: "path", path: ["best_start"]},
        operator: "<=",
        right: {
          kind: "binary",
          left: {kind: "path", path: ["max_address"]},
          operator: "-",
          right: {kind: "path", path: ["length"]}
        }
      }
    ] and
    transition_targets($validate_candidate_end) == [
      {
        target: "prepare_audit",
        arguments: [
          {kind: "name", path: ["bs"]},
          {kind: "name", path: ["handle"]},
          {kind: "name", path: ["stale_retry_used"]},
          {kind: "name", path: ["map_size"]},
          {kind: "name", path: ["desc_size"]},
          {kind: "name", path: ["key"]},
          {kind: "name", path: ["best_start"]},
          {kind: "name", path: ["best_pages"]},
          {kind: "name", path: ["length"]},
          {kind: "name", path: ["best_offset"]}
        ],
        guard: "when"
      },
      {target: "idle", arguments: [], guard: "always"}
    ];
    "validated span length no longer enters the overlap audit exactly")
| require(([$prepare_audit.parameters[].name] == [
      "bs", "handle", "stale_retry_used", "map_size", "desc_size", "key",
      "best_start", "best_pages", "length", "best_offset"
    ]) and
    ($prepare_audit.statements[2].initial_value == {
      kind: "binary",
      left: {kind: "name", path: ["checked_start"]},
      operator: "+",
      right: {kind: "name", path: ["checked_length"]}
    }) and
    (transition_targets($prepare_audit) == [{
      target: "audit_descriptor",
      arguments: [
        {kind: "name", path: ["bs"]},
        {kind: "name", path: ["handle"]},
        {kind: "name", path: ["stale_retry_used"]},
        {kind: "name", path: ["map_size"]},
        {kind: "name", path: ["desc_size"]},
        {kind: "name", path: ["key"]},
        {kind: "name", path: ["best_start"]},
        {kind: "name", path: ["best_pages"]},
        {kind: "name", path: ["length"]},
        {kind: "name", path: ["best_end"]},
        {kind: "name", path: ["best_offset"]},
        {kind: "integer", text: "0"}
      ],
      guard: "always"
    }]);
    "overlap audit no longer begins at the first descriptor with guarded best-end geometry")
| require(([$audit_descriptor.parameters[].name] == [
      "bs", "handle", "stale_retry_used", "map_size", "desc_size", "key",
      "best_start", "best_pages", "length", "best_end", "best_offset",
      "audit_offset"
    ]) and
    ($audit_descriptor.statements[0].initial_value.value == {
      kind: "indexed",
      collection: {
        kind: "member",
        receiver: {
          kind: "member",
          receiver: {kind: "name", path: ["self"]},
          member: "map_buf"
        },
        member: "bytes"
      },
      index: {kind: "name", path: ["audit_offset"]}
    }) and
    ([
      $audit_descriptor.statements[]
      | select(.kind == "transition")
      | {target: .target.path[0], guard: .guard.kind}
    ] == [
      {target: "audit_geometry", guard: "always"}
    ]) and
    ($audit_descriptor.statements[1].target.arguments[12] == {
      kind: "member",
      receiver: {kind: "name", path: ["d"]},
      member: "kind"
    }) and
    ($audit_descriptor.statements[1].target.arguments[13] == {
      kind: "member",
      receiver: {kind: "name", path: ["d"]},
      member: "attribute"
    }) and
    ($audit_descriptor.statements[1].target.arguments[14] == {
      kind: "member",
      receiver: {kind: "name", path: ["d"]},
      member: "physical_start"
    }) and
    ($audit_descriptor.statements[1].target.arguments[15] == {
      kind: "member",
      receiver: {kind: "name", path: ["d"]},
      member: "virtual_start"
    }) and
    ($audit_descriptor.statements[1].target.arguments[16] == {
      kind: "member",
      receiver: {kind: "name", path: ["d"]},
      member: "number_of_pages"
    });
    "map audit no longer submits every descriptor to complete geometry validation")
| require((($audit_geometry.statements[0].guard.value.left | conjunct_signatures) == [
      {
        left: {
          kind: "binary",
          left: {kind: "path", path: ["other_kind"]},
          operator: "<",
          right: {kind: "integer", text: "16"}
        },
        operator: "||",
        right: {
          kind: "binary",
          left: {kind: "path", path: ["other_kind"]},
          operator: ">=",
          right: {kind: "integer", text: "0x70000000"}
        }
      },
      {
        left: {
          kind: "binary",
          left: {kind: "path", path: ["other_attribute"]},
          operator: "&",
          right: {
            kind: "constant_member",
            type_name: "EfiMemoryAttribute",
            member: "bits",
            value: "0x30000fffffe00fe0"
          }
        },
        operator: "==",
        right: {kind: "integer", text: "0"}
      },
      {
        left: {
          kind: "binary",
          left: {kind: "path", path: ["other_attribute"]},
          operator: "&",
          right: {
            kind: "constant_member",
            type_name: "EfiMemoryAttribute",
            member: "bits",
            value: "0xffff00000000000"
          }
        },
        operator: "<=",
        right: {
          left: {
            kind: "path",
            path: ["other_attribute"]
          },
          operator: "&",
          right: {
            kind: "constant_member",
            type_name: "EfiMemoryAttribute",
            member: "bits",
            value: "0x4000000000000000"
          },
          kind: "binary"
        }
      },
      {
        left: {kind: "path", path: ["other_pages"]},
        operator: ">",
        right: {kind: "integer", text: "0"}
      },
      {
        left: {kind: "path", path: ["other_pages"]},
        operator: "<=",
        right: {kind: "integer", text: "4503599627370495"}
      },
      {
        left: {
          kind: "binary",
          left: {kind: "path", path: ["other_start"]},
          operator: "%",
          right: {kind: "integer", text: "4096"}
        },
        operator: "==",
        right: {kind: "integer", text: "0"}
      },
      {
        left: {
          kind: "binary",
          left: {kind: "path", path: ["other_virtual_start"]},
          operator: "%",
          right: {kind: "integer", text: "4096"}
        },
        operator: "==",
        right: {kind: "integer", text: "0"}
      }
    ]) and
    ([
      $audit_geometry.statements[]
      | select(.kind == "transition")
      | {target: .target.path[0], guard: .guard.kind}
    ] == [
      {target: "audit_geometry_end", guard: "when"},
      {target: "idle", guard: "always"}
    ]);
    "descriptor type, attribute validity, or physical/virtual alignment no longer fails closed")
| require(($audit_geometry_end.parameters[14].name == "other_pages") and
    ($audit_geometry_end.parameters[14].type_reference == {
      kind: "constrained",
      base_type: {kind: "named", name: "u64"},
      constraints: [{
        kind: "range",
        minimum: {kind: "integer", text: "1"},
        maximum: {kind: "integer", text: "4503599627370495"}
      }]
    }) and
    ($audit_geometry_end.statements[1].initial_value == {
      kind: "binary",
      left: {kind: "name", path: ["checked_pages"]},
      operator: "*",
      right: {kind: "integer", text: "4096"}
    }) and
    ($audit_geometry_end.statements[4].initial_value == {
      kind: "binary",
      left: {kind: "name", path: ["checked_other_length"]},
      operator: "-",
      right: {kind: "integer", text: "1"}
    }) and
    (($audit_geometry_end.statements[6].guard.value.left | conjunct_signatures) == [
      {
        left: {kind: "path", path: ["other_start"]},
        operator: "<=",
        right: {
          kind: "binary",
          left: {kind: "path", path: ["max_address"]},
          operator: "-",
          right: {kind: "path", path: ["last_offset"]}
        }
      },
      {
        left: {kind: "path", path: ["other_virtual_start"]},
        operator: "<=",
        right: {
          kind: "binary",
          left: {kind: "path", path: ["max_address"]},
          operator: "-",
          right: {kind: "path", path: ["last_offset"]}
        }
      }
    ]) and
    ([
      $audit_geometry_end.statements[]
      | select(.kind == "transition")
      | {target: .target.path[0], guard: .guard.kind}
    ] == [
      {target: "audit_overlap", guard: "when"},
      {target: "idle", guard: "always"}
    ]);
    "descriptor physical or virtual end no longer rejects arithmetic overflow exactly")
| require((($audit_overlap.statements[0].guard.value.left | conjunct_signatures) == [{
      left: {kind: "path", path: ["audit_offset"]},
      operator: "==",
      right: {kind: "path", path: ["best_offset"]}
    }]) and
    ([
      $audit_overlap.statements[]
      | select(.kind == "transition")
      | {target: .target.path[0], guard: .guard.kind}
    ] == [
      {target: "audit_step", guard: "when"},
      {target: "audit_classify", guard: "always"}
    ]);
    "selected descriptor no longer skips only overlap comparison after validation")
| require(($audit_classify.statements[1].initial_value == {
      kind: "binary",
      left: {kind: "name", path: ["checked_start"]},
      operator: "+",
      right: {kind: "name", path: ["last_offset"]}
    }) and
    (($audit_classify.statements[2].guard.value.left | conjunct_signatures) == [{
      left: {kind: "path", path: ["other_start"]},
      operator: "<",
      right: {kind: "path", path: ["best_start"]}
    }]) and
    ([
      $audit_classify.statements[]
      | select(.kind == "transition")
      | {target: .target.path[0], guard: .guard.kind}
    ] == [
      {target: "audit_left", guard: "when"},
      {target: "audit_right", guard: "always"}
    ]) and
    (($audit_left.statements[0].guard.value.left | conjunct_signatures) == [{
      left: {kind: "path", path: ["other_last"]},
      operator: "<",
      right: {kind: "path", path: ["best_start"]}
    }]) and
    (($audit_right.statements[0].guard.value.left | conjunct_signatures) == [{
      left: {kind: "path", path: ["other_start"]},
      operator: ">=",
      right: {kind: "path", path: ["best_end"]}
    }]) and
    ([
      $audit_left.statements[], $audit_right.statements[]
      | select(.kind == "transition")
      | {target: .target.path[0], guard: .guard.kind}
    ] == [
      {target: "audit_step", guard: "when"},
      {target: "idle", guard: "always"},
      {target: "audit_step", guard: "when"},
      {target: "idle", guard: "always"}
    ]);
    "overlapping descriptor no longer parks on both left and right cases")
| require((($audit_step.statements[0].guard.value.left | conjunct_signatures) == [
      {
        left: {
          kind: "binary",
          left: {
            kind: "binary",
            left: {kind: "path", path: ["audit_offset"]},
            operator: "+",
            right: {kind: "path", path: ["desc_size"]}
          },
          operator: "+",
          right: {kind: "path", path: ["desc_size"]}
        },
        operator: "<=",
        right: {kind: "path", path: ["map_size"]}
      },
      {
        left: {
          kind: "binary",
          left: {kind: "path", path: ["audit_offset"]},
          operator: "+",
          right: {kind: "path", path: ["desc_size"]}
        },
        operator: "<=",
        right: {kind: "integer", text: "65496"}
      }
    ]) and
    ($audit_step.statements[0].target.path[0] == "audit_descriptor") and
    ($audit_step.statements[0].target.arguments[11] == {
      kind: "binary",
      left: {kind: "name", path: ["audit_offset"]},
      operator: "+",
      right: {kind: "name", path: ["desc_size"]}
    }) and
    (transition_targets($audit_step)[1] == {
      target: "exit",
      arguments: [
        {kind: "name", path: ["bs"]},
        {kind: "name", path: ["handle"]},
        {kind: "name", path: ["stale_retry_used"]},
        {kind: "name", path: ["key"]},
        {kind: "name", path: ["best_start"]},
        {kind: "name", path: ["best_pages"]},
        {kind: "name", path: ["length"]}
      ],
      guard: "always"
    });
    "ExitBootServices no longer follows one complete bounded overlap audit")
| require(($exit.statements[0].kind == "local_data") and
          ($exit.statements[0].initial_value.kind == "call") and
          ($exit.statements[0].initial_value.target == "exit_boot_services") and
          ($exit.statements[0].initial_value.arguments[2] == {
            kind: "name",
            path: ["key"]
          }) and
          (transition_targets($exit) == [
            {
              target: "own",
              arguments: [
                { kind: "name", path: ["best_start"] },
                { kind: "name", path: ["best_pages"] },
                { kind: "name", path: ["length"] }
              ],
              guard: "when"
            },
            {
              target: "exit_failed",
              arguments: [
                { kind: "name", path: ["bs"] },
                { kind: "name", path: ["handle"] },
                { kind: "name", path: ["stale_retry_used"] },
                { kind: "name", path: ["exit_code"] }
              ],
              guard: "always"
            }
          ]);
    "ExitBootServices no longer separates successful ownership from failure handling")
| require(($exit_failed.parameters[2].name == "stale_retry_used") and
          ($exit_failed.parameters[2].type_reference.constraints[0] == {
            kind: "range",
            minimum: {kind: "integer", text: "0"},
            maximum: {kind: "integer", text: "1"}
          }) and
          (($exit_failed.statements[0].guard.value.left | conjunct_signatures) == [
            {
              left: {kind: "path", path: ["stale_retry_used"]},
              operator: "==",
              right: {kind: "integer", text: "0"}
            },
            {
              left: {kind: "path", path: ["exit_code"]},
              operator: "==",
              right: {
                kind: "constant_member",
                type_name: "EfiStatus",
                member: "code",
                value: "0x8000000000000002"
              }
            }
          ]) and
          (transition_targets($exit_failed) == [
            {
              target: "refresh_map",
              arguments: [
                { kind: "name", path: ["bs"] },
                { kind: "name", path: ["handle"] },
                { kind: "integer", text: "1" }
              ],
              guard: "when"
            },
            { target: "idle", arguments: [], guard: "always" }
          ]);
    "stale ExitBootServices retry is no longer one bounded whole-map transaction")
| first(
    $own.statements[]
    | select(.kind == "local_data" and .name == "first_extent")
  ) as $grant
| require(($grant.initial_value.kind == "call") and
          ($grant.initial_value.target == "grant") and
          ($grant.initial_value.arguments == [{ kind: "name", path: ["geometry"] }]) and
          ($own.statements[0].initial_value == {
            kind: "cast",
            value: {kind: "name", path: ["best_start"]},
            target_type: {kind: "named", name: "addr"},
            semantic_domain: [],
            semantic_domain_symbol: 0,
            semantic_domain_id: 0
          }) and
          ($own.statements[1].initial_value | field_value(.; "base")) == {
            kind: "name",
            path: ["base"]
          } and
          ($own.statements[1].initial_value | field_value(.; "length")) == {
            kind: "name",
            path: ["length"]
          } and
          (transition_targets($own) == [
            {
              target: "serial_init",
              arguments: [
                { kind: "name", path: ["first_extent"] },
                { kind: "name", path: ["best_pages"] }
              ],
              guard: "always"
            }
          ]);
    "qualified extent grant no longer immediately gates post-firmware serial initialization")
| require(($serial_init | has_granted_extent_parameter) and
          ($report_size | has_granted_extent_parameter) and
          ($oversized_mib | has_granted_extent_parameter) and
          ($tail_send | has_granted_extent_parameter) and
          ($owned_idle | has_granted_extent_parameter) and
          (($idle | has_granted_extent_parameter) | not) and
          (transition_targets($tail_send) == [
            {
              target: "owned_idle",
              arguments: [{ kind: "name", path: ["extent"] }],
              guard: "always"
            }
          ]) and
          (transition_targets($idle) == [
            { target: "idle", arguments: [], guard: "always" }
          ]) and
          (transition_targets($owned_idle) == [
            {
              target: "owned_idle",
              arguments: [{ kind: "name", path: ["extent"] }],
              guard: "always"
            }
          ]);
    "qualified extent no longer remains on the owned path or leaked onto failure idle")
| require((($to_mib.statements[0].guard.value.left | conjunct_signatures) == [
      {
        left: {kind: "path", path: ["pages"]},
        operator: ">=",
        right: {kind: "integer", text: "256"}
      },
      {
        left: {kind: "path", path: ["mib"]},
        operator: "<",
        right: {kind: "integer", text: "100000"}
      }
    ]) and
    (transition_targets($to_mib) == [
      {
        target: "to_mib",
        arguments: [
          {kind: "name", path: ["extent"]},
          {kind: "binary"},
          {kind: "binary"}
        ],
        guard: "when"
      },
      {
        target: "owned_wait",
        arguments: [
          {kind: "name", path: ["extent"]},
          {kind: "name", path: ["mib"]},
          {kind: "integer", text: "1000000"}
        ],
        guard: "always"
      }
    ]) and
    ($to_mib.statements[0].target.arguments[1] == {
      kind: "binary",
      left: {kind: "name", path: ["pages"]},
      operator: "-",
      right: {kind: "integer", text: "256"}
    }) and
    ($to_mib.statements[0].target.arguments[2] == {
      kind: "binary",
      left: {kind: "name", path: ["mib"]},
      operator: "+",
      right: {kind: "integer", text: "1"}
    });
    "page-to-MiB conversion no longer saturates after at most 100000 rounds")
| require(([$owned_wait.statements[] | .kind] == [
      "local_data", "assignment", "transition", "transition"
    ]) and
    ($owned_wait.statements[1] == {
      kind: "assignment",
      target: {kind: "name", path: ["lsr"]},
      value: {
        kind: "call",
        receiver: null,
        target: "asm#port_in",
        machine_arguments: [],
        arguments: [{kind: "integer", text: "1021"}],
        acknowledgement_synthesized: false,
        acknowledges_suspend: false,
        acknowledges_block: false
      }
    }) and
    ([$owned_wait_busy.statements[] | .kind] == ["transition", "transition"]) and
    ([$digits_wait.statements[] | .kind] == [
      "local_data", "assignment", "transition", "transition"
    ]) and
    ($digits_wait.statements[1] == {
      kind: "assignment",
      target: {kind: "name", path: ["lsr"]},
      value: {
        kind: "call",
        receiver: null,
        target: "asm#port_in",
        machine_arguments: [],
        arguments: [{kind: "integer", text: "1021"}],
        acknowledgement_synthesized: false,
        acknowledges_suspend: false,
        acknowledges_block: false
      }
    }) and
    ([$digits_wait_busy.statements[] | .kind] == ["transition", "transition"]);
    "UART readiness paths no longer perform exactly one LSR read and no busy-path writes")
| require(((first($owned_wait.statements[] | select(.kind == "transition")).guard.value.left
             | conjunct_signatures) == [{
      left: {
        kind: "binary",
        left: {kind: "path", path: ["lsr"]},
        operator: "&",
        right: {kind: "integer", text: "32"}
      },
      operator: "==",
      right: {kind: "integer", text: "0"}
    }]) and
    (transition_targets($owned_wait) == [
      {
        target: "owned_wait_busy",
        arguments: [
          {kind: "name", path: ["extent"]},
          {kind: "name", path: ["mib"]},
          {kind: "name", path: ["remaining_polls"]}
        ],
        guard: "when"
      },
      {
        target: "owned_send",
        arguments: [
          {kind: "name", path: ["extent"]},
          {kind: "name", path: ["mib"]}
        ],
        guard: "always"
      }
    ]) and
    (($owned_wait_busy.statements[0].guard.value.left | conjunct_signatures) == [{
      left: {kind: "path", path: ["remaining_polls"]},
      operator: ">",
      right: {kind: "integer", text: "1"}
    }]) and
    (transition_targets($owned_wait_busy) == [
      {
        target: "owned_wait",
        arguments: [
          {kind: "name", path: ["extent"]},
          {kind: "name", path: ["mib"]},
          {kind: "binary"}
        ],
        guard: "when"
      },
      {
        target: "owned_idle",
        arguments: [{kind: "name", path: ["extent"]}],
        guard: "always"
      }
    ]) and
    ($owned_wait_busy.statements[0].target.arguments[2] == {
      kind: "binary",
      left: {kind: "name", path: ["remaining_polls"]},
      operator: "-",
      right: {kind: "integer", text: "1"}
    }) and
    (transition_targets($owned_send) == [{
      target: "digits_wait",
      arguments: [
        {kind: "name", path: ["extent"]},
        {kind: "name", path: ["mib"]},
        {kind: "integer", text: "1000000"}
      ],
      guard: "always"
    }]) and
    ((first($digits_wait.statements[] | select(.kind == "transition")).guard.value.left
       | conjunct_signatures) == [{
      left: {
        kind: "binary",
        left: {kind: "path", path: ["lsr"]},
        operator: "&",
        right: {kind: "integer", text: "32"}
      },
      operator: "==",
      right: {kind: "integer", text: "0"}
    }]) and
    (transition_targets($digits_wait) == [
      {
        target: "digits_wait_busy",
        arguments: [
          {kind: "name", path: ["extent"]},
          {kind: "name", path: ["mib"]},
          {kind: "name", path: ["remaining_polls"]}
        ],
        guard: "when"
      },
      {
        target: "report_size",
        arguments: [
          {kind: "name", path: ["extent"]},
          {kind: "name", path: ["mib"]}
        ],
        guard: "always"
      }
    ]) and
    (($digits_wait_busy.statements[0].guard.value.left | conjunct_signatures) == [{
      left: {kind: "path", path: ["remaining_polls"]},
      operator: ">",
      right: {kind: "integer", text: "1"}
    }]) and
    (transition_targets($digits_wait_busy) == [
      {
        target: "digits_wait",
        arguments: [
          {kind: "name", path: ["extent"]},
          {kind: "name", path: ["mib"]},
          {kind: "binary"}
        ],
        guard: "when"
      },
      {
        target: "owned_idle",
        arguments: [{kind: "name", path: ["extent"]}],
        guard: "always"
      }
    ]) and
    ($digits_wait_busy.statements[0].target.arguments[2] == {
      kind: "binary",
      left: {kind: "name", path: ["remaining_polls"]},
      operator: "-",
      right: {kind: "integer", text: "1"}
    }) and
    (($report_size.statements[0].guard.value.left | conjunct_signatures) == [{
      left: {kind: "path", path: ["mib"]},
      operator: "<=",
      right: {kind: "integer", text: "99999"}
    }]) and
    (transition_targets($report_size) == [
      {
        target: "ten_thousands",
        arguments: [
          {kind: "name", path: ["extent"]},
          {kind: "name", path: ["mib"]},
          {kind: "integer", text: "0"}
        ],
        guard: "when"
      },
      {
        target: "oversized_mib",
        arguments: [{kind: "name", path: ["extent"]}],
        guard: "always"
      }
    ]) and
    ([
      $oversized_mib.statements[]
      | select(.kind == "call")
      | {target, arguments: [.arguments[] | {kind, text}]}
    ] == [
      {target: "asm#port_out", arguments: [{kind: "integer", text: "1016"}, {kind: "integer", text: "57"}]},
      {target: "asm#port_out", arguments: [{kind: "integer", text: "1016"}, {kind: "integer", text: "57"}]},
      {target: "asm#port_out", arguments: [{kind: "integer", text: "1016"}, {kind: "integer", text: "57"}]},
      {target: "asm#port_out", arguments: [{kind: "integer", text: "1016"}, {kind: "integer", text: "57"}]},
      {target: "asm#port_out", arguments: [{kind: "integer", text: "1016"}, {kind: "integer", text: "57"}]},
      {target: "asm#port_out", arguments: [{kind: "integer", text: "1016"}, {kind: "integer", text: "43"}]}
    ]) and
    (transition_targets($oversized_mib) == [{
      target: "tail_send",
      arguments: [{kind: "name", path: ["extent"]}],
      guard: "always"
    }]);
    "serial report no longer bounds UART polling, preserves the owned root on exhaustion, or emits the honest large bound")
| [
    $machine.states[]
    | select(has_granted_extent_parameter)
    | .name
  ] as $qualified_states
| [
    "serial_init",
    "to_mib",
    "owned_wait",
    "owned_wait_busy",
    "owned_send",
    "digits_wait",
    "digits_wait_busy",
    "report_size",
    "oversized_mib",
    "ten_thousands",
    "ten_thousands_emit",
    "ten_thousands_digit",
    "thousands",
    "thousands_emit",
    "thousands_digit",
    "hundreds",
    "hundreds_emit",
    "hundreds_digit",
    "tens",
    "tens_emit",
    "tens_digit",
    "ones_digit",
    "tail_send",
    "owned_idle"
  ] as $expected_qualified_states
| require($qualified_states == $expected_qualified_states;
    "post-grant serial/report graph no longer carries exactly one qualified extent")
| [
    $qualification.qualification_evidence[]
    | select(.subject == "Main::own_machine::own::first_extent")
  ] as $local_grants
| require(($local_grants | length) == 1 and
          $local_grants[0].domain == "Extent::Granted" and
          $local_grants[0].origin == "admitted_receipt" and
          $local_grants[0].program_point == "statement" and
          $local_grants[0].source == "ExtentRootProvider" and
          $local_grants[0].requirement == "ExtentRootProvider::grant";
    "owned-machine first extent lost its admitted provider receipt")
| [
    $qualification.qualification_evidence[]
    | select(.origin == "propagated" and (.subject | startswith("Main::own_machine::")))
    | .source
  ] as $propagated_sources
| [$expected_qualified_states[] | "Main::own_machine::" + .] as $expected_sources
| require($propagated_sources == $expected_sources;
    "qualification evidence no longer follows the complete post-grant graph")
| require($machine_contract.contract.service_reach == {
            interface: "published_ceiling",
            services: ["BootServices", "ExtentRootProvider", "MachineControl", "PortIo"]
          } and
          $machine_contract.implementation.checked_service_reach == [
            "BootServices", "ExtentRootProvider", "MachineControl", "PortIo"
          ] and
          $machine_contract.implementation.checked_synchronous_invocations == [
            "service:BootServices", "service:ExtentRootProvider"
          ] and
          $machine_contract.implementation.checked_may_suspend == false and
          $machine_contract.implementation.checked_may_block == false and
          $machine_contract.implementation.checked_crash_sites == [];
    "owned-machine handoff lost exact service composition or gained suspension, blocking, or local crashes")
| true
