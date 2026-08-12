def field($name):
  .fields[] | select(.name == $name) | .value;

def direct_assignment($receiver; $member):
  select(
    .kind == "assignment"
    and .target.kind == "member"
    and .target.member == $member
    and .target.receiver.kind == "member"
    and .target.receiver.member == $receiver
    and .target.receiver.receiver.kind == "name"
    and .target.receiver.receiver.path == ["output"]
  );

def nested_assignment($outer; $inner; $member):
  select(
    .kind == "assignment"
    and .target.kind == "member"
    and .target.member == $member
    and .target.receiver.kind == "member"
    and .target.receiver.member == $inner
    and .target.receiver.receiver.kind == "member"
    and .target.receiver.receiver.member == $outer
    and .target.receiver.receiver.receiver.kind == "name"
    and .target.receiver.receiver.receiver.path == ["output"]
  );

def one_value($statements; $receiver; $member):
  [$statements[] | direct_assignment($receiver; $member) | .value];

def one_nested_value($statements; $outer; $inner; $member):
  [$statements[] | nested_assignment($outer; $inner; $member) | .value];

def parameter_locations($statements):
  [
    $statements[]
    | select(
        .kind == "assignment"
        and .target.kind == "indexed"
        and .target.collection.kind == "member"
        and .target.collection.member == "locations"
        and .value.kind == "struct_literal"
        and .value.type_name == "ValueLocation"
      )
    | {
        parameter: (.target.collection.receiver.index.text | tonumber),
        location: (.target.index.text | tonumber),
        pointer: (.value | field("pointer") | field("register") | .path),
        has_copy: (.value | field("has_copy") | .value),
        copy_stack_byte_offset: (.value | field("copy_stack_byte_offset") | .text | tonumber),
        byte_size_source: (.value | field("byte_size") | {
          parameter: .receiver.index.path,
          member: .member
        }),
        alignment_source: (.value | field("alignment") | {
          parameter: .receiver.index.path,
          member: .member
        })
      }
  ];

def clobber_registers($statements):
  [
    $statements[]
    | select(
        .kind == "assignment"
        and .target.kind == "indexed"
        and .target.collection.kind == "member"
        and .target.collection.member == "registers"
        and .target.collection.receiver.kind == "member"
        and .target.collection.receiver.member == "ordinary_clobbers"
      )
    | {
        index: (.target.index.text | tonumber),
        register: .value.path
      }
  ];

[
  .. | objects | select(.name? == "UefiX86_64::plan")
] as $matches
| if ($matches | length) != 1 then
    error("expected exactly one typed UefiX86_64::plan machine")
  else
    $matches[0] as $machine
    | [$machine.states[] | select(.name == "build")] as $build_states
    | if ($build_states | length) != 1 then
        error("expected exactly one UefiX86_64::plan build state")
      else
        $build_states[0].statements as $statements
        | [
            {
              name: "Microsoft x64 convention",
              ok: (one_value($statements; "call"; "convention")
                == [{"kind":"name","path":["CallingConvention","MicrosoftX64"]}])
            },
            {
              name: "two semantic parameters",
              ok: (one_value($statements; "call"; "parameter_count")
                == [{"kind":"integer","text":"2"}])
            },
            {
              name: "RCX/RDX indirect copies and disjoint homes",
              ok: (parameter_locations($statements) == [
                {
                  parameter: 0,
                  location: 0,
                  pointer: ["MachineRegister", "X86Rcx"],
                  has_copy: true,
                  copy_stack_byte_offset: 32,
                  byte_size_source: {parameter: ["image"], member: "byte_size"},
                  alignment_source: {parameter: ["image"], member: "alignment"}
                },
                {
                  parameter: 1,
                  location: 0,
                  pointer: ["MachineRegister", "X86Rdx"],
                  has_copy: true,
                  copy_stack_byte_offset: 48,
                  byte_size_source: {parameter: ["initial_storage"], member: "byte_size"},
                  alignment_source: {parameter: ["initial_storage"], member: "alignment"}
                }
              ])
            },
            {
              name: "Microsoft x64 volatile register count",
              ok: (one_nested_value($statements; "call"; "ordinary_clobbers"; "register_count")
                == [{"kind":"integer","text":"7"}])
            },
            {
              name: "Microsoft x64 volatile register set",
              ok: (clobber_registers($statements) == [
                {index: 0, register: ["MachineRegister", "X86Rax"]},
                {index: 1, register: ["MachineRegister", "X86Rcx"]},
                {index: 2, register: ["MachineRegister", "X86Rdx"]},
                {index: 3, register: ["MachineRegister", "X86R8"]},
                {index: 4, register: ["MachineRegister", "X86R9"]},
                {index: 5, register: ["MachineRegister", "X86R10"]},
                {index: 6, register: ["MachineRegister", "X86R11"]}
              ])
            },
            {
              name: "16-byte stack alignment",
              ok: (one_value($statements; "call"; "stack_alignment")
                == [{"kind":"integer","text":"16"}])
            },
            {
              name: "32-byte shadow space",
              ok: (one_value($statements; "call"; "shadow_bytes")
                == [{"kind":"integer","text":"32"}])
            },
            {
              name: "ordinary call/return entry control",
              ok: (one_value($statements; "call"; "entry_control")
                == [{"kind":"name","path":["EntryControl","CallReturn"]}])
            },
            {
              name: "x86-64 long-mode entry",
              ok: (one_value($statements; "state"; "initial_regime")
                == [{"kind":"name","path":["MachineRegime","X86Long64"]}])
            },
            {
              name: "provider-selected entry stack",
              ok: (one_value($statements; "state"; "stack")
                == [{"kind":"name","path":["EntryStack","ProviderSelected"]}])
            },
            {
              name: "firmware-entry preemption classification",
              ok: (one_value($statements; "state"; "preemption")
                == [{"kind":"name","path":["Preemption","NotApplicable"]}])
            },
            {
              name: "general-register transitive use",
              ok: (one_nested_value($statements; "state"; "permitted_transitive_use"; "general_registers")
                == [{"kind":"boolean","value":true}])
            },
            {
              name: "flags transitive use",
              ok: (one_nested_value($statements; "state"; "permitted_transitive_use"; "flags")
                == [{"kind":"boolean","value":true}])
            }
          ]
        | map(select(.ok | not) | .name) as $failures
        | if ($failures | length) == 0 then
            true
          else
            error("UEFI policy mismatch: " + ($failures | join(", ")))
          end
      end
  end
