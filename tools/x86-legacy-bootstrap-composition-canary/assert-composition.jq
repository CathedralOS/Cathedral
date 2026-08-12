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
| typed_machine($typed; "X86IdtGateLayout::plan") as $gate_layout
| typed_machine($typed; "Pic8259::remap_masked") as $pic_remap
| typed_machine($typed; "Pic8259::unmask_timer") as $pic_unmask
| typed_machine($typed; "Pit8254::program_channel0_rate_generator") as $pit_program
| require(all([
      $double_fault,
      $nmi,
      $machine_check,
      $timer,
      $gate_layout,
      $pic_remap,
      $pic_unmask,
      $pit_program
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
| assignment($timer) as $timer_assignment
| field_value($timer_assignment; "vector").text as $timer_vector
| field_value($timer_assignment; "stack") as $timer_stack
| port_operations($pic_remap) as $remap_operations
| port_operations($pic_unmask) as $unmask_operations
| port_operations($pit_program) as $pit_operations
| require($dedicated_assignments == [
      { vector: 8,  stack_class: 1, ist_index: 1 },
      { vector: 2,  stack_class: 2, ist_index: 2 },
      { vector: 18, stack_class: 3, ist_index: 3 }
    ] and
    all($dedicated_assignments[]; .vector < 32);
    "dedicated fatal exception assignments no longer occupy the exception floor")
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
    "X86IdtGateLayout::plan"
    | contract_machine($contracts; .)
  ] as $pure_contracts
| [
    "Pic8259::remap_masked",
    "Pic8259::unmask_timer",
    "Pit8254::program_channel0_rate_generator"
    | contract_machine($contracts; .)
  ] as $port_contracts
| require(all($pure_contracts[]; pure_contract) and
          all($port_contracts[]; portio_contract);
    "bootstrap composition gained authority beyond pure policy/layout and exact PortIo leaves")
| true
