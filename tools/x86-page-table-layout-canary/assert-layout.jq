def require($condition; $message):
  if $condition then . else error($message) end;

def field_value($literal; $name):
  first($literal.fields[] | select(.name == $name) | .value);

def type_signature:
  if .kind == "named" then
    { kind, name }
  elif .kind == "constrained" then
    {
      kind,
      base: .base_type.name,
      minimum: .constraints[0].minimum.text,
      maximum: .constraints[0].maximum.text
    }
  else
    { kind }
  end;

def placement_rows($plan):
  [
    $plan.states[0].statements[]
    | select(.kind == "assignment")
    | . as $statement
    | field_value($statement.value; "key") as $key
    | field_value($statement.value; "placement") as $placement
    | {
        index: ($statement.target.index.text | tonumber),
        key_index: ($key.receiver.index.text | tonumber),
        container: (field_value($placement; "container").text | tonumber),
        container_width: (field_value($placement; "container_width").text | tonumber),
        destination_lsb: (field_value($placement; "destination_lsb").text | tonumber),
        source_lsb: (field_value($placement; "source_lsb").text | tonumber),
        width: (field_value($placement; "width").text | tonumber)
      }
  ];

.[0] as $typed
| .[1] as $contracts
| [
    $typed
    | ..
    | objects
    | select(.name? == "X86PageTableEntry" and .supply? == "checked_shape")
  ] as $schemas
| [
    $typed
    | ..
    | objects
    | select(.name? == "X86PageTableEntryLayout::plan")
  ] as $plans
| [
    $contracts
    | ..
    | objects
    | select(.machine? == "X86PageTableEntryLayout::plan")
  ] as $machine_contracts
| require($schemas | length == 1;
    "expected exactly one checked X86PageTableEntry schema")
| require($plans | length == 1;
    "expected exactly one typed X86PageTableEntryLayout::plan machine")
| require($machine_contracts | length == 1;
    "expected exactly one X86PageTableEntryLayout::plan contract")
| $schemas[0] as $schema
| $plans[0] as $plan
| $machine_contracts[0] as $machine_contract
| [
    $schema.members[]
    | { name, type: (.type_reference | type_signature) }
  ] as $actual_schema
| [
    { name: "present",          type: { kind: "named", name: "bool" } },
    { name: "writable",         type: { kind: "named", name: "bool" } },
    { name: "user",             type: { kind: "named", name: "bool" } },
    { name: "write_through",    type: { kind: "named", name: "bool" } },
    { name: "cache_disable",    type: { kind: "named", name: "bool" } },
    { name: "accessed",         type: { kind: "named", name: "bool" } },
    { name: "dirty",            type: { kind: "named", name: "bool" } },
    { name: "page_size_or_pat", type: { kind: "named", name: "bool" } },
    { name: "global",           type: { kind: "named", name: "bool" } },
    { name: "software_low",     type: { kind: "constrained", base: "u8",  minimum: "0", maximum: "7" } },
    { name: "frame_number",     type: { kind: "constrained", base: "u64", minimum: "0", maximum: "1099511627775" } },
    { name: "software_high",    type: { kind: "constrained", base: "u8",  minimum: "0", maximum: "127" } },
    { name: "protection_key",   type: { kind: "constrained", base: "u8",  minimum: "0", maximum: "15" } },
    { name: "no_execute",       type: { kind: "named", name: "bool" } }
  ] as $expected_schema
| require($actual_schema == $expected_schema;
    "x86 page-table entry schema or field constraints changed")
| placement_rows($plan) as $actual_placements
| [
    { index: 0,  key_index: 0,  container: 0, container_width: 64, destination_lsb: 0,  source_lsb: 0, width: 1 },
    { index: 1,  key_index: 1,  container: 0, container_width: 64, destination_lsb: 1,  source_lsb: 0, width: 1 },
    { index: 2,  key_index: 2,  container: 0, container_width: 64, destination_lsb: 2,  source_lsb: 0, width: 1 },
    { index: 3,  key_index: 3,  container: 0, container_width: 64, destination_lsb: 3,  source_lsb: 0, width: 1 },
    { index: 4,  key_index: 4,  container: 0, container_width: 64, destination_lsb: 4,  source_lsb: 0, width: 1 },
    { index: 5,  key_index: 5,  container: 0, container_width: 64, destination_lsb: 5,  source_lsb: 0, width: 1 },
    { index: 6,  key_index: 6,  container: 0, container_width: 64, destination_lsb: 6,  source_lsb: 0, width: 1 },
    { index: 7,  key_index: 7,  container: 0, container_width: 64, destination_lsb: 7,  source_lsb: 0, width: 1 },
    { index: 8,  key_index: 8,  container: 0, container_width: 64, destination_lsb: 8,  source_lsb: 0, width: 1 },
    { index: 9,  key_index: 9,  container: 0, container_width: 64, destination_lsb: 9,  source_lsb: 0, width: 3 },
    { index: 10, key_index: 10, container: 0, container_width: 64, destination_lsb: 12, source_lsb: 0, width: 40 },
    { index: 11, key_index: 11, container: 0, container_width: 64, destination_lsb: 52, source_lsb: 0, width: 7 },
    { index: 12, key_index: 12, container: 0, container_width: 64, destination_lsb: 59, source_lsb: 0, width: 4 },
    { index: 13, key_index: 13, container: 0, container_width: 64, destination_lsb: 63, source_lsb: 0, width: 1 }
  ] as $expected_placements
| require($actual_placements == $expected_placements;
    "x86 page-table entry placements no longer tile the expected 64-bit word")
| first(
    $plan.states[0].statements[]
    | select(.kind == "expression" and .value.type_name? == "Plan")
    | .value
  ) as $result
| require((field_value($result; "entry_count").text == "14") and
          (field_value($result; "size_fixed").text == "8") and
          (field_value($result; "size_is_dynamic").value == false) and
          (field_value($result; "align").text == "8");
    "x86 page-table layout must remain fixed-size, 8-byte, and 8-aligned")
| require(($machine_contract.implementation.checked_may_suspend == false) and
          ($machine_contract.implementation.checked_may_block == false) and
          ($machine_contract.implementation.checked_service_reach == []) and
          ($machine_contract.implementation.checked_synchronous_invocations == []) and
          ($machine_contract.implementation.checked_crash_sites == []) and
          ($machine_contract.implementation.checked_crash_calls == []) and
          ($machine_contract.implementation.checked_termination.kind == "terminates");
    "pure x86 layout construction gained effects, calls, crashes, or nontermination")
| true
