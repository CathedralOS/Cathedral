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
  (.implementation.checked_crash_calls == []) and
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
            { name: "self",         is_self: true,  is_mutable: true,  type: "Self" },
            { name: "divisor_low",  is_self: false, is_mutable: false, type: "u8" },
            { name: "divisor_high", is_self: false, is_mutable: false, type: "u8" }
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
                { kind: "name", path: ["divisor_low"] },
                { kind: "name", path: ["divisor_high"] }
              ]
            }
          ];
    "masked legacy-timer preparation no longer remaps before exact divisor programming")
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
    "X86IdtGateLayout::plan"
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
          all($port_contracts[]; portio_contract);
    "bootstrap composition gained authority beyond pure policy/layout and exact PortIo leaves")
| true
