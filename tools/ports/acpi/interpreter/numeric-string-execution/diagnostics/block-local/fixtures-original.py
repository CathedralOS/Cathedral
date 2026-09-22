"""Actual AML numeric-string conversions with independent Python expectations."""
import importlib.util
import json
import sys
from pathlib import Path

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]


def load(label, path):
    spec = importlib.util.spec_from_file_location(label, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


mid = load("numeric_string_mid_fixtures", HERE.parent / "mid-execution/fixtures.py")
ns = mid.ns
NAMED, GENERIC = mid.NAMED, mid.GENERIC
integer, pkg, ret, op, name = mid.integer, mid.pkg, mid.ret, mid.op, mid.name
MAX = (1 << 64) - 1
FORMATS = {"decimal": 0x97, "hex": 0x98}


def encoded(kind, value):
    if kind == "integer":
        return integer(value)
    if kind == "string":
        return b"\x0d" + value + b"\0"
    if kind == "package":
        return pkg(0x12, b"\0")
    assert kind == "buffer"
    return pkg(0x11, integer(len(value)) + value)


def expected(kind, value, bits, form):
    """Existing primary conversion profile plus retained pin hex presentation."""
    if kind == "integer":
        normalized = value & ((1 << bits) - 1)
        raw = (str(normalized) if form == "decimal" else f"0x{normalized:X}").encode()
    elif kind == "string":
        if any(byte == 0 or byte >= 128 for byte in value):
            return "InvalidState", b""  # Ordinary named TermArg admission precedes conversion.
        raw = value
    elif kind == "buffer":
        raw = ",".join(str(byte) if form == "decimal" else f"0x{byte:02X}" for byte in value).encode()
    else:
        return "UnsupportedValue", b""
    return ("Capacity", b"") if len(raw) > 256 else ("Success", raw)


def owned(object_id, raw):
    return dict(object_id=object_id, owner=object_id, length=len(raw), bytes=raw.hex())


def execution_cases():
    rows = []

    def add(label, form, source_kind, source_value, *, bits=64, target=b"\0", destination=None,
            body=None, methods=b"", setup=None, error=None, raw=None, count=None, result_object=None,
            fresh=None, owned_after=None, after=None, result_kind=3, number=0, source_encoding=None,
            reference_after=None, note=""):
        wanted, formatted = expected(source_kind, source_value, bits, form)
        wanted = wanted if error is None else error
        formatted = formatted if raw is None else raw
        destination = integer(7) if destination is None else destination
        source = encoded(source_kind, source_value) if source_encoding is None else source_encoding
        expression = op(FORMATS[form], name("SRC"), target)
        table = b"\x08" + name("SRC") + source + b"\x08" + name("DST") + destination
        table += pkg(0x14, name("MAIN") + b"\0" + (ret(expression) if body is None else body)) + methods
        initial = 4 if methods else 3
        fresh = initial if fresh is None else fresh
        success = wanted == "Success"
        if count is None:
            count = initial + int(success)
        if result_object is None and result_kind == 3 and success:
            result_object = fresh
        allocations = ([owned(fresh, formatted)] if success and result_kind == 3 else []) if owned_after is None else owned_after
        rows.append(dict(name=f"{form}_{label}_{bits}", table=table.hex(), kind=result_kind, number=number,
            bytes=formatted.hex() if result_kind == 3 else "", bits=bits, after=after or {}, object_count=count,
            error=wanted, note=note, setup=setup or [], mutate_source=False, result_object=result_object,
            owned_after=allocations, reference_after=reference_after or [], format=form))

    for form, opcode in FORMATS.items():
        for bits in (32, 64):
            for value in (0, 1, 0x100000001, MAX):
                add("integer_" + str(value), form, "integer", value, bits=bits, after={"DST": 7})
            for label, raw in [("empty_string", b""), ("string_identity", b"0xnot-a-number,+-"),
                               ("full_string", b"Z" * 256)]:
                add(label, form, "string", raw, bits=bits, after={"DST": 7})
            for label, raw in [("empty_buffer", b""), ("zero_buffer", b"\0"),
                               ("mixed_buffer", bytes([0, 1, 9, 10, 15, 16, 99, 100, 254, 255]))]:
                add(label, form, "buffer", raw, bits=bits, after={"DST": 7})
            limits = [("buffer_fit", bytes([255]) * 64), ("buffer_overflow", bytes([255]) * 65),
                      ("buffer_exact_256", bytes([255]) * 63 + bytes([10, 1])),
                      ("buffer_257", bytes([255]) * 64 + bytes([1]))] if form == "decimal" else [
                      ("buffer_fit", bytes([255]) * 51), ("buffer_overflow", bytes([255]) * 52)]
            for label, raw in limits:
                add(label, form, "buffer", raw, bits=bits, after={"DST": 7})
            for kind, value in [("integer", 0x100000001), ("string", b"Identity"), ("buffer", b"\0\xff")]:
                _, raw = expected(kind, value, bits, form)
                fresh = 3 if kind == "integer" else 4
                add("inline_" + kind, form, "integer", 0, bits=bits, raw=raw,
                    body=ret(op(opcode, encoded(kind, value), b"\0")), count=fresh + 1, fresh=fresh)
            _, raw = expected("buffer", b"A", bits, form)
            for kind, value in [("integer", 7), ("string", b"old longer"), ("buffer", b"old longer"), ("package", b"")]:
                add("named_" + kind + "_replacement", form, "buffer", b"A", bits=bits,
                    destination=encoded(kind, value), target=name("DST"), owned_after=[owned(3, raw), owned(1, raw)])
            add("self_target", form, "buffer", b"A", bits=bits, target=name("SRC"), owned_after=[owned(3, raw), owned(0, raw)])
            add("local_expression", form, "buffer", b"A", bits=bits, target=b"\x60", count=5,
                owned_after=[owned(3, raw), owned(4, raw)])
            add("local_return", form, "buffer", b"A", bits=bits,
                body=op(opcode, name("SRC"), b"\x60") + ret(b"\x60"), count=5, result_object=4,
                owned_after=[owned(3, raw), owned(4, raw)])
            add("nested", form, "buffer", b"A", bits=bits,
                body=ret(op(opcode, op(opcode, name("SRC"), b"\0"), b"\0")), count=5, result_object=4,
                owned_after=[owned(3, raw), owned(4, raw)])
            add("bare_statement", form, "buffer", b"A", bits=bits,
                body=op(opcode, name("SRC"), b"\0") + ret(name("DST")), result_kind=1, number=7,
                count=4, owned_after=[owned(3, raw)])
            callee = pkg(0x14, name("CAL") + b"\x01" + op(opcode, b"\x68", b"\x68") + ret(b"\x68"))
            add("callee_argument_local_copy", form, "buffer", b"A", bits=bits,
                body=ret(name("CAL") + name("SRC")), methods=callee, count=6, result_object=5,
                owned_after=[owned(4, raw), owned(5, raw)])
            callee = pkg(0x14, name("CAL") + b"\0" + op(0x70, integer(9), name("DST")) + ret(integer(0x100000001)))
            _, callee_raw = expected("integer", 0x100000001, bits, form)
            add("callee_earlier_effect", form, "integer", 0, bits=bits,
                body=ret(op(opcode, name("CAL"), b"\0")), methods=callee, raw=callee_raw, count=5, fresh=4, after={"DST": 9})
            oversized = bytes([255]) * (65 if form == "decimal" else 52)
            _, first = expected("integer", 7, bits, form)
            add("earlier_conversion_before_capacity", form, "buffer", oversized, bits=bits,
                body=op(opcode, integer(7), name("DST")) + ret(op(opcode, name("SRC"), b"\0")),
                count=4, error="Capacity", owned_after=[owned(3, first), owned(1, first)])
            add("virtual_buffer_padding", form, "buffer", b"A\0\0\0\0", bits=bits,
                source_encoding=pkg(0x11, integer(5) + b"A"))
            add("initializer_dominates", form, "buffer", b"ABC", bits=bits,
                source_encoding=pkg(0x11, integer(1) + b"ABC"), note="Canonical max(declared, initializer); public pin may panic during load.")
            add("package_source", form, "package", b"", bits=bits, after={"DST": 7})
        for kind in ("Named", "Local", "Arg"):
            _, raw = expected("buffer", b"A", 64, form)
            add("transparent_" + kind.lower(), form, "integer", 0, destination=encoded("buffer", b"A"), raw=raw,
                setup=[f"program.store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};"])
        for kind in ("RefOf", "Index"):
            add("explicit_" + kind.lower(), form, "integer", 0, error="UnsupportedValue",
                setup=[f"program.store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};"])
            _, raw = expected("buffer", b"A", 64, form)
            callee = pkg(0x14, name("CAL") + b"\x01" + ret(op(opcode, name("SRC"), b"\x68")))
            add("argument_" + kind.lower(), form, "buffer", b"A", body=ret(name("CAL") + name("DST")), methods=callee,
                setup=[f"program.store.space.objects[1].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:0}};"],
                count=5, fresh=4, owned_after=[owned(4, raw), owned(0, raw)],
                reference_after=[dict(object_id=1, kind=kind, referent=0)])
        add("full_arena", form, "buffer", b"A", error="Capacity", count=64, setup=["program.store.space.object_count=64;"])
        add("last_slot_null", form, "buffer", b"A", count=64, fresh=63, setup=["program.store.space.object_count=63;"])
        add("last_slot_local_rollback", form, "buffer", b"A", target=b"\x60", error="Capacity", count=63,
            setup=["program.store.space.object_count=63;"])
        add("debug_target", form, "buffer", b"A", target=b"\x5b\x31", error="UnresolvedService")
        add("source_cycle", form, "integer", 0, error="InvalidState",
            setup=["program.store.space.objects[0].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};"])
        for label, patches, error in [
            ("wrong_owner", ["program.store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:1}};"], "InvalidState"),
            ("uninitialized_backing", ["program.store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:0}};"], "InvalidState"),
            ("oversized_backing", ["program.store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:0}};", "program.store.bytes.blocks[0].initialized=true;program.store.bytes.blocks[0].length=257;"], "Capacity"),
            ("invalid_full_string", ["program.store.space.objects[0].value=Value::String {string_storage:StringStorage::Owned {string_owner:0}};", "program.store.bytes.blocks[0].initialized=true;program.store.bytes.blocks[0].length=2;program.store.bytes.blocks[0].bytes[0]=65;program.store.bytes.blocks[0].bytes[1]=255;"], "InvalidState"),
            ("wrong_source_unit", ["program.store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:1,buffer_initializer:Span {unit:8,start:0,end:1}}};"], "InvalidState"),
        ]:
            add(label, form, "buffer", b"A", error=error, setup=patches)
        _, raw = expected("string", b"AB", 64, form)
        add("dirty_string_tail", form, "string", b"AB", raw=raw,
            setup=["program.store.space.objects[0].value=Value::String {string_storage:StringStorage::Owned {string_owner:0}};", "program.store.bytes.blocks[0].initialized=true;program.store.bytes.blocks[0].length=2;program.store.bytes.blocks[0].bytes[0]=65;program.store.bytes.blocks[0].bytes[1]=66;program.store.bytes.blocks[0].bytes[255]=255;"])
    assert len(rows) == len({row["name"] for row in rows})
    assert all(len(bytes.fromhex(row["table"])) <= 1024 for row in rows)
    return rows


def rows(group):
    if group == "execution":
        return execution_cases()
    if group == "bridge":
        return load("numeric_string_bridge_fixtures", HERE / "bridge_fixtures.py").cases()
    return mid.rows({"mid": "execution", "mid_bridge": "bridge"}.get(group, group))


def render_rows(group, selected):
    if group == "bridge":
        return load("numeric_string_bridge_fixtures", HERE / "bridge_fixtures.py").render(selected)
    if group != "execution":
        return mid.render_rows({"mid": "execution", "mid_bridge": "bridge"}.get(group, group), selected)
    source, names = ns.base.render(selected)
    needle = "ge_same_bytes(&snapshot.bytes,&row.bytes,0,row.byte_count,true)"
    assert source.count(needle) == 1
    source = source.replace(needle, "ge_same_bytes(&snapshot.bytes,&row.bytes,0,256,true)")
    needle = "matched && transport,bytes,after,count,mutation"
    assert source.count(needle) == 1
    source = source.replace(needle, "matched && transport && nse_identity(result.operand,row.patch),bytes,after,count,mutation")
    needle = "let after:bool=ge_after(&program,row);"
    assert source.count(needle) == 1
    source = source.replace(needle, "let scalar_after:bool=ge_after(&program,row);let backing_after:bool=nse_after(&program,row.patch);let after:bool=scalar_after && backing_after;")
    source += "\nuse aml::model::StringStorage;\nuse aml::model::ByteBlock;\n"
    source += "machine nse_identity(operand:Operand,patch:u64)->bool {transition patch {"
    for index, row in enumerate(selected):
        source += f"{index+1} -> " + ("(true) " if row["result_object"] is None else f'object(operand,{row["result_object"]}) ')
    source += "_ -> (false)} state object(operand:Operand,wanted:u64)->bool {transition operand {Operand::Object {object_id} -> (object_id==wanted) _ -> (false)}}}\n"
    source += "machine nse_after(program:&Program,patch:u64)->bool {transition patch {"
    for index, row in enumerate(selected):
        source += f"{index+1} -> row_{index+1}(program) "
    source += "_ -> (false)}\n"
    for index, row in enumerate(selected):
        source += f"state row_{index+1}(program:&Program)->bool {{"
        checks = []
        for ordinal, block in enumerate(row["owned_after"]):
            raw = bytes.fromhex(block["bytes"])
            assert len(raw) == block["length"] <= 256 and block["owner"] == block["object_id"]
            source += f"let mut bytes_{ordinal}:[u8;256];"
            source += "".join(f"bytes_{ordinal}[{at}]={value};" for at, value in enumerate(raw) if value)
            source += f'let owned_{ordinal}:bool=nse_owned(program,{block["object_id"]},{len(raw)},&bytes_{ordinal});'
            checks.append(f"owned_{ordinal}")
        for ordinal, reference in enumerate(row["reference_after"]):
            source += f'let reference_{ordinal}:bool=nse_reference(program,{reference["object_id"]},ReferenceKind::{reference["kind"]},{reference["referent"]});'
            checks.append(f"reference_{ordinal}")
        source += (" && ".join(checks) if checks else "true") + "}\n"
    source += "}\n" + HELPERS
    return source, names


HELPERS = '''
machine nse_owned(program:&Program,id:u64,length:u64,expected:&[u8;256])->bool {
 let count:u64=program.store.space.object_count;
 transition count<=64 && id<count && id<64 {true -> value(program,id,length,expected) _ -> (false)}
 state value(program:&Program,id:u64,length:u64,expected:&[u8;256])->bool {
  transition program.store.space.objects[id].value {Value::String {string_storage} -> storage(program,id,length,expected,string_storage) _ -> (false)}
 }
 state storage(program:&Program,id:u64,length:u64,expected:&[u8;256],storage:StringStorage)->bool {
  transition storage {StringStorage::Owned {string_owner} -> owner(program,id,length,expected,string_owner) _ -> (false)}
 }
 state owner(program:&Program,id:u64,length:u64,expected:&[u8;256],owner:u64)->bool {
  transition owner==id && id<64 {true -> block(program,id,length,expected) _ -> (false)}
 }
 state block(program:&Program,id:u64,length:u64,expected:&[u8;256])->bool {
  let block:ByteBlock=program.store.bytes.blocks[id];let same:bool=ge_same_bytes(&block.bytes,expected,0,256,true);
  block.initialized && block.length==length && same
 }
}
machine nse_reference(program:&Program,id:u64,kind:ReferenceKind,referent:u64)->bool {
 let count:u64=program.store.space.object_count;
 transition count<=64 && id<count && id<64 {true -> value(program,id,kind,referent) _ -> (false)}
 state value(program:&Program,id:u64,kind:ReferenceKind,referent:u64)->bool {
  transition program.store.space.objects[id].value {Value::Reference {kind as actual,object_id} -> (actual==kind && object_id==referent) _ -> (false)}
 }
}
'''


if __name__ == "__main__":
    selected = execution_cases()
    (HERE / "execution-cases.json").write_text(json.dumps(selected, indent=2) + "\n")
    print(len(selected), "authored loaded-AML behavior/control pairs; not execution proof")
