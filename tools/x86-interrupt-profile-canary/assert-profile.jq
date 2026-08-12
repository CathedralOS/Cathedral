def field($name):
  .fields[] | select(.name == $name) | .value;

def constrained_u8($minimum; $maximum):
  . == {
    kind: "constrained",
    base_type: {kind: "named", name: "u8"},
    constraints: [
      {
        kind: "range",
        minimum: {kind: "integer", text: $minimum},
        maximum: {kind: "integer", text: $maximum}
      }
    ]
  };

def expression_path:
  if .kind == "name" then
    .path
  elif .kind == "member" then
    [(.receiver | expression_path)[], .member]
  else
    null
  end;

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
| first(
    $typed
    | ..
    | objects
    | select(
        .name? == "X86BootstrapExceptionDisposition"
        and has("members")
      )
  ) as $exception_disposition
| first(
    $typed
    | ..
    | objects
    | select(
        .name? == "X86BootstrapExceptionEntryPolicy"
        and has("members")
      )
  ) as $exception_policy_type
| first(
    $typed
    | ..
    | objects
    | select(
        .name? == "x86_bootstrap_exception_entry_policy"
        and has("states")
      )
  ) as $exception_policy
| [
    $exception_policy.states[0:3][]
    | {
        state: .name,
        vector: .statements[0].guard.value.left.right.text,
        target: .statements[0].target.path[0],
        stack_class: (
          .statements[0].target.arguments[1]
          | field("stack_class")
          | .text
        ),
        ist_index: (
          .statements[0].target.arguments[1]
          | field("ist_index")
          | .text
        ),
        fallback: .statements[1].target.path[0]
      }
  ] as $exception_branches
| (
    $exception_policy.states[]
    | select(.name == "dedicated")
    | .statements[0].value
  ) as $dedicated_policy
| (
    $exception_policy.states[]
    | select(.name == "current_stack")
    | .statements[0].value
  ) as $current_stack_policy
| [
    $contracts.machines[]
    | select(.machine == "x86_bootstrap_exception_entry_policy")
  ] as $exception_policy_contracts
| first(
    $typed
    | ..
    | objects
    | select(
        .name? == "X86BootstrapExceptionGateCandidate"
        and has("members")
      )
  ) as $gate_candidate_type
| first(
    $typed
    | ..
    | objects
    | select(
        .name? == "X86BootstrapExceptionGatePolicyCheck"
        and has("members")
      )
  ) as $gate_policy_check_type
| first(
    $typed
    | ..
    | objects
    | select(
        .name? == "x86_validate_bootstrap_exception_gate_policy"
        and has("states")
      )
  ) as $gate_policy_validator
| [
    $gate_policy_validator.states[0:5][]
    | first(.statements[] | select(.kind == "transition")) as $success_statement
    | $success_statement.guard.value.left as $comparison
    | {
        state: .name,
        left: ($comparison.left | expression_path),
        right: (
          $comparison.right
          | if .kind == "integer" then .text else expression_path end
        ),
        success: $success_statement.target.path[0],
        failure: (
          first(.statements[] | select(.guard.kind? == "always"))
          | .target.path[0]
        )
      }
  ] as $gate_policy_checks
