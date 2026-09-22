"""Independent explicit-number String oracles and complete Store/Frame pairs.

Primary ACPI 6.6 19.6.139-140 and 19.3.5.5 govern source kinds and direct
target replacement. Prefix/padding follows the existing numeric-string PORT.
No generated data type duplicates an execution or pipeline type name.
"""
import importlib.util
import json
import re
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location(
    "numeric_string_bridge_mid_fixtures", HERE.parent / "mid-execution/fixtures.py"
)
mid = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mid)
ns = mid.ns
MAX = (1 << 64) - 1
FORMATS = ("Decimal", "Hexadecimal")


def formatted(kind, value, bits, format):
    if kind in ("inline", "integer"):
        value &= (1 << bits) - 1
        return (str(value) if format == "Decimal" else f"0x{value:X}").encode("ascii")
    if kind == "string":
        return value
    return ",".join(str(byte) if format == "Decimal" else f"0x{byte:02X}"
                    for byte in value).encode("ascii")


def owned(slot, raw, string=False, store="store", fresh=False):
    kind, prefix = ("String", "string") if string else ("Buffer", "buffer")
    value = f"Value::{kind} {{{prefix}_storage:{kind}Storage::Owned {{{prefix}_owner:{slot}}}}}"
    result = (f"{store}.space.objects[{slot}]=Object {{value:{value}}};" if fresh
              else f"{store}.space.objects[{slot}].value={value};")
    result += f"{store}.bytes.blocks[{slot}]=ByteBlock {{initialized:true,length:{len(raw)}}};"
    return result + "".join(f"{store}.bytes.blocks[{slot}].bytes[{index}]={byte};"
                            for index, byte in enumerate(raw) if byte)


def _source(kind, value):
    if kind == "inline":
        return f"Operand::Integer {{number:{value}}}", ""
    if kind == "integer":
        setup = f"store.space.objects[0].value=Value::Integer {{number:{value}}};"
    else:
        setup = owned(0, value, kind == "string")
    return "Operand::Object {object_id:0}", setup


