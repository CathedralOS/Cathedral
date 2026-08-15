def require($condition; $message):
  if $condition then . else error($message) end;

def typed_machine($typed; $name):
  first($typed | .. | objects | select(.name? == $name));

def contract_machine($contracts; $name):
  first($contracts | .. | objects | select(.machine? == $name));

def field_value($literal; $name):
  first($literal.fields[] | select(.name == $name) | .value);

def assignment($machine):
  $machine.states[0].statements[0].value;

def port_operations($machine):
  [
    $machine.states[0].statements[]
    | {
        target,
        arguments: [
          .arguments[]
          | if .kind == "integer" then { kind, text }
            elif .kind == "name" then { kind, path }
            else { kind }
            end
        ]
      }
  ];

def source_calls($machine):
  [
    $machine.states[0].statements[]
    | {
        kind,
        receiver,
        target,
        arguments: [
          .arguments[]
          | if .kind == "name" then { kind, path }
            elif .kind == "integer" then { kind, text }
            else { kind }
            end
        ]
      }
  ];

def pure_contract:
  (.contract.service_reach.interface == "internal_inferred") and
  (.implementation.checked_service_reach == []) and
  (.implementation.checked_synchronous_invocations == []) and
  (.implementation.checked_may_suspend == false) and
  (.implementation.checked_may_block == false) and
  (.implementation.checked_crash_sites == []) and
  (.implementation.checked_termination.kind == "terminates");

def portio_contract:
  (.contract.service_reach == {
    interface: "published_ceiling",
    services: ["PortIo"]
  }) and
  (.implementation.checked_service_reach == ["PortIo"]) and
  (.implementation.checked_synchronous_invocations == []) and
  (.implementation.checked_may_suspend == false) and
  (.implementation.checked_may_block == false) and
  (.implementation.checked_crash_sites == []) and
  (.implementation.checked_crash_calls == []) and
  (.implementation.checked_termination.kind == "terminates");

.[0] as $typed
| .[1] as $contracts
| typed_machine($typed; "x86_double_fault_entry_stack") as $double_fault
| typed_machine($typed; "x86_nmi_entry_stack") as $nmi
| typed_machine($typed; "x86_machine_check_entry_stack") as $machine_check
| typed_machine($typed; "x86_legacy_timer_entry_stack") as $timer
| typed_machine($typed; "x86_bootstrap_exception_entry_policy") as $exception_policy
| typed_machine($typed; "X86IdtGateLayout::plan") as $gate_layout
| typed_machine($typed; "Pic8259::remap_masked") as $pic_remap
| typed_machine($typed; "Pic8259::unmask_timer") as $pic_unmask
| typed_machine($typed; "Pit8254::program_channel0_rate_generator") as $pit_program
| typed_machine($typed; "legacy_pit_bootstrap_rate_policy") as $rate_policy
| typed_machine($typed; "LegacyBootstrapTimerRouteRatePolicy") as $route_rate_type
| typed_machine($typed; "legacy_bootstrap_timer_route_rate_policy") as $route_rate_policy
| typed_machine($typed; "LegacyPicPitTimerProvider::prepare_masked") as $prepare_masked
| require(all([
      $double_fault,
      $nmi,
      $machine_check,
      $timer,
      $exception_policy,
      $gate_layout,
      $pic_remap,
      $pic_unmask,
      $pit_program,
      $rate_policy,
      $route_rate_type,
      $route_rate_policy,
      $prepare_masked
    ][]; . != null);
    "bootstrap composition is missing a required typed machine")
| [
    assignment($double_fault),
    assignment($nmi),
    assignment($machine_check)
    | {
        vector: (field_value(.; "vector").text | tonumber),
        stack_class: (field_value(field_value(.; "stack"); "stack_class").text | tonumber),
        ist_index: (field_value(field_value(.; "stack"); "ist_index").text | tonumber)
      }
  ] as $dedicated_assignments
| [
    $exception_policy.states[0:3][]
    | {
        vector: (.statements[0].guard.value.left.right.text | tonumber),
        stack_class: (
          field_value(.statements[0].target.arguments[1]; "stack_class").text
          | tonumber
        ),
        ist_index: (
          field_value(.statements[0].target.arguments[1]; "ist_index").text
          | tonumber
        ),
        fallback: .statements[1].target.path[0]
      }
  ] as $exception_policy_branches
| (
    $exception_policy.states[]
    | select(.name == "current_stack")
    | .statements[0].value
  ) as $current_stack_policy
| assignment($timer) as $timer_assignment
| field_value($timer_assignment; "vector").text as $timer_vector
| field_value($timer_assignment; "stack") as $timer_stack
| port_operations($pic_remap) as $remap_operations
| port_operations($pic_unmask) as $unmask_operations
| port_operations($pit_program) as $pit_operations
| assignment($rate_policy) as $selected_rate_policy
| assignment($route_rate_policy) as $selected_route_rate_policy
| source_calls($prepare_masked) as $prepare_operations
| require($dedicated_assignments == [
      { vector: 8,  stack_class: 1, ist_index: 1 },
      { vector: 2,  stack_class: 2, ist_index: 2 },
      { vector: 18, stack_class: 3, ist_index: 3 }
    ] and
    all($dedicated_assignments[]; .vector < 32);
    "dedicated fatal exception assignments no longer occupy the exception floor")
