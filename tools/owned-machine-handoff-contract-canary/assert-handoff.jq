def require($condition; $message):
  if $condition then . else error($message) end;

def state($machine; $name):
  first($machine.states[] | select(.name == $name));

def field_value($literal; $name):
  first($literal.fields[] | select(.name == $name) | .value);

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
| $machines[0] as $machine
| $machine_contracts[0] as $machine_contract
| state($machine; "own_machine") as $entry
| state($machine; "refresh_map") as $refresh_map
| state($machine; "walk") as $walk
| state($machine; "step") as $step
| state($machine; "exit") as $exit
| state($machine; "exit_failed") as $exit_failed
| state($machine; "own") as $own
| state($machine; "serial_init") as $serial_init
| state($machine; "tail_send") as $tail_send
| state($machine; "idle") as $idle
| state($machine; "owned_idle") as $owned_idle
| require(transition_targets($entry) == [
      {
        target: "refresh_map",
        arguments: [
          { kind: "name", path: ["bs"] },
          { kind: "name", path: ["handle"] }
        ],
        guard: "always"
      }
    ];
    "owned-machine entry no longer begins with one fresh map/key transaction")
| first(
    $refresh_map.statements[]
    | select(.kind == "local_data" and .name == "status")
  ) as $map_call
| require(($map_call.initial_value.kind == "call") and
          ($map_call.initial_value.target == "get_memory_map") and
          (transition_targets($refresh_map) == [
            {
              target: "walk",
              arguments: [
                { kind: "name", path: ["bs"] },
                { kind: "name", path: ["handle"] },
                { kind: "name", path: ["map_size"] },
                { kind: "name", path: ["desc_size"] },
                { kind: "name", path: ["key"] },
                { kind: "integer", text: "0" },
                { kind: "integer", text: "0" },
                { kind: "integer", text: "0" }
              ],
              guard: "when"
            },
            { target: "idle", arguments: [], guard: "always" }
          ]);
    "fresh map/key transaction no longer gates descriptor walking")
| require(all(
      $walk.statements[]
      | select(.kind == "transition");
      .target.path[0] == "step"
      and .target.arguments[4] == {kind: "name", path: ["key"]}
    ) and
    ($step.statements[0].target.path[0] == "walk") and
    ($step.statements[0].target.arguments[4] == {kind: "name", path: ["key"]}) and
    ($step.statements[1].target.path[0] == "exit") and
    ($step.statements[1].target.arguments[2] == {kind: "name", path: ["key"]});
    "descriptor walk no longer carries exactly the key returned with its map")
| require(($exit.statements[0].kind == "local_data") and
          ($exit.statements[0].initial_value.kind == "call") and
          ($exit.statements[0].initial_value.target == "exit_boot_services") and
          (transition_targets($exit) == [
            {
              target: "own",
              arguments: [
                { kind: "name", path: ["best_start"] },
                { kind: "name", path: ["best_pages"] }
              ],
              guard: "when"
            },
            {
              target: "exit_failed",
              arguments: [
                { kind: "name", path: ["bs"] },
                { kind: "name", path: ["handle"] },
                { kind: "name", path: ["exit_code"] }
              ],
              guard: "always"
            }
          ]);
    "ExitBootServices no longer separates successful ownership from failure handling")
| ($exit_failed.statements[0].guard.value.left.right) as $invalid_parameter
| require(($invalid_parameter.kind == "member") and
          ($invalid_parameter.member == "code") and
          ($invalid_parameter.receiver.type_name == "EfiStatus") and
          (($invalid_parameter.receiver | field_value(.; "code") | .text)
            == "0x8000000000000002") and
          (transition_targets($exit_failed) == [
            {
              target: "refresh_map",
              arguments: [
                { kind: "name", path: ["bs"] },
                { kind: "name", path: ["handle"] }
              ],
              guard: "when"
            },
            { target: "idle", arguments: [], guard: "always" }
          ]);
    "stale ExitBootServices key no longer refreshes the whole map/key transaction")
| first(
    $own.statements[]
    | select(.kind == "local_data" and .name == "first_extent")
  ) as $grant
| require(($grant.initial_value.kind == "call") and
          ($grant.initial_value.target == "grant") and
          ($grant.initial_value.arguments == [{ kind: "name", path: ["geometry"] }]) and
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
| [
    $machine.states[]
    | select(has_granted_extent_parameter)
    | .name
  ] as $qualified_states
| [
    "serial_init",
    "to_mib",
    "owned_wait",
    "owned_send",
    "digits_wait",
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