def cases():
    rows = []

    def add(name, kind="integer", value=42, bits=64, format="Decimal", setup="",
            frame_setup="", target="Target::Null", operand=None, error=None,
            expected="", result_id=3, parent=0, slot=0, no_parent=False):
        source, initial = _source(kind, value)
        raw = formatted(kind, value, bits, format)
        if error is None:
            error = ("Capacity" if kind in ("buffer", "string") and len(value) > 256
                     else "Encoding" if kind == "string" and any(b == 0 or b >= 128 for b in value)
                     else "Capacity" if len(raw) > 256 else "Success")
        if error == "Success":
            expected = (f"expected_store.space.object_count={result_id + 1};"
                        + owned(result_id, raw, True, "expected_store", True) + expected)
            if not no_parent:
                expected += (f"expected_frame.operations[{parent}].count={slot + 1};"
                             f"expected_frame.operations[{parent}].operands[{slot}]=Operand::Object {{object_id:{result_id}}};")
        else:
            if expected:
                raise ValueError("failed retirement must expect complete rollback")
            raw = b""
        # Poison future slots as well as the inherited inactive byte/cache tails.
        poison = ("store.space.objects[3]=Object {value:Value::Integer {number:91},has_next:true,next:62};"
                  "store.bytes.blocks[3]=ByteBlock {initialized:true,length:256};store.bytes.blocks[3].bytes[255]=201;"
                  "store.space.objects[4]=Object {value:Value::Integer {number:92},has_next:true,next:61};"
                  "store.bytes.blocks[4]=ByteBlock {initialized:true,length:255};store.bytes.blocks[4].bytes[254]=202;")
        rows.append(dict(name=name, bits=bits, format=format, setup=poison + initial + setup,
                         frame_setup=frame_setup, target=target, operand=operand or source,
                         expected=expected, error=error, scalar=False, copy=False,
                         result_id=result_id, bytes=raw.hex(), parent=parent, slot=slot,
                         no_parent=no_parent))

    for bits in (32, 64):
        for format in FORMATS:
            tag = f"{format.lower()}_{bits}"
            for number in (0, MAX, 1 << 32):
                add(f"inline_{number}_{tag}", kind="inline", value=number, bits=bits, format=format)
            add("named_integer_" + tag, value=MAX, bits=bits, format=format)
            add("string_identity_" + tag, kind="string", value=b"not-a-number,0xAb", bits=bits, format=format)
            add("buffer_each_byte_" + tag, kind="buffer", value=bytes([0, 9, 10, 15, 16, 99, 100, 255]), bits=bits, format=format)

    for format in FORMATS:
        tag = format.lower()
        for length in (0, 255, 256):
            add(f"string_{length}_{tag}", kind="string", value=b"Z" * length, format=format)
        add("buffer_empty_" + tag, kind="buffer", value=b"", format=format)
        add("buffer_256_" + tag, kind="buffer", value=bytes(256), format=format)
        add("source_padding_" + tag, kind="buffer", value=b"2A\0\0\0", format=format,
            setup="store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:5,buffer_initializer:Span {unit:7,end:2}}};")
        add("source_initializer_dominates_" + tag, kind="buffer", value=b"2A", format=format,
            setup="store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:1,buffer_initializer:Span {unit:7,end:2}}};")
        add("source_string_" + tag, kind="string", value=b"2A", format=format,
            setup="store.space.objects[0].value=Value::String {string_storage:StringStorage::Source {string_source:Span {unit:7,end:2}}};")
        add("dirty_source_tail_" + tag, kind="string", value=b"AB", format=format,
            setup="store.bytes.blocks[0].bytes[255]=255;")
    for label, raw in (("255", bytes([255])*64), ("256", bytes([255])*63+b"\0\x0a"),
                       ("259", bytes([255])*65), ("zero_128", bytes(128)), ("zero_129", bytes(129))):
        add("decimal_expansion_" + label, kind="buffer", value=raw)
    for length in (51, 52):
        add(f"hex_expansion_{length}", kind="buffer", value=bytes([255])*length, format="Hexadecimal")

    for format in FORMATS:
        tag = format.lower()
        raw = formatted("integer", 42, 64, format)
        for label, destination in (
            ("integer", "Value::Integer {number:7}"), ("package", "Value::Package {}"),
            ("reference", "Value::Reference {kind:ReferenceKind::RefOf,object_id:2}"),
            ("uninitialized", "Value::Uninitialized"),
        ):
            add("named_" + label + "_" + tag, format=format, target="Target::Named {object_id:1}",
                setup=f"store.space.objects[1].value={destination};",
                expected=owned(1, raw, True, "expected_store"))
        for string in (False, True):
            add("named_" + ("string" if string else "buffer") + "_" + tag, format=format,
                target="Target::Named {object_id:1}", setup=owned(1, b"old longer", string)+"store.bytes.blocks[1].bytes[255]=255;",
                expected=owned(1, raw, True, "expected_store"))
        add("self_target_" + tag, format=format, target="Target::Named {object_id:0}",
            expected=owned(0, raw, True, "expected_store"))
        for label, binding in (("uninitialized", "Binding::Uninitialized"),
                               ("integer", "Binding::Integer {number:97}"),
                               ("shared", "Binding::Shared {shared_id:2}"),
                               ("cell", "Binding::Cell {cell_id:2}")):
            for local in (True, False):
                family, index_name = ("locals", "local_index") if local else ("arguments", "argument_index")
                kind = "Local" if local else "Argument"
                fresh = label != "cell"
                slot = 4 if fresh else 2
                extra = owned(slot, raw, True, "expected_store", fresh)
                if fresh:
                    extra = f"expected_store.space.object_count=5;expected_frame.{family}[0]=Binding::Cell {{cell_id:4}};" + extra
                add(f"{kind.lower()}_{label}_{tag}", format=format, target=f"Target::{kind} {{{index_name}:0}}",
                    frame_setup=f"frame.{family}[0]={binding};", expected=extra)
        for reference in ("RefOf", "Index"):
            for binding in ("Shared", "Cell"):
                field = "shared_id" if binding == "Shared" else "cell_id"
                add(f"arg_{reference.lower()}_{binding.lower()}_{tag}", format=format,
                    target="Target::Argument {argument_index:0}",
                    setup=f"store.space.objects[2].value=Value::Reference {{kind:ReferenceKind::{reference},object_id:1}};",
                    frame_setup=f"frame.arguments[0]=Binding::{binding} {{{field}:2}};",
                    expected=owned(1, raw, True, "expected_store"))
        add("debug_" + tag, format=format, target="Target::Debug", error="UnresolvedService")

    raw = b"42"
    for index in range(1, 7):
        add(f"argument_index_{index}", target=f"Target::Argument {{argument_index:{index}}}",
            frame_setup=f"frame.arguments[{index}]=Binding::Integer {{number:17}};",
            expected=f"expected_store.space.object_count=5;expected_frame.arguments[{index}]=Binding::Cell {{cell_id:4}};"+owned(4, raw, True, "expected_store", True))
    add("local_index_7", target="Target::Local {local_index:7}",
        expected="expected_store.space.object_count=5;expected_frame.locals[7]=Binding::Cell {cell_id:4};"+owned(4, raw, True, "expected_store", True))
    add("local_shared_max_is_replaced", target="Target::Local {local_index:0}",
        frame_setup=f"frame.locals[0]=Binding::Shared {{shared_id:{MAX}}};",
        expected="expected_store.space.object_count=5;expected_frame.locals[0]=Binding::Cell {cell_id:4};"+owned(4, raw, True, "expected_store", True))

    for kind in ("Named", "Local", "Arg"):
        add("transparent_"+kind.lower(), kind="string", value=b"literal!",
            setup=owned(2, b"literal!", True)+f"store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:2}};")
    add("transparent_chain", setup="store.space.objects[0].value=Value::Reference {kind:ReferenceKind::Named,object_id:1};store.space.objects[1].value=Value::Reference {kind:ReferenceKind::Arg,object_id:2};store.space.objects[2].value=Value::Integer {number:42};")
    for kind in ("RefOf", "Index"):
        add("source_"+kind.lower()+"_opaque", setup=f"store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:{MAX}}};", error="UnsupportedValue")
    add("source_self_cycle", setup="store.space.objects[0].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};", error="ReferenceCycle")
    add("source_two_cycle", setup="store.space.objects[0].value=Value::Reference {kind:ReferenceKind::Local,object_id:1};store.space.objects[1].value=Value::Reference {kind:ReferenceKind::Arg,object_id:0};", error="ReferenceCycle")
    add("source_reference_max", setup=f"store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::Named,object_id:{MAX}}};", error="InvalidState")
    add("uninitialized_operand", operand="Operand::Uninitialized", error="Uninitialized")
    add("uninitialized_object", setup="store.space.objects[0].value=Value::Uninitialized;", error="Uninitialized")
    for kind in ("Package", "Method", "FieldUnit", "OperationRegion", "BufferField", "NameReference"):
        add("unsupported_"+kind.lower(), setup=f"store.space.objects[0].value=Value::{kind} {{}};", error="UnsupportedValue")

    for name, setup, error in (
        ("owner_max", f"store.space.objects[0].value=Value::String {{string_storage:StringStorage::Owned {{string_owner:{MAX}}}}};", "InvalidState"),
        ("owner_other", "store.space.objects[0].value=Value::String {string_storage:StringStorage::Owned {string_owner:1}};", "InvalidState"),
        ("not_initialized", "store.bytes.blocks[0].initialized=false;", "InvalidState"),
        ("length_max", f"store.bytes.blocks[0].length={MAX};", "Capacity"),
        ("length_257", "store.bytes.blocks[0].length=257;", "Capacity"),
        ("late_encoding", owned(0, b"Z"*256, True)+"store.bytes.blocks[0].bytes[255]=255;", "Encoding"),
        ("embedded_nul", "store.bytes.blocks[0].bytes[1]=0;", "Encoding"),
        ("source_unit", "store.space.objects[0].value=Value::String {string_storage:StringStorage::Source {string_source:Span {unit:8,end:2}}};", "Bounds"),
        ("source_reversed", "store.space.objects[0].value=Value::String {string_storage:StringStorage::Source {string_source:Span {unit:7,start:2,end:1}}};", "Bounds"),
        ("source_end_max", f"store.space.objects[0].value=Value::String {{string_storage:StringStorage::Source {{string_source:Span {{unit:7,end:{MAX}}}}}}};", "Bounds"),
        ("declared_max", f"store.space.objects[0].value=Value::Buffer {{buffer_storage:BufferStorage::Source {{declared_size:{MAX},buffer_initializer:Span {{unit:7,end:2}}}}}};", "Capacity"),
    ):
        add(name, kind="string", value=b"AB", setup=setup, error=error)
    for kind, value in (("inline", MAX), ("integer", MAX), ("string", b"AB"), ("buffer", b"\0\xff")):
        add(kind+"_ignores_source_metadata", kind=kind, value=value,
            frame_setup=f"frame.source_length={MAX};frame.body.unit={MAX};")
    add("source_length_max", kind="string", value=b"AB", frame_setup=f"frame.source_length={MAX};",
        setup="store.space.objects[0].value=Value::String {string_storage:StringStorage::Source {string_source:Span {unit:7,end:2}}};", error="Capacity")
    add("source_bad_unit_before_declared_capacity", kind="buffer", value=b"AB",
        setup=f"store.space.objects[0].value=Value::Buffer {{buffer_storage:BufferStorage::Source {{declared_size:{MAX},buffer_initializer:Span {{unit:8,end:2}}}}}};", error="Bounds")

    for name, target, frame_setup, setup, error in (
        ("named_fresh", "Target::Named {object_id:3}", "", "", "MissingObject"),
        ("named_max", f"Target::Named {{object_id:{MAX}}}", "", "", "MissingObject"),
        ("cell_fresh", "Target::Local {local_index:0}", "frame.locals[0]=Binding::Cell {cell_id:3};", "", "MissingObject"),
        ("cell_max", "Target::Local {local_index:0}", f"frame.locals[0]=Binding::Cell {{cell_id:{MAX}}};", "", "MissingObject"),
        ("argument_fresh", "Target::Argument {argument_index:0}", "frame.arguments[0]=Binding::Shared {shared_id:3};", "", "InvalidState"),
        ("argument_max", "Target::Argument {argument_index:0}", f"frame.arguments[0]=Binding::Shared {{shared_id:{MAX}}};", "", "InvalidState"),
        ("argument_referent_fresh", "Target::Argument {argument_index:0}", "frame.arguments[0]=Binding::Shared {shared_id:2};", "store.space.objects[2].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:3};", "MissingObject"),
        ("local_index_max", f"Target::Local {{local_index:{MAX}}}", "", "", "InvalidTarget"),
        ("argument_index_max", f"Target::Argument {{argument_index:{MAX}}}", "", "", "InvalidTarget"),
        ("local_index_8", "Target::Local {local_index:8}", "", "", "InvalidTarget"),
        ("argument_index_7", "Target::Argument {argument_index:7}", "", "", "InvalidTarget"),
        ("target_owner", "Target::Named {object_id:1}", "", "store.space.objects[1].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:2}};", "InvalidState"),
        ("target_encoding", "Target::Named {object_id:1}", "", owned(1, b"\xff", True), "InvalidState"),
        ("target_field", "Target::Named {object_id:1}", "", "store.space.objects[1].value=Value::FieldUnit {};", "UnresolvedRegion"),
        ("target_region", "Target::Named {object_id:1}", "", "store.space.objects[1].value=Value::OperationRegion {};", "UnresolvedRegion"),
        ("target_method", "Target::Named {object_id:1}", "", "store.space.objects[1].value=Value::Method {};", "UnsupportedValue"),
    ):
        add(name, target=target, frame_setup=frame_setup, setup=setup, error=error)
    add("encoding_precedes_target", kind="string", value=b"A\xff", target=f"Target::Named {{object_id:{MAX}}}")
    add("format_capacity_precedes_debug", kind="buffer", value=bytes([255])*65, target="Target::Debug")
    add("target_precedes_arena", setup="store.space.object_count=64;", target="Target::Named {object_id:64}", error="MissingObject")
    add("arena_precedes_late_kind", setup="store.space.object_count=64;store.space.objects[1].value=Value::Method {};", target="Target::Named {object_id:1}", error="Capacity")
    add("full_arena", setup="store.space.object_count=64;", error="Capacity")
    add("count_max", setup=f"store.space.object_count={MAX};", error="InvalidState")
    add("inline_count_max", kind="inline", setup=f"store.space.object_count={MAX};", error="Capacity")
    add("namespace_max", setup=f"store.space.count={MAX};", error="Capacity")
    add("operand_id_max", operand=f"Operand::Object {{object_id:{MAX}}}", error="InvalidState")
    add("last_slot_null", setup="store.space.object_count=63;", result_id=63)
    add("last_slot_named", setup="store.space.object_count=63;", result_id=63, target="Target::Named {object_id:1}", expected=owned(1, b"42", True, "expected_store"))
    add("last_slot_cell", setup="store.space.object_count=63;", result_id=63, target="Target::Local {local_index:0}",
        frame_setup="frame.locals[0]=Binding::Cell {cell_id:2};", expected=owned(2, b"42", True, "expected_store"))
    add("last_slot_local_rollback", setup="store.space.object_count=63;", target="Target::Local {local_index:0}", error="Capacity")
    add("last_slot_argument_rollback", setup="store.space.object_count=63;", target="Target::Argument {argument_index:0}", error="Capacity")
    for target, label in (("Target::Null", "null"), ("Target::Named {object_id:1}", "named"), ("Target::Local {local_index:0}", "local")):
        add("parent_full_"+label, target=target,
            frame_setup="frame.operations[0].count=1;frame.operations[0].operands[0]=Operand::Integer {number:97};", error="InvalidState")
    add("parent_count_max", frame_setup=f"frame.operation_count={MAX};", error="InvalidState")
    add("operand_count_max", frame_setup=f"frame.operations[0].count={MAX};", error="InvalidState")
    add("parent_zero_arity", frame_setup="frame.operations[0].value_arity=0;", error="InvalidState")
    add("parent_arity_max", frame_setup=f"frame.operations[0].value_arity={MAX};")
    add("parent_last_slot", parent=15, slot=6, frame_setup="frame.operation_count=16;frame.operations[15]=Operation {opcode:0xa4,value_arity:7,count:6};")
    add("parent_slot_bound", frame_setup=f"frame.operations[0].value_arity={MAX};frame.operations[0].count=7;", error="InvalidState")
    add("no_parent", no_parent=True, frame_setup="frame.operation_count=0;")
    if len({row["name"] for row in rows}) != len(rows):
        raise ValueError("duplicate numeric-string bridge case")
    return rows


