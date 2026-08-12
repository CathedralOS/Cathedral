def require($condition; $message):
  if $condition then . else error($message) end;

def machine_signature:
  {
    machine: .name,
    state: .states[0].name,
    state_count: (.states | length),
    parameters: [
      .states[0].parameters[]
      | { name, is_self, is_mutable }
    ],
    operations: [
      .states[0].statements[]
      | {
          kind,
          target,
          arguments: [.arguments[] | { kind, text }]
        }
    ]
  };

.[0] as $typed
| .[1] as $contracts
| [
    $typed
    | ..
    | objects
    | select(.name? == "Pic8259::remap_masked" or .name? == "Pic8259::unmask_timer")
    | machine_signature
  ] | sort_by(.machine) as $actual_machines
| [
    {
      machine: "Pic8259::remap_masked",
      state: "remap_masked",
      state_count: 1,
      parameters: [
        { name: "self", is_self: true, is_mutable: true }
      ],
      operations: [
        { kind: "call", target: "asm#port_out", arguments: [{ kind: "integer", text: "0x20" }, { kind: "integer", text: "0x11" }] },
        { kind: "call", target: "asm#port_out", arguments: [{ kind: "integer", text: "0xa0" }, { kind: "integer", text: "0x11" }] },
        { kind: "call", target: "asm#port_out", arguments: [{ kind: "integer", text: "0x21" }, { kind: "integer", text: "0x20" }] },
        { kind: "call", target: "asm#port_out", arguments: [{ kind: "integer", text: "0xa1" }, { kind: "integer", text: "0x28" }] },
        { kind: "call", target: "asm#port_out", arguments: [{ kind: "integer", text: "0x21" }, { kind: "integer", text: "0x4" }] },
        { kind: "call", target: "asm#port_out", arguments: [{ kind: "integer", text: "0xa1" }, { kind: "integer", text: "0x2" }] },
        { kind: "call", target: "asm#port_out", arguments: [{ kind: "integer", text: "0x21" }, { kind: "integer", text: "0x1" }] },
        { kind: "call", target: "asm#port_out", arguments: [{ kind: "integer", text: "0xa1" }, { kind: "integer", text: "0x1" }] },
        { kind: "call", target: "asm#port_out", arguments: [{ kind: "integer", text: "0x21" }, { kind: "integer", text: "0xff" }] },
        { kind: "call", target: "asm#port_out", arguments: [{ kind: "integer", text: "0xa1" }, { kind: "integer", text: "0xff" }] }
      ]
    },
    {
      machine: "Pic8259::unmask_timer",
      state: "unmask_timer",
      state_count: 1,
      parameters: [
        { name: "self", is_self: true, is_mutable: true }
      ],
      operations: [
        { kind: "call", target: "asm#port_out", arguments: [{ kind: "integer", text: "0x21" }, { kind: "integer", text: "0xfe" }] },
        { kind: "call", target: "asm#port_out", arguments: [{ kind: "integer", text: "0xa1" }, { kind: "integer", text: "0xff" }] }
      ]
    }
  ] as $expected_machines
| require($actual_machines == $expected_machines;
    "8259 remap/mask phase order or timer-only unmask policy changed")
| [
    $contracts
    | ..
    | objects
    | select(.machine? == "Pic8259::remap_masked" or .machine? == "Pic8259::unmask_timer")
  ] | sort_by(.machine) as $machine_contracts
| require(($machine_contracts | length) == 2;
    "expected exactly two PIC initialization machine contracts")
| require(all($machine_contracts[];
      .contract.service_reach == {
        interface: "published_ceiling",
        services: ["PortIo"]
      }
      and .implementation.checked_service_reach == ["PortIo"]
      and .implementation.checked_may_suspend == false
      and .implementation.checked_may_block == false
      and .implementation.checked_synchronous_invocations == []
      and .implementation.checked_crash_sites == []
      and .implementation.checked_crash_calls == []
      and .implementation.checked_termination.kind == "terminates"
    );
    "PIC initialization leaves lost exact PortIo reach or gained calls, blocking, crashes, or nontermination")
| true
