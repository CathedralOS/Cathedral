"""Loaded AML composition with synthetic Field writes and independent byte traces.

The existing field-write driver and comparators are reused unchanged.  The
oracle below uses only Python integer/byte arithmetic, never production helpers.
These authored expectations are not checked execution evidence by themselves.
"""

import importlib.util
import json
import sys
from pathlib import Path

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
BASE = ROOT / "tools/ports/acpi/pipeline/field-writes/fixtures.py"
spec = importlib.util.spec_from_file_location("integration_field_write_fixtures", BASE)
fw = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fw)
MAX = (1 << 64) - 1


def encoded(kind, value):
    if kind == "integer":
        return fw.integer(value)
    if kind == "string":
        return b"\x0d" + value + b"\0"
    assert kind == "buffer"
    return fw.package([0x11], fw.integer(len(value)) + value)


def to_string(source, maximum):
    return b"\x9c" + source + fw.integer(maximum) + b"\0"


def concat(left, right, target=b"\0"):
    return b"\x73" + left + right + target


def mid(source, start, count):
    return b"\x9e" + source + fw.integer(start) + fw.integer(count) + b"\0"


def trace_oracle(kind, value, size, bits=13, offset=3, width=1, fail_at=MAX):
    """Preserve each touched native word, committing memory only on Write ACK."""
    memory = bytearray((index * 29 + 3) % 256 for index in range(512))
    initial = bytes(memory)
    if kind == "integer":
        parts = [value & ((1 << size) - 1) & ((1 << bits) - 1)]
    elif kind == "string":
        parts = [byte & ((1 << bits) - 1) for byte in value]
    else:
        assert kind == "buffer"
        number = int.from_bytes(value, "little")
        parts = [(number >> (ordinal * bits)) & ((1 << bits) - 1)
                 for ordinal in range(max(1, (len(value) * 8 + bits - 1) // bits))]
    events = []
    reads = writes = payloads = 0
    outcome = "Success"
    first = offset // (width * 8) * width
    last = (offset + bits + width * 8 - 1) // (width * 8) * width
    for ordinal, part in enumerate(parts):
        serial = 0
        for chunk, at in enumerate(range(first, last, width)):
            lo, hi = max(offset, at * 8), min(offset + bits, (at + width) * 8)
            count, shift = hi - lo, lo - at * 8
            mask = ((1 << count) - 1) << shift
            old = int.from_bytes(memory[at:at + width], "little")
            if count < width * 8:
                events.append(dict(kind="Read", field=1, payload=ordinal, serial=serial,
                    chunk=chunk, offset=at, width=width, value=0, response=old))
                serial += 1
                if len(events) - 1 == fail_at:
                    outcome = "UnresolvedRegion"
                    break
                reads += 1
            written = (old & ~mask) | (((part >> (lo - offset)) << shift) & mask)
            events.append(dict(kind="Write", field=1, payload=ordinal, serial=serial,
                chunk=chunk, offset=at, width=width, value=written, response=written))
            serial += 1
            if len(events) - 1 == fail_at:
                outcome = "UnresolvedRegion"
                break
            memory[at:at + width] = written.to_bytes(width, "little")
            writes += 1
        if outcome != "Success":
            break
        payloads += 1
    return dict(events=events, initial=initial.hex(), memory=bytes(memory).hex(),
                reads=reads, writes=writes, payloads=payloads, outcome=outcome)


def cases():
    rows = []

    def add(name, size, source, right, expression, kind, value, allocated,
            *, fail_at=MAX, quota=16384):
        width = 1 if size == 32 else 2
        bits, offset = 13, 3
        flags = {1: 1, 2: 2}[width]  # unlocked Preserve
        region = b"\x5b\x80REG0\0" + fw.integer(0) + fw.integer(512)
        field = fw.package([0x5b, 0x81], b"REG0" + bytes([flags]) +
                           b"\0" + fw.encoded_length(offset) + b"FLD0" + fw.encoded_length(bits))
        declarations = b"\x08SRC0" + encoded(*source)
        if right is not None:
            declarations += b"\x08RGHT" + encoded(*right)
        aml = region + field + declarations + fw.package([0x14], b"MAIN\0\xa4" + expression)
        initial_count = 4 if right is None else 5
        allocations = [dict(id=initial_count + index, kind=entry_kind, raw=raw.hex())
                       for index, (entry_kind, raw) in enumerate(allocated)]
        observed = trace_oracle(kind, value, size, bits, offset, width, fail_at)
        if observed["outcome"] == "Success" and kind != "integer" and len(value) > quota:
            observed["outcome"] = "Capacity"
        source_id = allocations[-1]["id"] if kind != "integer" else 0
        rows.append(dict(name=f"{name}_{size}", aml=aml.hex(), size=size, kind=kind,
            number=value if kind == "integer" else 0, raw="" if kind == "integer" else value.hex(),
            source_id=source_id, initial_count=initial_count, allocations=allocations,
            object_count=initial_count + len(allocations), bits=bits, offset=offset, width=width,
            base=0, fail_at=fail_at, quota=quota, **observed))

    for size in (32, 64):
        add("to_string_store", size, ("buffer", b"ABC\0Z"), None,
            b"\x70" + to_string(b"SRC0", 3) + b"FLD0", "string", b"ABC", [("string", b"ABC")])
        add("concat_direct_string_target", size, ("string", b"AB"), ("string", b"C"),
            concat(b"SRC0", b"RGHT", b"FLD0"), "string", b"ABC", [("string", b"ABC")])
        add("logical_to_string_store", size, ("buffer", b"AB\0Z"), ("string", b"AB"),
            b"\x70\x93" + to_string(b"SRC0", 2) + b"RGHTFLD0", "integer", (1 << size) - 1,
            [("string", b"AB")])
        add("mid_concat_store", size, ("buffer", bytes([1, 2, 3])), ("buffer", bytes([4, 5])),
            b"\x70" + mid(concat(b"SRC0", b"RGHT"), 1, 3) + b"FLD0", "buffer", bytes([2, 3, 4]),
            [("buffer", bytes([1, 2, 3, 4, 5])), ("buffer", bytes([2, 3, 4]))])
        add("to_string_concat_late_failure", size, ("buffer", b"AB\0Z"), ("string", b"CD"),
            b"\x70" + concat(to_string(b"SRC0", 2), b"RGHT") + b"FLD0", "string", b"ABCD",
            [("string", b"AB"), ("string", b"ABCD")], fail_at=3)
        add("concat_direct_quota_after_acks", size, ("buffer", bytes([0x12, 0x34])), ("buffer", bytes([0x56, 0x78])),
            concat(b"SRC0", b"RGHT", b"FLD0"), "buffer", bytes([0x12, 0x34, 0x56, 0x78]),
            [("buffer", bytes([0x12, 0x34, 0x56, 0x78]))], quota=2)
    assert len(rows) == len({row["name"] for row in rows}) == 12
    assert all(len(bytes.fromhex(row["aml"])) <= 1024 and len(row["events"]) < 512 for row in rows)
    assert all(row["writes"] > 0 for row in rows)
    return rows


def allocation_expectation(row):
    source = ""
    for allocation in row["allocations"]:
        object_id, kind = allocation["id"], allocation["kind"]
        raw = bytes.fromhex(allocation["raw"])
        name = f"allocated_{object_id}"
        source += fw.array(name, raw, 256)
        storage = (f"Value::String {{string_storage:StringStorage::Owned {{string_owner:{object_id}}}}}"
                   if kind == "string" else
                   f"Value::Buffer {{buffer_storage:BufferStorage::Owned {{buffer_owner:{object_id}}}}}")
        source += f"expected_store.space.objects[{object_id}]=Object {{value:{storage}}};\n"
        source += f"expected_store.bytes.blocks[{object_id}]=ByteBlock {{initialized:true,length:{len(raw)},bytes:{name}}};\n"
    source += f'expected_store.space.object_count={row["object_count"]};\n'
    return source


def render(selected):
    """Return one ordinary Suite module body and raw checked-runner selections."""
    assert selected
    source = fw.COMPARATOR.read_text().split("machine ba_check(")[0] + "\nuse aml::model::Outcome;\n" + fw.IMPORTS + fw.HELPERS
    names = []
    for row in selected:
        for control in (False, True):
            label = "Suite::" + row["name"] + ("_control" if control else "_positive")
            names.append(label + "=" + str(int(control)))
            source += f"\nmachine {label}(&mut self)->i32 {{\n" + fw.array("input", bytes.fromhex(row["aml"]), 1024)
            source += f'let prepared:Prepared=prepare_program(input,{len(bytes.fromhex(row["aml"]))},7,128,128);\n'
            source += f'let loaded:bool=prepared.outcome==Outcome::Success && prepared.program.store.space.object_count=={row["initial_count"]};\n'
            source += "let program:Program=prepared.program;let mut expected_store:ObjectStore=ObjectStore {space:program.store.space,bytes:program.store.bytes};\n"
            source += allocation_expectation(row)
            source += "let mut path:Path=Path {absolute:true,count:1};path.segments[0]=1313423693;let arguments:[Value;7];\n"
            source += f'let mut session:Session=begin_limited(program,path,&arguments,0,IntegerSize::{"FourBytes" if row["size"]==32 else "EightBytes"},1024,17,64,{row["quota"]});\n'
            source += "let mut memory:[u8;512];fw_seed(&mut memory,0,512);let mut expected_memory:[u8;512];fw_seed(&mut expected_memory,0,512);\n"
            source += "".join(f"expected_memory[{index}]={value};" for index, (old, value) in
                              enumerate(zip(bytes.fromhex(row["initial"]), bytes.fromhex(row["memory"]))) if old != value) + "\n"
            if control and row["outcome"] != "Success":
                source += "expected_memory[0]=expected_memory[0]^8;\n"
            source += f'let mut trace:Trace=Trace {{}};let mut expected:Trace=Trace {{count:{len(row["events"])}}};\n'
            for index, event in enumerate(row["events"]):
                outer = f'session:17,serial:{index+1},unit:7,field:1,region:0,base:0,payload:{event["payload"]}'
                inner = f'kind:RequestKind::{event["kind"]},correlation:17,serial:{event["serial"]},chunk:{event["chunk"]},offset:{event["offset"]},width:{event["width"]},value:{event["value"]}'
                source += f"expected.requests[{index}]=NativeRequest {{{outer},request:Request {{{inner}}}}};expected.values[{index}]={event['response']};\n"
            source += f'let outcome:ExecutionOutcome=fw_drive(&mut session,&mut memory,&mut trace,0,1024,{row["fail_at"]});\n'
            source += "let all_memory:bool=fw_memory(&memory,&expected_memory,0,512,true);let all_trace:bool=fw_trace(&trace,&expected,0,512,true);let store_same:bool=fx_store(&session.program.store,&expected_store);\n"
            expected_source = (f'Operand::Integer {{number:{row["number"]}}}' if row["kind"] == "integer"
                               else f'Operand::Object {{object_id:{row["source_id"]}}}')
            source += f"let incoming_same:bool=ba_operand(session.incoming,{expected_source});let source_same:bool=ba_operand(session.transfer.source,{expected_source});\n"
            if row["outcome"] == "Success":
                raw = bytes.fromhex(row["raw"])
                source += fw.array("result_bytes", raw, 256)
                kind = {"integer": 1, "buffer": 2, "string": 3}[row["kind"]]
                number = row["number"] ^ (1 if control and kind == 1 else 0)
                object_id = row["source_id"] ^ (1 if control and kind != 1 else 0)
                source += f"let result:bool=fw_result(&session,{kind},{number},{object_id},&result_bytes,{len(raw)});\n"
                terminal = "session.state==State::Finished && result"
            else:
                terminal = "session.state==State::Failed && !session.result.has_value"
            source += f'transition loaded && outcome==ExecutionOutcome::{row["outcome"]} && session.outcome==ExecutionOutcome::{row["outcome"]} && {terminal} && all_memory && all_trace && store_same && incoming_same && source_same && session.issued=={len(row["events"])} && session.accepted=={len(row["events"])} && session.reads=={row["reads"]} && session.writes=={row["writes"]} && session.payloads=={row["payloads"]} {{true -> (0) _ -> (1)}}\n}}\n'
    return source, names


if __name__ == "__main__":
    rows = cases()
    (HERE / "pipeline-cases.json").write_text(json.dumps(rows, indent=2) + "\n")
    print(len(rows), "authored loaded-AML behavior/control pairs; not execution proof")
