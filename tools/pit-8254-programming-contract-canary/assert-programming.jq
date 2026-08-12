def require($condition; $message):
  if $condition then . else error($message) end;

.[0] as $typed
| .[1] as $contracts
| [
    $typed
    | ..
    | objects
    | select(.name? == "Pit8254::program_channel0_rate_generator")
  ] as $machines
| [
    $contracts
    | ..
    | objects
    | select(.machine? == "Pit8254::program_channel0_rate_generator")
  ] as $machine_contracts
| require(($machines | length) == 1;
    "expected exactly one typed Pit8254::program_channel0_rate_generator machine")
| require(($machine_contracts | length) == 1;
    "expected exactly one Pit8254::program_channel0_rate_generator contract")
| $machines[0] as $machine
| $machine_contracts[0] as $machine_contract
| require(($machine.states | length) == 1 and
          $machine.states[0].name == "program_channel0_rate_generator" and
          [$machine.states[0].parameters[] | {
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
          ];
    "PIT programming leaf no longer accepts exactly two validated divisor bytes")
| [
    $machine.states[0].statements[]
    | {
        kind,
        target,
        arguments: [
          .arguments[]
          | if .kind == "integer" then { kind, text }
            elif .kind == "name" then { kind, path }
            else { kind }
            end
        ]
      }
  ] as $actual_operations
| [
    {
      kind: "call",
      target: "asm#port_out",
      arguments: [
        { kind: "integer", text: "0x43" },
        { kind: "integer", text: "0x34" }
      ]
    },
    {
      kind: "call",
      target: "asm#port_out",
      arguments: [
        { kind: "integer", text: "0x40" },
        { kind: "name", path: ["divisor_low"] }
      ]
    },
    {
      kind: "call",
      target: "asm#port_out",
      arguments: [
        { kind: "integer", text: "0x40" },
        { kind: "name", path: ["divisor_high"] }
      ]
    }
  ] as $expected_operations
| require($actual_operations == $expected_operations;
    "PIT channel-0 command or low/high reload ordering changed")
| require($machine_contract.contract.service_reach == {
            interface: "published_ceiling",
            services: ["PortIo"]
          } and
          $machine_contract.implementation.checked_service_reach == ["PortIo"] and
          $machine_contract.implementation.checked_may_suspend == false and
          $machine_contract.implementation.checked_may_block == false and
          $machine_contract.implementation.checked_synchronous_invocations == [] and
          $machine_contract.implementation.checked_crash_sites == [] and
          $machine_contract.implementation.checked_crash_calls == [] and
          $machine_contract.implementation.checked_termination.kind == "terminates";
    "PIT programming leaf lost exact PortIo reach or gained calls, blocking, crashes, or nontermination")
| true