def _body_span(source, name):
    start = source.index("machine " + name + "(")
    opened = source.index("{", start)
    depth = 0
    for at in range(opened, len(source)):
        if source[at] == "{": depth += 1
        elif source[at] == "}":
            depth -= 1
            if depth == 0: return start, at + 1
    raise ValueError("unterminated fixture machine")


def render(selected):
    selected = list(selected)
    if not selected:
        raise ValueError("select at least one retirement pair")
    source, names = ns.render_bridge(selected)
    source += "\nuse execution::numeric_string_execution::retire_numeric_string;\n"
    source += "use integer_helpers::string_numbers::NumberFormat;\n"
    for row in selected:
        old = f'write_generic_target(&input,&mut store,&mut frame,{row["target"]},{row["operand"]},false)'
        new = f'retire_numeric_string(&input,&mut store,&mut frame,{row["operand"]},{row["target"]},NumberFormat::{row["format"]})'
        for control in (False, True):
            name = "Suite::" + row["name"] + ("_control" if control else "_positive")
            start, end = _body_span(source, name)
            body = source[start:end]
            if body.count(old) != 1 or body.count("let mut expected_store:ObjectStore=") != 1:
                raise ValueError("borrowed bridge invocation/snapshot changed")
            setup = "frame.operation_count=1;frame.operations[0]=Operation {opcode:0xa4,value_arity:1};" + row["frame_setup"]
            body = body.replace(old, new).replace("let mut expected_store:ObjectStore=", setup+"let mut expected_store:ObjectStore=")
            if control:
                needle = "expected_frame.lookup_cache[15].object_id=72;"
                if body.count(needle) != 1:
                    raise ValueError("borrowed bridge control changed")
                choice = sum(row["name"].encode()) % 4
                if choice == 0 and row["error"] == "Success":
                    raw = bytes.fromhex(row["bytes"])
                    byte = raw[255] if len(raw) == 256 else 0
                    mutation = f'expected_store.bytes.blocks[{row["result_id"]}].bytes[255]={byte ^ 1};'
                elif choice == 1 and row["error"] == "Success" and not row["no_parent"]:
                    mutation = f'expected_frame.operations[{row["parent"]}].operands[{row["slot"]}]=Operand::Object {{object_id:{MAX}}};'
                elif choice == 2:
                    mutation = "expected_store.bytes.blocks[62].bytes[255]=1;"
                elif choice == 3:
                    mutation = ""
                    wanted = "InvalidState" if row["error"] == "Success" else "Success"
                    body = body.replace(f'transition outcome==ExecutionOutcome::{row["error"]} &&', f'transition outcome==ExecutionOutcome::{wanted} &&')
                else:
                    mutation = needle
                body = body.replace(needle, mutation)
            source = source[:start] + body + source[end:]
    if any(int(number) > MAX for number in re.findall(r"(?<![A-Za-z_])\b\d+\b", source)):
        raise ValueError("generated integer exceeds u64")
    return source, names


if __name__ == "__main__":
    rows = cases()
    source, names = render(rows)
    (HERE / "bridge-cases.json").write_text(json.dumps(rows, indent=2) + "\n")
    print(len(rows), "authored complete Store/Frame pairs; execution not claimed")
