def field($name):
  .fields[] | select(.name == $name) | .value;

.[0] as $typed
| .[1] as $contracts
| [
    $typed
    | ..
    | objects
    | select(.name? as $name | [
        "x86_double_fault_entry_stack",
        "x86_legacy_timer_entry_stack",
        "x86_machine_check_entry_stack",
        "x86_nmi_entry_stack"
      ] | index($name))
  ] as $profile_machines
| [
    $profile_machines[]
    | .states[0].statements[0].value as $assignment
    | {
        machine: .name,
        state_count: (.states | length),
        state_name: .states[0].name,
        return_type: .states[0].return_type,
        statement_count: (.states[0].statements | length),
        value_type: $assignment.type_name,
        vector: ($assignment | field("vector") | .text),
        stack_type: ($assignment | field("stack") | .type_name),
        stack_class: ($assignment | field("stack") | field("stack_class") | .text),
        ist_index: ($assignment | field("stack") | field("ist_index") | .text)
      }
  ] | sort_by(.machine) as $assignments
| [
    $contracts.machines[]
    | select(.machine as $name | [
        "x86_double_fault_entry_stack",
        "x86_legacy_timer_entry_stack",
        "x86_machine_check_entry_stack",
        "x86_nmi_entry_stack"
      ] | index($name))
  ] | sort_by(.machine) as $profile_contracts
| [
    {
      name: "exact four vector/stack records",
      ok: ($assignments == [
        {
          machine: "x86_double_fault_entry_stack",
          state_count: 1,
          state_name: "entry",
          return_type: {kind: "named", name: "X86EntryStackAssignment"},
          statement_count: 1,
          value_type: "X86EntryStackAssignment",
          vector: "8",
          stack_type: "X86IstStackClass",
          stack_class: "1",
          ist_index: "1"
        },
        {
          machine: "x86_legacy_timer_entry_stack",
          state_count: 1,
          state_name: "entry",
          return_type: {kind: "named", name: "X86EntryStackAssignment"},
          statement_count: 1,
          value_type: "X86EntryStackAssignment",
          vector: "0x20",
          stack_type: "X86IstStackClass",
          stack_class: "4",
          ist_index: "4"
        },
        {
          machine: "x86_machine_check_entry_stack",
          state_count: 1,
          state_name: "entry",
          return_type: {kind: "named", name: "X86EntryStackAssignment"},
          statement_count: 1,
          value_type: "X86EntryStackAssignment",
          vector: "18",
          stack_type: "X86IstStackClass",
          stack_class: "3",
          ist_index: "3"
        },
        {
          machine: "x86_nmi_entry_stack",
          state_count: 1,
          state_name: "entry",
          return_type: {kind: "named", name: "X86EntryStackAssignment"},
          statement_count: 1,
          value_type: "X86EntryStackAssignment",
          vector: "2",
          stack_type: "X86IstStackClass",
          stack_class: "2",
          ist_index: "2"
        }
      ])
    },
    {
      name: "four pure terminating profile machines",
      ok: (
        ($profile_contracts | length) == 4
        and all($profile_contracts[];
          .implementation.checked_may_suspend == false
          and .implementation.checked_may_block == false
          and .implementation.checked_service_reach == []
          and .implementation.checked_synchronous_invocations == []
          and .implementation.checked_crash_sites == []
          and .implementation.checked_crash_calls == []
          and .implementation.checked_termination.kind == "terminates"
          and (.implementation.inferred_write_frames | length) == 1
          and .implementation.inferred_write_frames[0].completeness == "complete"
          and .implementation.inferred_write_frames[0].paths == []
        )
      )
    }
  ]
| map(select(.ok | not) | .name) as $failures
| if ($failures | length) == 0 then
    true
  else
    error("x86 interrupt-profile mismatch: " + ($failures | join(", ")))
  end
