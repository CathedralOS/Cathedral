def require($condition; $message):
  if $condition then . else error($message) end;

.[0] as $typed
| .[1] as $contracts
| [
    $typed
    | ..
    | objects
    | select(.name? == "uart_16550_fact_snapshot")
  ] as $machines
| [
    $contracts
    | ..
    | objects
    | select(.machine? == "uart_16550_fact_snapshot")
  ] as $machine_contracts
| require($machines | length == 1;
    "expected exactly one typed uart_16550_fact_snapshot machine")
| require($machine_contracts | length == 1;
    "expected exactly one uart_16550_fact_snapshot contract")
| $machines[0] as $machine
| $machine_contracts[0] as $machine_contract
| require(($machine.states | length) == 1 and
          $machine.states[0].name == "entry" and
          $machine.states[0].return_type == { kind: "named", name: "Uart16550FactSnapshot" } and
          ($machine.states[0].statements | length) == 1 and
          $machine.states[0].statements[0].kind == "expression" and
          $machine.states[0].statements[0].value.type_name == "Uart16550FactSnapshot";
    "UART fact snapshot no longer has one pure record-producing state")
| [
    $machine.states[0].statements[0].value.fields[]
    | { name, value: .value.text }
  ] as $actual_facts
| [
    { name: "com1",                    value: "0x3f8" },
    { name: "data_offset",             value: "0" },
    { name: "interrupt_enable_offset", value: "1" },
    { name: "divisor_low_offset",      value: "0" },
    { name: "divisor_high_offset",     value: "1" },
    { name: "fifo_control_offset",     value: "2" },
    { name: "line_control_offset",     value: "3" },
    { name: "modem_control_offset",    value: "4" },
    { name: "line_status_offset",      value: "5" },
    { name: "scratch_offset",          value: "7" },
    { name: "lcr_dlab",                value: "0x80" },
    { name: "lcr_8n1",                 value: "0x3" },
    { name: "lsr_data_ready",          value: "0x1" },
    { name: "lsr_thr_empty",           value: "0x20" },
    { name: "fcr_enable_clear",        value: "0xc7" },
    { name: "mcr_dtr_rts_out2",        value: "0xb" }
  ] as $expected_facts
| require($actual_facts == $expected_facts;
    "UART COM1 register map, banking aliases, or control/status masks changed")
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
    "pure UART fact evaluation gained effects, writes, calls, crashes, or nontermination")
| true