| require(($exception_policy.states | map(.name)) == [
      "entry",
      "double_fault",
      "machine_check",
      "dedicated",
      "current_stack"
    ] and
    $exception_policy_branches == [
      { vector: 2,  stack_class: 2, ist_index: 2, fallback: "double_fault" },
      { vector: 8,  stack_class: 1, ist_index: 1, fallback: "machine_check" },
      { vector: 18, stack_class: 3, ist_index: 3, fallback: "current_stack" }
    ] and
    field_value($current_stack_policy; "stack").path == ["EntryStack", "Interrupted"] and
    field_value($current_stack_policy; "ist_index").text == "0" and
    field_value($current_stack_policy; "disposition").path == [
      "X86BootstrapExceptionDisposition",
      "FatalDiagnostic"
    ];
    "complete exception policy no longer covers the fatal 0-31 floor")
| require($timer_vector == $remap_operations[2].arguments[1].text and
          $timer_vector == "0x20" and
          field_value($timer_stack; "stack_class").text == "4" and
          field_value($timer_stack; "ist_index").text == "4";
    "legacy timer vector, PIC master remap, and shared maskable-IRQ stack disagree")
| first(
    $gate_layout.states[0].statements[]
    | select(.kind == "expression" and .value.type_name? == "Plan")
    | .value
  ) as $gate_plan
| require(field_value($gate_plan; "entry_count").text == "7" and
          field_value($gate_plan; "size_fixed").text == "16" and
          field_value($gate_plan; "size_is_dynamic").value == false and
          field_value($gate_plan; "align").text == "16";
    "bootstrap bundle lost the pure fixed x86 IDT gate geometry")
| require(($remap_operations | length) == 10 and
          $remap_operations[-2:] == [
            {
              target: "asm#port_out",
              arguments: [
                { kind: "integer", text: "0x21" },
                { kind: "integer", text: "0xff" }
              ]
            },
            {
              target: "asm#port_out",
              arguments: [
                { kind: "integer", text: "0xa1" },
                { kind: "integer", text: "0xff" }
              ]
            }
          ];
    "PIC remap phase no longer ends with both controllers masked")
| require($pit_operations == [
      {
        target: "asm#port_out",
        arguments: [
          { kind: "integer", text: "0x43" },
          { kind: "integer", text: "0x34" }
        ]
      },
      {
        target: "asm#port_out",
        arguments: [
          { kind: "integer", text: "0x40" },
          { kind: "name", path: ["divisor_low"] }
        ]
      },
      {
        target: "asm#port_out",
        arguments: [
          { kind: "integer", text: "0x40" },
          { kind: "name", path: ["divisor_high"] }
        ]
      }
    ];
    "PIT programming is no longer a distinct command/low/high phase")
| require($rate_policy.states == [{
      name: "entry",
      parameters: [],
      return_type: {kind: "named", name: "LegacyPitBootstrapRatePolicy"},
      contracts: [],
      statements: [{kind: "expression", value: $selected_rate_policy}]
    }] and
    field_value($selected_rate_policy; "input_hz").text == "1193182" and
    field_value($selected_rate_policy; "target_hz").text == "100" and
    field_value($selected_rate_policy; "divisor").text == "11932" and
    field_value($selected_rate_policy; "divisor_low").text == "0x9c" and
    field_value($selected_rate_policy; "divisor_high").text == "0x2e" and
    ((1193182 + 50) / 100 | floor) == 11932;
    "legacy bootstrap PIT policy no longer selects the rounded 100 Hz divisor 0x2e9c")
| require(($route_rate_type.members | map([
      .name,
      .relevance,
      .type_reference.kind,
      .type_reference.name
    ])) == [
      ["entry", "relevant", "named", "X86EntryStackAssignment"],
      ["rate", "relevant", "named", "LegacyPitBootstrapRatePolicy"]
    ] and
    ($route_rate_policy.states | length) == 1 and
    $route_rate_policy.states[0].name == "entry" and
    $route_rate_policy.states[0].parameters == [] and
    $route_rate_policy.states[0].return_type == {
      kind: "named",
      name: "LegacyBootstrapTimerRouteRatePolicy"
    } and
    ($route_rate_policy.states[0].statements | length) == 1 and
    $selected_route_rate_policy.type_name ==
      "LegacyBootstrapTimerRouteRatePolicy" and
    ($selected_route_rate_policy.fields | map([
      .name,
      .value.kind,
      .value.receiver,
      .value.target,
      .value.arguments
    ])) == [
      ["entry", "call", null, "x86_legacy_timer_entry_stack", []],
      ["rate", "call", null, "legacy_pit_bootstrap_rate_policy", []]
    ];
    "legacy timer route/rate policy no longer composes the exact authored facts")
