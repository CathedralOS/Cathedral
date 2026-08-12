def require($condition; $message):
  if $condition then . else error($message) end;

def field_value($literal; $name):
  first($literal.fields[] | select(.name == $name) | .value);

.[0] as $typed
| .[1] as $contracts
| [
    $typed
    | ..
    | objects
    | select(.name? == "local_apic_register_map_snapshot")
  ] as $machines
| [
    $contracts
    | ..
    | objects
    | select(.machine? == "local_apic_register_map_snapshot")
  ] as $machine_contracts
| require($machines | length == 1;
    "expected exactly one typed local_apic_register_map_snapshot machine")
| require($machine_contracts | length == 1;
    "expected exactly one local_apic_register_map_snapshot contract")
| $machines[0] as $machine
| $machine_contracts[0] as $machine_contract
| require(($machine.states | length) == 1 and
          $machine.states[0].name == "entry" and
          $machine.states[0].return_type == { kind: "named", name: "LocalApicRegisterMapSnapshot" } and
          ($machine.states[0].statements | length) == 1 and
          $machine.states[0].statements[0].kind == "expression" and
          $machine.states[0].statements[0].value.type_name == "LocalApicRegisterMapSnapshot";
    "local-APIC map snapshot no longer has one pure record-producing state")
| field_value($machine.states[0].statements[0].value; "pairs") as $pairs
| require($pairs.kind == "array_literal";
    "local-APIC register correspondence is no longer one fixed table")
| [
    $pairs.values[]
    | {
        xapic_offset: field_value(.; "xapic_offset").text,
        x2apic_msr: field_value(.; "x2apic_msr").text
      }
  ] as $actual_pairs
| [
    { xapic_offset: "0x20",  x2apic_msr: "0x802" },
    { xapic_offset: "0x30",  x2apic_msr: "0x803" },
    { xapic_offset: "0x80",  x2apic_msr: "0x808" },
    { xapic_offset: "0xb0",  x2apic_msr: "0x80b" },
    { xapic_offset: "0xf0",  x2apic_msr: "0x80f" },
    { xapic_offset: "0x320", x2apic_msr: "0x832" },
    { xapic_offset: "0x380", x2apic_msr: "0x838" },
    { xapic_offset: "0x390", x2apic_msr: "0x839" },
    { xapic_offset: "0x3e0", x2apic_msr: "0x83e" }
  ] as $expected_pairs
| require($actual_pairs == $expected_pairs;
    "xAPIC offsets no longer match their exact x2APIC MSR identities")
| require(($machine_contract.implementation.checked_may_suspend == false) and
          ($machine_contract.implementation.checked_may_block == false) and
          ($machine_contract.implementation.checked_service_reach == []) and
          ($machine_contract.implementation.checked_synchronous_invocations == []) and
          ($machine_contract.implementation.checked_crash_sites == []) and
          ($machine_contract.implementation.checked_crash_calls == []) and
          ($machine_contract.implementation.checked_termination.kind == "terminates") and
          ($machine_contract.implementation.inferred_write_frames | length) == 1 and
          ($machine_contract.implementation.inferred_write_frames[0].completeness == "complete") and
          ($machine_contract.implementation.inferred_write_frames[0].paths == []);
    "pure local-APIC map evaluation gained effects, writes, calls, crashes, or nontermination")
| true