| [
    $contracts.machines[]
    | select(.machine == "x86_validate_bootstrap_exception_gate_policy")
  ] as $gate_policy_validator_contracts
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
    },
    {
      name: "total exception-policy data shape",
      ok: (
        $exception_disposition.members == [
          {
            kind: "variant",
            identity: null,
            name: "FatalDiagnostic",
            payload: [],
            retired_payload_identities: []
          }
        ]
        and ($exception_policy_type.members | map(.name)) == [
          "vector",
          "stack",
          "ist_index",
          "disposition"
        ]
        and ($exception_policy_type.members[0].type_reference | constrained_u8("0"; "31"))
        and $exception_policy_type.members[1].type_reference == {
          kind: "named",
          name: "EntryStack"
        }
        and ($exception_policy_type.members[2].type_reference | constrained_u8("0"; "7"))
        and $exception_policy_type.members[3].type_reference == {
          kind: "named",
          name: "X86BootstrapExceptionDisposition"
        }
      )
    },
    {
      name: "total exception-policy branch coverage",
      ok: (
        ($exception_policy.states | map(.name)) == [
          "entry",
          "double_fault",
          "machine_check",
          "dedicated",
          "current_stack"
        ]
        and all($exception_policy.states[];
          .return_type == {
            kind: "named",
            name: "X86BootstrapExceptionEntryPolicy"
          }
        )
        and all($exception_policy.states[].parameters[] | select(.name == "vector");
          .type_reference | constrained_u8("0"; "31")
        )
        and $exception_branches == [
          {
            state: "entry",
            vector: "2",
            target: "dedicated",
            stack_class: "2",
            ist_index: "2",
            fallback: "double_fault"
          },
          {
            state: "double_fault",
            vector: "8",
            target: "dedicated",
            stack_class: "1",
            ist_index: "1",
            fallback: "machine_check"
          },
          {
            state: "machine_check",
            vector: "18",
            target: "dedicated",
            stack_class: "3",
            ist_index: "3",
            fallback: "current_stack"
          }
        ]
      )
    },
    {
      name: "coupled dedicated and fatal current-stack policies",
      ok: (
        $dedicated_policy.type_name == "X86BootstrapExceptionEntryPolicy"
        and ($dedicated_policy | field("vector") | .path) == ["vector"]
        and ($dedicated_policy | field("stack") | .type_name) == "EntryStack"
        and ($dedicated_policy | field("stack") | field("class")) == {
          kind: "member",
          receiver: {kind: "name", path: ["selected"]},
          member: "stack_class"
        }
        and ($dedicated_policy | field("ist_index")) == {
          kind: "member",
          receiver: {kind: "name", path: ["selected"]},
          member: "ist_index"
        }
        and ($dedicated_policy | field("disposition") | .path) == [
          "X86BootstrapExceptionDisposition",
          "FatalDiagnostic"
        ]
        and $current_stack_policy.type_name == "X86BootstrapExceptionEntryPolicy"
        and ($current_stack_policy | field("vector") | .path) == ["vector"]
        and ($current_stack_policy | field("stack") | .path) == [
          "EntryStack",
          "Interrupted"
        ]
        and ($current_stack_policy | field("ist_index") | .text) == "0"
        and ($current_stack_policy | field("disposition") | .path) == [
          "X86BootstrapExceptionDisposition",
          "FatalDiagnostic"
        ]
      )
    },
    {
      name: "pure terminating total exception-policy machine",
      ok: (
        ($exception_policy_contracts | length) == 1
        and all($exception_policy_contracts[];
          .implementation.checked_may_suspend == false
          and .implementation.checked_may_block == false
          and .implementation.checked_service_reach == []
          and .implementation.checked_synchronous_invocations == []
          and .implementation.checked_crash_sites == []
          and .implementation.checked_crash_calls == []
          and .implementation.checked_termination.kind == "terminates"
          and (.implementation.inferred_write_frames | length) == 5
          and all(.implementation.inferred_write_frames[];
            .completeness == "complete"
            and .paths == []
          )
        )
      )
    },
    {
      name: "gate policy-candidate and partial-result shapes",
      ok: (
        ($gate_candidate_type.members | map(.name)) == [
          "vector",
          "gate",
          "disposition"
        ]
        and ($gate_candidate_type.members[0].type_reference | constrained_u8("0"; "31"))
        and $gate_candidate_type.members[1].type_reference == {
          kind: "named",
          name: "X86IdtGate"
        }
        and $gate_candidate_type.members[2].type_reference == {
          kind: "named",
          name: "X86BootstrapExceptionDisposition"
        }
        and ($gate_policy_check_type.members | map(.name)) == [
          "Rejected",
          "PolicyConsistent"
        ]
        and $gate_policy_check_type.members[0].payload == []
        and $gate_policy_check_type.members[1].payload == [
          {
            identity: null,
            name: "candidate",
            type_reference: {
              kind: "named",
              name: "X86BootstrapExceptionGateCandidate"
            }
          }
        ]
      )
    },
    {
      name: "gate policy validator checks only decided prepublication fields",
      ok: (
        ($gate_policy_validator.states | map(.name)) == [
          "entry",
          "check_ist",
          "check_disposition",
          "check_attributes",
          "check_reserved",
          "accept",
          "reject"
        ]
        and $gate_policy_validator.states[0].parameters[0].name == "expected_vector"
        and ($gate_policy_validator.states[0].parameters[0].type_reference
          | constrained_u8("0"; "31"))
        and $gate_policy_validator.states[0].statements[0].initial_value == {
          kind: "call",
          receiver: null,
          target: "x86_bootstrap_exception_entry_policy",
          machine_arguments: [],
          arguments: [{kind: "name", path: ["expected_vector"]}],
          acknowledgement_synthesized: false,
          acknowledges_suspend: false,
          acknowledges_block: false
        }
        and $gate_policy_checks == [
          {
            state: "entry",
            left: ["candidate", "vector"],
            right: ["policy", "vector"],
            success: "check_ist",
            failure: "reject"
          },
          {
            state: "check_ist",
            left: ["candidate", "gate", "ist"],
            right: ["policy", "ist_index"],
            success: "check_disposition",
            failure: "reject"
          },
          {
            state: "check_disposition",
            left: ["candidate", "disposition"],
            right: ["policy", "disposition"],
            success: "check_attributes",
            failure: "reject"
          },
          {
            state: "check_attributes",
            left: ["candidate", "gate", "type_attributes"],
            right: "0x8e",
            success: "check_reserved",
            failure: "reject"
          },
          {
            state: "check_reserved",
            left: ["candidate", "gate", "reserved"],
            right: "0",
            success: "accept",
            failure: "reject"
          }
        ]
        and all($gate_policy_validator.states[];
          .return_type == {
            kind: "named",
            name: "X86BootstrapExceptionGatePolicyCheck"
          }
        )
        and (
          $gate_policy_validator.states[]
          | select(.name == "accept")
          | .statements[0].value.type_name
        ) == "X86BootstrapExceptionGatePolicyCheck"
        and (
          $gate_policy_validator.states[]
          | select(.name == "reject")
          | .statements[0].value.path
        ) == ["X86BootstrapExceptionGatePolicyCheck", "Rejected"]
      )
    },
    {
      name: "pure terminating gate policy validator",
      ok: (
        ($gate_policy_validator_contracts | length) == 1
        and all($gate_policy_validator_contracts[];
          .implementation.checked_may_suspend == false
          and .implementation.checked_may_block == false
          and .implementation.checked_service_reach == []
          and .implementation.checked_synchronous_invocations == []
          and .implementation.checked_crash_sites == []
          and (.implementation.checked_crash_calls | length) == 1
          and .implementation.checked_crash_calls[0].state == "entry"
          and .implementation.checked_crash_calls[0].statement_ordinal == 0
          and .implementation.checked_crash_calls[0].call_ordinal == 0
          and .implementation.checked_crash_calls[0].target_machine == "x86_bootstrap_exception_entry_policy"
          and .implementation.checked_crash_calls[0].target_state == "entry"
          and .implementation.checked_crash_calls[0].path_guard_conjuncts == []
          and .implementation.checked_crash_calls[0].path_guard_consequences == []
          and .implementation.checked_crash_calls[0].surviving_buckets == []
          and .implementation.checked_termination.kind == "terminates"
          and (.implementation.inferred_write_frames | length) == 7
          and .implementation.inferred_write_frames[0].state == "entry"
          and .implementation.inferred_write_frames[0].completeness == "opaque"
          and .implementation.inferred_write_frames[0].paths == []
          and all(.implementation.inferred_write_frames[1:7][];
            .completeness == "complete"
            and .paths == []
          )
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
