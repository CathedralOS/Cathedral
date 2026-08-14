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

def placement_signature($placement):
  if any($placement.fields[]; .name == "offset") then
    {
      kind: "at",
      offset: (field_value($placement; "offset").text | tonumber)
    }
  else
    {
      kind: "bits",
      container: (field_value($placement; "container").text | tonumber),
      container_width: (field_value($placement; "container_width").text | tonumber),
      destination_lsb: (field_value($placement; "destination_lsb").text | tonumber),
      source_lsb: (field_value($placement; "source_lsb").text | tonumber),
      width: (field_value($placement; "width").text | tonumber)
    }
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
        placement: placement_signature($placement)
      }
  ];

.[0] as $typed
| .[1] as $contracts
| [
    $typed
    | ..
    | objects
    | select(.name? == "X86IdtGate" and .supply? == "checked_shape")
  ] as $schemas
| [
    $typed
    | ..
    | objects
    | select(.name? == "X86IdtGateLayout::plan")
  ] as $plans
| [
    $contracts
    | ..
    | objects
    | select(.machine? == "X86IdtGateLayout::plan")
  ] as $machine_contracts
| require($schemas | length == 1;
    "expected exactly one checked X86IdtGate schema")
| require($plans | length == 1;
    "expected exactly one typed X86IdtGateLayout::plan machine")
| require($machine_contracts | length == 1;
    "expected exactly one X86IdtGateLayout::plan contract")
| $schemas[0] as $schema
| $plans[0] as $plan
| $machine_contracts[0] as $machine_contract
| [
    $schema.members[]
    | { name, relevance, type: (.type_reference | type_signature) }
  ] as $actual_schema
| [
    { name: "entry",           relevance: "relevant", type: { kind: "named", name: "u64" } },
    { name: "selector",        relevance: "relevant", type: { kind: "named", name: "u16" } },
    { name: "ist",             relevance: "relevant", type: { kind: "constrained", base: "u8",  minimum: "0", maximum: "7" } },
    { name: "type_attributes", relevance: "relevant", type: { kind: "named", name: "u8" } },
    { name: "reserved",        relevance: "relevant", type: { kind: "constrained", base: "u32", minimum: "0", maximum: "0" } }
  ] as $expected_schema
| require($actual_schema == $expected_schema;
    "x86 IDT gate schema or field constraints changed")
| placement_rows($plan) as $actual_placements
| [
    {
      index: 0,
      key_index: 0,
      placement: { kind: "bits", container: 0, container_width: 16, destination_lsb: 0, source_lsb: 0, width: 16 }
    },
    { index: 1, key_index: 1, placement: { kind: "at", offset: 2 } },
    { index: 2, key_index: 2, placement: { kind: "at", offset: 4 } },
    { index: 3, key_index: 3, placement: { kind: "at", offset: 5 } },
    {
      index: 4,
      key_index: 0,
      placement: { kind: "bits", container: 6, container_width: 16, destination_lsb: 0, source_lsb: 16, width: 16 }
    },
    {
      index: 5,
      key_index: 0,
      placement: { kind: "bits", container: 8, container_width: 32, destination_lsb: 0, source_lsb: 32, width: 32 }
    },
    { index: 6, key_index: 4, placement: { kind: "at", offset: 12 } }
  ] as $expected_placements
| require($actual_placements == $expected_placements;
    "x86 IDT gate no longer has the exact 16-byte field geometry and entry fragmentation")
| first(
    $plan.states[0].statements[]
    | select(.kind == "expression" and .value.type_name? == "Plan")
    | .value
  ) as $result
| require((field_value($result; "entry_count").text == "7") and
          (field_value($result; "size_fixed").text == "16") and
          (field_value($result; "size_is_dynamic").value == false) and
          (field_value($result; "align").text == "16");
    "x86 IDT gate layout must remain fixed-size, 16-byte, and 16-aligned")
| require(($machine_contract.implementation.checked_may_suspend == false) and
          ($machine_contract.implementation.checked_may_block == false) and
          ($machine_contract.implementation.checked_service_reach == []) and
          ($machine_contract.implementation.checked_synchronous_invocations == []) and
          (($machine_contract.implementation.inferred_write_frames
            | map({state, completeness, paths})) == [{
              state: "plan",
              completeness: "complete",
              paths: ["self.entries"]
            }]) and
          ($machine_contract.implementation.checked_crash_sites == []) and
          ($machine_contract.implementation.checked_crash_calls == []) and
          ($machine_contract.implementation.checked_termination.kind == "terminates");
    "IDT layout construction gained effects, calls, crashes, nontermination, or writes outside its entry buffer")
| true
