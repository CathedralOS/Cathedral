def require($condition; $message):
  if $condition then . else error($message) end;

def typed_named($typed; $name; $member):
  [
    $typed
    | ..
    | objects
    | select(.name? == $name and has($member))
  ];

.[0] as $typed
| .[1] as $contracts
| typed_named($typed; "TimerRootWakeState"; "members") as $state_types
| typed_named($typed; "TimerRootWakeState::record_and_publish"; "states") as $machines
| [
    $contracts.machines[]
    | select(.machine == "TimerRootWakeState::record_and_publish")
  ] as $machine_contracts
| require(($state_types | length) == 1;
    "expected exactly one TimerRootWakeState data declaration")
| require(($machines | length) == 1;
    "expected exactly one TimerRootWakeState::record_and_publish machine")
| require(($machine_contracts | length) == 1;
    "expected exactly one timer-root work machine contract")
| $state_types[0] as $state_type
| $machines[0] as $machine
| $machine_contracts[0] as $machine_contract
| require([
      $state_type.members[]
      | {kind, name, relevance, type: .type_reference.name}
    ] == [
      {
        kind: "field",
        name: "observed_time",
        relevance: "relevant",
        type: "AtomicU64"
      },
      {
        kind: "field",
        name: "wake_pending",
        relevance: "relevant",
        type: "AtomicU32"
      }
    ];
    "timer-root state is no longer exactly one atomic time and wake marker")
| require(($machine.states | length) == 1 and
          $machine.states[0].name == "record_and_publish" and
          $machine.states[0].return_type == null and
          [$machine.states[0].parameters[] | {
            name,
            is_self,
            is_mutable,
            type: (
              if .type_reference.kind == "reference" then {
                kind: "reference",
                mutable: .type_reference.is_mutable,
                referee: .type_reference.referee.name
              } else {
                kind: .type_reference.kind,
                name: .type_reference.name
              } end
            )
          }] == [
            {
              name: "self",
              is_self: true,
              is_mutable: false,
              type: {kind: "reference", mutable: false, referee: "Self"}
            },
            {
              name: "observed_time",
              is_self: false,
              is_mutable: false,
              type: {kind: "named", name: "u64"}
            }
          ];
    "timer-root work no longer accepts shared preallocated state and one time observation")
| require($machine.states[0].statements == [
      {
        kind: "assignment",
        target: {
          kind: "member",
          receiver: {kind: "name", path: ["self"]},
          member: "observed_time"
        },
        value: {
          kind: "atomic",
          value: {kind: "name", path: ["observed_time"]},
          result: null,
          ordering: "Store(NoOrdering)"
        }
      },
      {
        kind: "local_data",
        name: "prior_pending",
        type_reference: {kind: "named", name: "u32"},
        initial_value: {kind: "integer", text: "0"}
      },
      {
        kind: "assignment",
        target: {
          kind: "member",
          receiver: {kind: "name", path: ["self"]},
          member: "wake_pending"
        },
        value: {
          kind: "atomic",
          value: {
            kind: "binary",
            left: {kind: "name", path: ["prior_pending"]},
            operator: "|",
            right: {kind: "integer", text: "1"}
          },
          result: {kind: "name", path: ["prior_pending"]},
          ordering: "ReadModifyWrite(Publish)"
        }
      }
    ];
    "timer-root work is no longer exactly time-store then one-bit release publication")
| require($machine_contract.contract.supply == "checked_body" and
          $machine_contract.contract.service_reach.interface == "internal_inferred" and
          $machine_contract.contract.synchronous_invocation == {
            interface: "internal_inferred",
            targets: []
          } and
          $machine_contract.implementation.checked_service_reach == [] and
          $machine_contract.implementation.checked_synchronous_invocations == [] and
          $machine_contract.implementation.checked_may_suspend == false and
          $machine_contract.implementation.checked_may_block == false and
          $machine_contract.implementation.checked_crash_sites == [] and
          $machine_contract.implementation.checked_crash_calls == [] and
          $machine_contract.implementation.checked_termination == {
            kind: "terminates",
            premises: []
          } and
          [$machine_contract.implementation.inferred_write_frames[] | {
            state,
            completeness,
            paths
          }] == [{
            state: "record_and_publish",
            completeness: "complete",
            paths: ["self.observed_time", "self.wake_pending"]
          }];
    "timer-root work gained effects, calls, blocking, crashes, nontermination, or a wider frame")
| true
