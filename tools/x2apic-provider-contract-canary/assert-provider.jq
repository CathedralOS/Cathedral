def require($condition; $message):
  if $condition then . else error($message) end;

def type_signature:
  if .kind == "constrained" then
    {
      kind,
      base: .base_type.name,
      minimum: .constraints[0].minimum.text,
      maximum: .constraints[0].maximum.text
    }
  else
    { kind, name }
  end;

def operand_signature:
  if .kind == "integer" then
    { kind, text }
  elif .kind == "name" then
    { kind, path }
  else
    { kind }
  end;

def machine_signature:
  {
    machine: .name,
    state: .states[0].name,
    state_count: (.states | length),
    self_is_mutable: .states[0].parameters[0].is_mutable,
    parameters: [
      .states[0].parameters[]
      | select(.is_self | not)
      | { name, type: (.type_reference | type_signature) }
    ],
    operations: [
      .states[0].statements[]
      | {
          kind,
          target,
          arguments: [.arguments[] | operand_signature]
        }
    ]
  };

.[0] as $typed
| .[1] as $contracts
| [
    $typed
    | ..
    | objects
    | select(.name? as $name | [
        "X2ApicTimer::arm_one_shot",
        "X2ApicTimer::complete_timer_acknowledgement",
        "X2ApicTimer::configure_one_shot_divide_by_16",
        "X2ApicTimer::stop"
      ] | index($name))
    | machine_signature
  ] | sort_by(.machine) as $actual_machines
| [
    {
      machine: "X2ApicTimer::arm_one_shot",
      state: "arm_one_shot",
      state_count: 1,
      self_is_mutable: true,
      parameters: [
        { name: "initial_count", type: { kind: "constrained", base: "u64", minimum: "1", maximum: "4294967295" } }
      ],
      operations: [
        {
          kind: "call",
          target: "asm#wrmsr",
          arguments: [
            { kind: "integer", text: "0x838" },
            { kind: "name", path: ["initial_count"] }
          ]
        }
      ]
    },
    {
      machine: "X2ApicTimer::complete_timer_acknowledgement",
      state: "complete_timer_acknowledgement",
      state_count: 1,
      self_is_mutable: true,
      parameters: [],
      operations: [
        {
          kind: "call",
          target: "asm#wrmsr",
          arguments: [
            { kind: "integer", text: "0x80b" },
            { kind: "integer", text: "0" }
          ]
        }
      ]
    },
    {
      machine: "X2ApicTimer::configure_one_shot_divide_by_16",
      state: "configure_one_shot_divide_by_16",
      state_count: 1,
      self_is_mutable: true,
      parameters: [
        { name: "vector", type: { kind: "constrained", base: "u64", minimum: "32", maximum: "255" } }
      ],
      operations: [
        {
          kind: "call",
          target: "asm#wrmsr",
          arguments: [
            { kind: "integer", text: "0x83e" },
            { kind: "integer", text: "3" }
          ]
        },
        {
          kind: "call",
          target: "asm#wrmsr",
          arguments: [
            { kind: "integer", text: "0x832" },
            { kind: "name", path: ["vector"] }
          ]
        }
      ]
    },
    {
      machine: "X2ApicTimer::stop",
      state: "stop",
      state_count: 1,
      self_is_mutable: true,
      parameters: [],
      operations: [
        {
          kind: "call",
          target: "asm#wrmsr",
          arguments: [
            { kind: "integer", text: "0x838" },
            { kind: "integer", text: "0" }
          ]
        }
      ]
    }
  ] as $expected_machines
| require($actual_machines == $expected_machines;
    "x2APIC provider inputs, leaf separation, or checked wrmsr sequence changed")
| [
    $contracts
    | ..
    | objects
    | select(.machine? as $name | [
        "X2ApicTimer::arm_one_shot",
        "X2ApicTimer::complete_timer_acknowledgement",
        "X2ApicTimer::configure_one_shot_divide_by_16",
        "X2ApicTimer::stop"
      ] | index($name))
  ] | sort_by(.machine) as $machine_contracts
| require(($machine_contracts | length) == 4;
    "expected exactly four x2APIC provider machine contracts")
| require(all($machine_contracts[];
      .contract.service_reach == {
        interface: "published_ceiling",
        services: ["MachineControl"]
      }
      and .implementation.checked_service_reach == ["MachineControl"]
      and .implementation.checked_may_suspend == false
      and .implementation.checked_may_block == false
      and .implementation.checked_synchronous_invocations == []
      and .implementation.checked_crash_sites == []
      and .implementation.checked_crash_calls == []
      and .implementation.checked_termination.kind == "terminates"
    );
    "x2APIC provider leaves lost exact MachineControl reach or gained calls, blocking, crashes, or nontermination")
| true
