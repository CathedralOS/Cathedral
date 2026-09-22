"""Independent logical-result oracles and complete Store/Frame retirement pairs."""
import importlib.util
import json
import operator
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location(
    "logical_bridge_mid_fixtures", HERE.parent / "mid-execution" / "fixtures.py"
)
mid = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mid)
ns = mid.ns
MAX = (1 << 64) - 1
RELATIONS = {
    0x93: ("eq", operator.eq), 0x94: ("gt", operator.gt),
    0x95: ("lt", operator.lt), 0x9293: ("ne", operator.ne),
    0x9294: ("le", operator.le), 0x9295: ("ge", operator.ge),
}


class Rejected(Exception):
    pass


def checked_bytes(kind, value):
    if len(value) > 256:
        raise Rejected("Capacity")
    if kind == "string" and any(byte == 0 or byte >= 128 for byte in value):
        raise Rejected("Encoding")
    return value


def number(kind, value, bits):
    if kind in ("inline", "integer"):
        return value & ((1 << bits) - 1)
    raw = checked_bytes(kind, value)
    if not raw:
        raise Rejected("Empty")
    if kind == "buffer":
        return int.from_bytes(raw[:bits // 8], "little")
    digits = []
    for byte in raw[:bits // 4]:
        if chr(byte) not in "0123456789abcdefABCDEF":
            break
        digits.append(chr(byte))
    return int("".join(digits), 16) if digits else 0


def converted(specification, target, bits):
    kind, value = specification
    if target == "integer":
        return number(kind, value, bits)
    if kind in ("inline", "integer"):
        value = number(kind, value, bits)
        return (value.to_bytes(bits // 8, "little") if target == "buffer"
                else f"{value:0{bits // 4}X}".encode("ascii"))
    raw = checked_bytes(kind, value)
    if kind == target:
        return raw
    if target == "buffer":
        result = raw + b"\0" if raw else b""
    else:
        result = b" ".join(f"{byte:02X}".encode("ascii") for byte in raw)
    if len(result) > 256:
        raise Rejected("Capacity")
    return result


def expected_number(opcode, left, right, bits):
    mask = (1 << bits) - 1
    if opcode == 0x92:
        return mask if number(*left, bits) == 0 else 0
    if opcode in (0x90, 0x91):
        # Deliberately admit both sides before applying Python's truth operators.
        a, b = number(*left, bits), number(*right, bits)
        truth = (a != 0 and b != 0) if opcode == 0x90 else (a != 0 or b != 0)
    else:
        target = "integer" if left[0] in ("inline", "integer") else left[0]
        a, b = converted(left, target, bits), converted(right, target, bits)
        truth = RELATIONS[opcode][1](a, b)
    return mask if truth else 0


def owned(slot, raw, string=False):
    kind, prefix = ("String", "string") if string else ("Buffer", "buffer")
    result = (
        f"store.space.objects[{slot}].value=Value::{kind} "
        f"{{{prefix}_storage:{kind}Storage::Owned {{{prefix}_owner:{slot}}}}};"
        f"store.bytes.blocks[{slot}]=ByteBlock {{initialized:true,length:{len(raw)}}};"
    )
    return result + "".join(
        f"store.bytes.blocks[{slot}].bytes[{i}]={byte};"
        for i, byte in enumerate(raw) if byte
    )


def operand(slot, value):
    kind, content = value
    if kind == "inline":
        return f"Operand::Integer {{number:{content}}}", ""
    setup = (f"store.space.objects[{slot}].value=Value::Integer {{number:{content}}};"
             if kind == "integer" else owned(slot, content, kind == "string"))
    return f"Operand::Object {{object_id:{slot}}}", setup


def cases():
    rows = []

    def add(name, opcode=0x93, left=("inline", 1), right=("inline", 1), bits=64,
            setup="", frame_setup="", error=None, result=None, left_operand=None,
            right_operand=None, parent=0, slot=0, no_parent=False):
        a, a_setup = operand(0, left)
        b, b_setup = operand(1, right)
        if error is None:
            try:
                result = expected_number(opcode, left, right, bits) if result is None else result
                error = "Success"
            except Rejected as failure:
                error = str(failure)
        expected = ""
        if error == "Success" and not no_parent:
            assert result is not None
            expected = (
                f"expected_frame.operations[{parent}].count={slot + 1};"
                f"expected_frame.operations[{parent}].operands[{slot}]="
                f"Operand::Integer {{number:{result}}};"
            )
        rows.append(dict(
            name=name, opcode=opcode, setup=a_setup + b_setup + setup,
            expected=expected, operand=left_operand or a, right=right_operand or b,
            target="Target::Null", bits=bits, error=error, scalar=False, copy=False,
            frame_setup=frame_setup, result=result if error == "Success" else None,
            parent=parent, slot=slot, no_parent=no_parent,
        ))

    # Every relation returns both Boolean values at both widths, with exact masks.
    for bits in (32, 64):
        for opcode, (label, relation) in RELATIONS.items():
            witnesses = [("less", 1, 2), ("equal", MAX, MAX), ("greater", MAX, 1)]
            selected = [next(row for row in witnesses if relation(row[1], row[2]) == truth)
                        for truth in (False, True)]
            for ordering, a, b in selected:
                add(f"{label}_{ordering}_{bits}", opcode, ("inline", a), ("inline", b), bits)
        for opcode, label in [(0x90, "and"), (0x91, "or")]:
            for a, b in [(0, 0), (0, 9), (7, 0), (7, 9)]:
                add(f"{label}_{a}_{b}_{bits}", opcode, ("inline", a), ("inline", b), bits)
        for value in (0, 1, 1 << 32):
            add(f"not_{value}_{bits}", 0x92, ("inline", value), bits=bits)

        # Directional coercion, including inline/Object paths and equal-prefix order.
        mixed = [
            ("integer_object", 0x93, ("inline", 42), ("integer", 42)),
            ("object_integer", 0x93, ("integer", 42), ("inline", 42)),
            ("integer_string", 0x93, ("integer", 42), ("string", b"2AZ")),
            ("integer_buffer", 0x93, ("inline", 42), ("buffer", b"\x2a")),
            ("string_integer", 0x93, ("string", f"{42:0{bits // 4}X}".encode()), ("inline", 42)),
            ("string_buffer", 0x93, ("string", b"00 AF"), ("buffer", b"\0\xaf")),
            ("buffer_integer", 0x93, ("buffer", (42).to_bytes(bits // 8, "little")), ("inline", 42)),
            ("buffer_string", 0x93, ("buffer", b"A\0"), ("string", b"A")),
            ("buffer_lexical", 0x95, ("buffer", b"\0\xff"), ("buffer", b"\x01")),
            ("string_prefix", 0x95, ("string", b"A"), ("string", b"AA")),
            ("truth_hex", 0x90, ("string", b"1Z"), ("buffer", b"\x01")),
            ("truth_no_0x_prefix", 0x91, ("string", b"0x12"), ("inline", 0)),
            ("truth_fifth_byte", 0x90, ("buffer", b"\0\0\0\0\x01"), ("inline", 1)),
        ]
        for name, opcode, left, right in mixed:
            add(f"{name}_{bits}", opcode, left, right, bits)

    # Reach both separate lexical retirement dispatches for every relation.
    lexical = {
        "buffer": [(0x93, b"A", b"B"), (0x94, b"B", b"A"),
                   (0x95, b"B", b"A"), (0x9293, b"A", b"B"),
                   (0x9294, b"B", b"A"), (0x9295, b"A", b"A")],
        "string": [(0x93, b"B", b"A"), (0x94, b"A", b"B"),
                   (0x95, b"A", b"A"), (0x9293, b"A", b"A"),
                   (0x9294, b"A", b"A"), (0x9295, b"A", b"B")],
    }
    for kind, witnesses in lexical.items():
        for opcode, left, right in witnesses:
            add(f"{kind}_{RELATIONS[opcode][0]}_dispatch", opcode, (kind, left), (kind, right))

    add("same_type_empty_buffer", left=("buffer", b""), right=("buffer", b""))
    add("same_type_empty_string", left=("string", b""), right=("string", b""))
    add("full_buffer", left=("buffer", b"Z" * 256), right=("buffer", b"Z" * 256))
    add("full_string", left=("string", b"Z" * 256), right=("string", b"Z" * 256))
    add("owned_tails_ignored", left=("buffer", b"A"), right=("buffer", b"A"),
        setup="store.bytes.blocks[0].bytes[255]=93;store.bytes.blocks[1].bytes[255]=94;")
    add("source_buffer_padding", left=("buffer", b"2A\0\0"), right=("buffer", b"2A\0\0"),
        setup="store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:4,buffer_initializer:Span {unit:7,end:2}}};")
    add("source_string", left=("string", b"2A"), right=("inline", 42), opcode=0x90,
        setup="store.space.objects[0].value=Value::String {string_storage:StringStorage::Source {string_source:Span {unit:7,end:2}}};")

    for kind in ("Named", "Local", "Arg"):
        add("transparent_" + kind.lower(), left=("integer", 42), right=("inline", 42),
            setup=f"store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:2}};store.space.objects[2].value=Value::Integer {{number:42}};")
    add("same_object_alias", left=("buffer", b"A"), right=("buffer", b"A"),
        setup="store.space.objects[1].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};")
    for kind in ("RefOf", "Index"):
        add("explicit_" + kind.lower(), left=("integer", 0),
            setup=f"store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};", error="UnsupportedValue")
    add("left_cycle", left=("integer", 1), setup="store.space.objects[0].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};", error="ReferenceCycle")
    add("right_cycle", right=("integer", 1), setup="store.space.objects[1].value=Value::Reference {kind:ReferenceKind::Named,object_id:1};", error="ReferenceCycle")
    for side in ("left", "right"):
        for label, object_id in [("fresh", 3), ("max", MAX)]:
            add(f"{side}_{label}_id", **{side + "_operand": f"Operand::Object {{object_id:{object_id}}}"}, error="InvalidState")
    add("object_count_max", left=("integer", 1), setup=f"store.space.object_count={MAX};", error="InvalidState")
    add("full_arena_object", left=("integer", 1), right=("integer", 1), setup="store.space.object_count=64;")
    add("full_arena_inline", setup="store.space.object_count=64;")
    add("inline_ignores_unrelated_metadata", setup=f"store.space.object_count={MAX};store.space.count={MAX};",
        frame_setup=f"frame.source_length={MAX};frame.body.unit={MAX};")
    add("owned_ignores_source_metadata", left=("buffer", b"A"), right=("buffer", b"A"),
        frame_setup=f"frame.source_length={MAX};frame.body.unit={MAX};")

    for label, slot, expression in [
        ("uninitialized_left", "left", "Operand::Uninitialized"),
        ("uninitialized_right", "right", "Operand::Uninitialized"),
    ]:
        add(label, **{slot + "_operand": expression}, error="Uninitialized")
    add("uninitialized_object", left=("integer", 1), setup="store.space.objects[0].value=Value::Uninitialized;", error="Uninitialized")
    for kind in ("Package", "FieldUnit"):
        add("unsupported_" + kind.lower(), left=("integer", 1), setup=f"store.space.objects[0].value=Value::{kind} {{}};", error="UnsupportedValue")
    add("not_ignores_invalid_right", opcode=0x92, left=("inline", 0), right_operand=f"Operand::Object {{object_id:{MAX}}}")
    add("not_ignores_uninitialized_right", opcode=0x92, left=("inline", 0), right_operand="Operand::Uninitialized")
    add("not_ignores_cycle_right", opcode=0x92, left=("inline", 0), right=("integer", 1),
        setup="store.space.objects[1].value=Value::Reference {kind:ReferenceKind::Named,object_id:1};")

    add("and_admits_decisive_right", opcode=0x90, left=("inline", 0), right=("string", b"1Z\xff"))
    add("or_admits_decisive_right", opcode=0x91, left=("inline", 1), right_operand="Operand::Uninitialized", error="Uninitialized")
    add("and_empty_right", opcode=0x90, left=("inline", 0), right=("buffer", b""))
    add("or_empty_right", opcode=0x91, left=("inline", 1), right=("string", b""))
    add("integer_empty_relation", left=("inline", 0), right=("buffer", b""))
    add("left_encoding_before_right_id", left=("string", b"A\xff"), right_operand=f"Operand::Object {{object_id:{MAX}}}")
    add("left_uninitialized_before_right_cycle", left_operand="Operand::Uninitialized", right=("integer", 1),
        setup="store.space.objects[1].value=Value::Reference {kind:ReferenceKind::Named,object_id:1};", error="Uninitialized")
    add("left_unsupported_before_right_id", left=("integer", 1), right_operand=f"Operand::Object {{object_id:{MAX}}}",
        setup="store.space.objects[0].value=Value::Package {};", error="UnsupportedValue")
    add("right_encoding_after_decisive_prefix", opcode=0x95, left=("string", b"A"), right=("string", b"Z\xff"))
    add("left_numeric_encoding_tail", opcode=0x90, left=("string", b"0Z\xff"), right=("inline", 0))
    add("string_buffer_conversion_capacity", left=("string", b""), right=("buffer", b"A" * 86))
    add("buffer_string_conversion_capacity", left=("buffer", b""), right=("string", b"A" * 256))
    add("left_capacity_before_right_id", left=("string", b"A"), right_operand=f"Operand::Object {{object_id:{MAX}}}",
        setup="store.bytes.blocks[0].length=257;", error="Capacity")
    add("wrong_owner", left=("buffer", b"A"), setup="store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:1}};", error="InvalidState")
    add("missing_backing", left=("buffer", b"A"), setup="store.bytes.blocks[0].initialized=false;", error="InvalidState")
    add("source_unit", left=("buffer", b"A"), setup="store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:1,buffer_initializer:Span {unit:8,end:1}}};", error="Bounds")

    add("parent_full", frame_setup="frame.operations[0].count=1;frame.operations[0].operands[0]=Operand::Integer {number:99};", error="InvalidState")
    add("parent_count_max", frame_setup=f"frame.operation_count={MAX};", error="InvalidState")
    add("last_parent_last_operand", parent=15, slot=6,
        frame_setup="frame.operation_count=16;frame.operations[15]=Operation {opcode:0xa4,value_arity:7,count:6};")
    add("no_parent", no_parent=True, frame_setup="frame.operation_count=0;")
    add("no_parent_still_admits_right", opcode=0x90, left=("inline", 0), right_operand="Operand::Uninitialized",
        no_parent=True, frame_setup="frame.operation_count=0;", error="Uninitialized")
    add("unknown_opcode_before_operands", opcode=0xffff, left_operand="Operand::Uninitialized", right_operand=f"Operand::Object {{object_id:{MAX}}}", error="BadEncoding")
    assert len({row["name"] for row in rows}) == len(rows)
    return rows


def render(selected):
    source, names = ns.render_bridge(selected)
    source += "\nuse execution::logical_execution::retire_logical;\n"
    marker = "let mut expected_store:ObjectStore="
    source = source.replace(marker, "frame.operation_count=1;frame.operations[0]=Operation {opcode:0xa4,value_arity:1};" + marker)
    for row in selected:
        old = f'write_generic_target(&input,&mut store,&mut frame,{row["target"]},{row["operand"]},false)'
        new = f'retire_logical(&input,&store,&mut frame,{row["opcode"]},{row["operand"]},{row["right"]})'
        for control in (False, True):
            suffix = "control" if control else "positive"
            start = source.index(f'machine Suite::{row["name"]}_{suffix}(')
            end = source.find("\nmachine Suite::", start + 1)
            end = len(source) if end < 0 else end
            body = source[start:end]
            assert body.count(old) == 1
            body = body.replace(old, new).replace(marker, row["frame_setup"] + marker)
            if control:
                # Stable under sub-selection: controls depend on the case name.
                mode = sum(row["name"].encode("ascii")) % 4
                mutation = "expected_frame.lookup_cache[15].object_id=72;"
                if mode == 0 and row["error"] == "Success" and not row["no_parent"]:
                    mutation = f'expected_frame.operations[{row["parent"]}].operands[{row["slot"]}]=Operand::Integer {{number:{row["result"] ^ 1}}};'
                elif mode == 1:
                    mutation = "expected_store.bytes.blocks[63].bytes[254]=1;"
                elif mode == 2:
                    mutation = ""
                    replacement = "InvalidState" if row["error"] == "Success" else "Success"
                    expected_error = f'outcome==ExecutionOutcome::{row["error"]}'
                    assert body.count(expected_error) == 1
                    body = body.replace(expected_error, f"outcome==ExecutionOutcome::{replacement}")
                assert body.count("expected_frame.lookup_cache[15].object_id=72;") == 1
                body = body.replace("expected_frame.lookup_cache[15].object_id=72;", mutation)
            source = source[:start] + body + source[end:]
    return source, names


if __name__ == "__main__":
    selected = cases()
    source, names = render(selected)
    assert len(names) == 2 * len(selected)
    (HERE / "bridge-cases.json").write_text(json.dumps(selected, indent=2) + "\n")
    print(f"{len(selected)} authored complete Store/Frame pairs; execution pending")