| require(($prepare_masked.states | length) == 1 and
          $prepare_masked.states[0].name == "prepare_masked" and
          [$prepare_masked.states[0].parameters[] | {
            name,
            is_self,
            is_mutable,
            type: (
              if .type_reference.kind == "named" then .type_reference.name
              else .type_reference.referee.name
              end
            )
          }] == [
            { name: "self", is_self: true, is_mutable: true, type: "Self" }
          ] and
          $prepare_operations == [
            {
              kind: "call",
              receiver: ["self", "pic"],
              target: "remap_masked",
              arguments: []
            },
            {
              kind: "call",
              receiver: ["self", "pit"],
              target: "program_channel0_rate_generator",
              arguments: [
                { kind: "integer", text: "0x9c" },
                { kind: "integer", text: "0x2e" }
              ]
            }
          ];
    "masked legacy-timer preparation no longer owns the exact 100 Hz divisor policy")
| require($unmask_operations == [
      {
        target: "asm#port_out",
        arguments: [
          { kind: "integer", text: "0x21" },
          { kind: "integer", text: "0xfe" }
        ]
      },
      {
        target: "asm#port_out",
        arguments: [
          { kind: "integer", text: "0xa1" },
          { kind: "integer", text: "0xff" }
        ]
      }
    ];
    "timer unmask phase no longer admits only master IRQ0")
| [
    "x86_double_fault_entry_stack",
    "x86_nmi_entry_stack",
    "x86_machine_check_entry_stack",
    "x86_legacy_timer_entry_stack",
    "x86_bootstrap_exception_entry_policy",
    "X86IdtGateLayout::plan",
    "legacy_pit_bootstrap_rate_policy",
    "legacy_bootstrap_timer_route_rate_policy"
    | contract_machine($contracts; .)
  ] as $pure_contracts
| [
    "Pic8259::remap_masked",
    "Pic8259::unmask_timer",
    "Pit8254::program_channel0_rate_generator",
    "LegacyPicPitTimerProvider::prepare_masked"
    | contract_machine($contracts; .)
  ] as $port_contracts
| require(all($pure_contracts[]; pure_contract) and
          all(
            $pure_contracts[]
            | select(
                .machine != "x86_bootstrap_exception_entry_policy" and
                .machine != "legacy_bootstrap_timer_route_rate_policy"
              );
            .implementation.checked_crash_calls == []
          ) and
          (
            $pure_contracts[]
            | select(.machine == "x86_bootstrap_exception_entry_policy")
            | .implementation.checked_crash_calls
            | map([
                .state,
                .statement_ordinal,
                .call_ordinal,
                .target_machine,
                .target_state,
                .surviving_buckets
              ])
          ) == [
            ["dedicated", 0, 0, "x86_exception_delivery_shape", "entry", []],
            ["current_stack", 0, 0, "x86_exception_delivery_shape", "entry", []]
          ] and
          (
            $pure_contracts[]
            | select(.machine == "legacy_bootstrap_timer_route_rate_policy")
            | .implementation.checked_crash_calls
            | map([
                .state,
                .statement_ordinal,
                .call_ordinal,
                .target_machine,
                .target_state,
                .path_guard_conjuncts,
                .path_guard_consequences,
                .surviving_buckets
              ])
          ) == [
            ["entry", 0, 0, "x86_legacy_timer_entry_stack", "entry", [], [], []],
            ["entry", 0, 1, "legacy_pit_bootstrap_rate_policy", "entry", [], [], []]
          ] and
          (
            $pure_contracts[]
            | select(.machine == "legacy_bootstrap_timer_route_rate_policy")
            | .implementation.inferred_write_frames
            | map({state, completeness, paths})
          ) == [{
            state: "entry",
            completeness: "complete",
            paths: []
          }] and
          all($port_contracts[]; portio_contract);
    "bootstrap composition gained authority beyond pure policy/layout and exact PortIo leaves")
| [
    $port_contracts[]
    | {
        machine: .machine,
        frames: [
          .implementation.inferred_write_frames[]
          | {state, completeness, paths}
        ]
      }
  ] as $port_frames
| require($port_frames == [
      {
        machine: "Pic8259::remap_masked",
        frames: [{
          state: "remap_masked",
          completeness: "complete",
          paths: ["self"]
        }]
      },
      {
        machine: "Pic8259::unmask_timer",
        frames: [{
          state: "unmask_timer",
          completeness: "complete",
          paths: ["self"]
        }]
      },
      {
        machine: "Pit8254::program_channel0_rate_generator",
        frames: [{
          state: "program_channel0_rate_generator",
          completeness: "complete",
          paths: ["$P0", "$P1", "self"]
        }]
      },
      {
        machine: "LegacyPicPitTimerProvider::prepare_masked",
        frames: [{
          state: "prepare_masked",
          completeness: "complete",
          paths: ["self.pic", "self.pit"]
        }]
      }
    ];
    "legacy timer preparation or a hardware leaf escaped its exact complete frame")
| true
