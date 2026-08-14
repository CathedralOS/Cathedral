def require($condition; $message):
  if $condition then . else error($message) end;

def typed_named($typed; $name; $member):
  [
    $typed
    | ..
    | objects
    | select(.name? == $name and has($member))
  ];

def constrained_vector:
  . == {
    kind: "constrained",
    base_type: {kind: "named", name: "u32"},
    constraints: [{
      kind: "range",
      minimum: {kind: "integer", text: "0"},
      maximum: {kind: "integer", text: "31"}
    }]
  };

.[0] as $typed
| .[1] as $contracts
| typed_named($typed; "BootstrapFatalExceptionDiagnosticState"; "members") as $state_types
| typed_named(
    $typed;
    "BootstrapFatalExceptionDiagnosticState::record_and_abort";
    "states"
  ) as $machines
| [
    $contracts.machines[]
    | select(.machine == "BootstrapFatalExceptionDiagnosticState::record_and_abort")
  ] as $machine_contracts
| require(($state_types | length) == 1;
    "expected exactly one bootstrap fatal-diagnostic state declaration")
| require(($machines | length) == 1;
    "expected exactly one bootstrap fatal-diagnostic work machine")
| require(($machine_contracts | length) == 1;
    "expected exactly one bootstrap fatal-diagnostic machine contract")
| $state_types[0] as $state_type
| $machines[0] as $machine
| $machine_contracts[0] as $machine_contract
| require([
      $state_type.members[]
      | {kind, name, relevance, type: .type_reference.name}
    ] == [
      {kind: "field", name: "vector", relevance: "relevant", type: "AtomicU32"},
      {kind: "field", name: "published", relevance: "relevant", type: "AtomicU32"}
    ];
    "fatal-diagnostic state is no longer exactly one atomic vector and validity marker")
| require(($machine.states | length) == 1 and
          $machine.states[0].name == "record_and_abort" and
          $machine.states[0].return_type == null and
          ($machine.states[0].parameters | length) == 2 and
          $machine.states[0].parameters[0] == {
            name: "self",
            type_reference: {
              kind: "reference",
              referee: {kind: "named", name: "Self"},
              is_mutable: false
            },
            is_const: false,
            is_mutable: false,
            is_self: true
          } and
          $machine.states[0].parameters[1].name == "vector" and
          ($machine.states[0].parameters[1].type_reference | constrained_vector) and
          $machine.states[0].parameters[1].is_self == false and
          $machine.states[0].parameters[1].is_mutable == false;
    "fatal-diagnostic leaf no longer accepts shared preallocated state and one 0-31 vector")
| require($machine.contracts == [{
      kind: "crashes",
      crash_cause: "Abort",
      facts: [],
      token_count: 2
    }];
    "fatal-diagnostic leaf no longer publishes one unconditional Abort ceiling")
| require($machine.states[0].statements == [
      {
        kind: "assignment",
        target: {
          kind: "member",
          receiver: {kind: "name", path: ["self"]},
          member: "vector"
        },
        value: {
          kind: "atomic",
          value: {kind: "name", path: ["vector"]},
          result: null,
          ordering: "Store(NoOrdering)"
        }
      },
      {
        kind: "local_data",
        name: "prior_published",
        type_reference: {kind: "named", name: "u32"},
        initial_value: {kind: "integer", text: "0"}
      },
      {
        kind: "assignment",
        target: {
          kind: "member",
          receiver: {kind: "name", path: ["self"]},
          member: "published"
        },
        value: {
          kind: "atomic",
          value: {
            kind: "binary",
            left: {kind: "name", path: ["prior_published"]},
            operator: "|",
            right: {kind: "integer", text: "1"}
          },
          result: {kind: "name", path: ["prior_published"]},
          ordering: "ReadModifyWrite(Publish)"
        }
      },
      {
        kind: "transition",
        target: {kind: "terminal"},
        continuation: null,
        guard: {kind: "always"},
        crash_cause: "Abort"
      }
    ];
    "fatal-diagnostic work is no longer exactly vector-store, release publication, then Abort")
| require($machine_contract.contract.supply == "checked_body" and
          $machine_contract.contract.service_reach.interface == "internal_inferred" and
          $machine_contract.contract.synchronous_invocation == {
            interface: "internal_inferred",
            targets: []
          } and
          $machine_contract.contract.crashes == {
            interface: "published_ceiling",
            buckets: [{cause: "Abort", alternative_guards: ["true"]}]
          } and
          $machine_contract.implementation.checked_service_reach == [] and
          $machine_contract.implementation.checked_synchronous_invocations == [] and
          $machine_contract.implementation.checked_may_suspend == false and
          $machine_contract.implementation.checked_may_block == false and
          $machine_contract.implementation.checked_crash_sites == [{
            state: "record_and_abort",
            statement_ordinal: 3,
            cause: "Abort",
            path_guard_conjuncts: [],
            path_guard_consequences: [],
            guard_covering_buckets: [1],
            covering_buckets: [1],
            frontier_lower_bound: []
          }] and
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
            state: "record_and_abort",
            completeness: "complete",
            paths: ["self.published", "self.vector"]
          }];
    "fatal-diagnostic work gained effects, calls, blocking, a return path, or a wider frame")
| true
