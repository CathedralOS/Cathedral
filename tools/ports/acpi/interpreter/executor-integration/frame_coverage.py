"""Complete the borrowed Frame oracle without changing historical fixtures.

Only recognized comparator bodies are accepted. Existing Concatenate/write
augmentation is retained byte-for-byte after its helper structure is checked.
These generators are authored assertions, not a claim of checked execution.
"""
import ast
import importlib.util
import re
from pathlib import Path

HERE = Path(__file__).resolve().parent
COMPARATOR = HERE.parent / "generic-execution/focused/bridge-atomicity/main.omg"
WRITE_FIXTURES = HERE.parent / "field-write-execution/fixtures.py"
MAX = (1 << 64) - 1
HELPER = "ei_frame_pending_write"
IMPORT = "use execution::execution_model::DeferredWrite;"
BASE_FRAME = """machine ba_frame(a:&Frame,b:&Frame)->bool {
 let scope:bool=fx_path(a.scope,b.scope);let body:bool=fx_span(a.body,b.body);let result:bool=ba_operand(a.return_value,b.return_value);
 let arrays:bool=ba_frame_arrays(a,b,0,16,true);
 scope && body && result && arrays && a.source_length==b.source_length && a.size==b.size && a.pc==b.pc && a.end==b.end && a.block_count==b.block_count && a.operation_count==b.operation_count && a.cache_count==b.cache_count && a.returned==b.returned && a.has_return_value==b.has_return_value
}"""
FRAME_HELPERS = (
    "fx_path", "fx_span", "fx_segments", "fx_segment", "ba_binding",
    "ba_operand", "ba_target", "ba_block", "ba_cache", "ba_operation",
    "ba_operands", "ba_operand_at", "ba_frame_arrays", "ba_frame_at",
    "ba_frame_small", "ba_frame_argument",
)


def _compact(source):
    source = re.sub(r"/\*.*?\*/|//[^\n]*", "", source, flags=re.S)
    return re.sub(r"\s+", "", source)


def _machine(source, name):
    """Return one complete machine, rejecting missing/duplicate definitions."""
    matches = list(re.finditer(r"\bmachine\s+" + re.escape(name) + r"\s*\(", source))
    if len(matches) != 1:
        raise ValueError(f"expected one {name} machine, found {len(matches)}")
    start = matches[0].start()
    opened = source.find("{", matches[0].end())
    if opened < 0:
        raise ValueError(f"missing body for {name}")
    depth = 0
    for at in range(opened, len(source)):
        if source[at] == "{":
            depth += 1
        elif source[at] == "}":
            depth -= 1
            if depth == 0:
                return start, at + 1, source[start:at + 1]
    raise ValueError(f"unclosed body for {name}")


def _deferred(name):
    definitions = [
        node for node in ast.parse(WRITE_FIXTURES.read_text()).body
        if isinstance(node, ast.Assign)
        and any(isinstance(t, ast.Name) and t.id == "DEFERRED" for t in node.targets)
    ]
    if len(definitions) != 1:
        raise ValueError("expected one literal field-write DEFERRED comparator")
    source = ast.literal_eval(definitions[0].value)
    # Bind all seven coordinates and delegate both operands and the target to
    # the same complete tagged comparators used for ordinary Frame members.
    required = (
        "DeferredWrite::None -> none(b)",
        "DeferredWrite::Field {object,source,result,stored,has_second,second_target,second_value}",
        "let source_same:bool=ba_operand(source,other_source);",
        "let result_same:bool=ba_operand(result,other_result);",
        "let target_same:bool=ba_target(target,other_target);",
        "id==other_id && stored==other_stored && has_second==other_second && value==other_value && source_same && result_same && target_same",
    )
    if not all(_compact(piece) in _compact(source) for piece in required):
        raise ValueError("field-write DEFERRED comparator has an unknown structure")
    return re.sub(r"\bfw_deferred\b", name, source).strip()


def _extended_frame(name):
    return BASE_FRAME.replace(
        "let arrays:bool=ba_frame_arrays(a,b,0,16,true);",
        "let arrays:bool=ba_frame_arrays(a,b,0,16,true);"
        f"let pending:bool={name}(a.deferred_write,b.deferred_write);",
    ).replace(
        " scope && body && result && arrays &&",
        " pending && a.field_writes==b.field_writes && scope && body && result && arrays &&",
    )


def augment(source):
    """Extend a recognized legacy oracle; reject partial/unknown coverage.

    A fully augmented input is returned unchanged only when its entire Frame
    comparison, recursive array/union helpers and import match known forms.
    """
    canonical = COMPARATOR.read_text()
    for name in FRAME_HELPERS:
        if _compact(_machine(source, name)[2]) != _compact(_machine(canonical, name)[2]):
            raise ValueError(f"unknown or incomplete Frame helper: {name}")
    start, end, frame = _machine(source, "ba_frame")
    import_count = len(re.findall(
        r"\buse\s+execution\s*::\s*execution_model\s*::\s*DeferredWrite\s*;", source
    ))
    if import_count > 1:
        raise ValueError("duplicate DeferredWrite imports")
    for name in (HELPER, "fw_deferred"):
        if _compact(frame) == _compact(_extended_frame(name)):
            if import_count != 1:
                raise ValueError("complete Frame comparator needs DeferredWrite import")
            if _compact(_machine(source, name)[2]) != _compact(_deferred(name)):
                raise ValueError(f"incomplete deferred-write comparator: {name}")
            return source
    if _compact(frame) != _compact(BASE_FRAME):
        raise ValueError("unknown or partially augmented ba_frame comparator")
    if any(re.search(r"\bmachine\s+" + name + r"\s*\(", source)
           for name in (HELPER, "fw_deferred")):
        raise ValueError("unwired deferred-write comparator already exists")
    result = source[:start] + _extended_frame(HELPER) + source[end:]
    if not import_count:
        result += "\n" + IMPORT + "\n"
    result += "\n" + _deferred(HELPER) + "\n"
    # Run the same structural admission on what this function publishes.
    if augment(result) != result:
        raise ValueError("augmentation failed its own complete-coverage check")
    return result


def _pending(**changes):
    fields = dict(
        object="63", source="Operand::Object {object_id:61}",
        result="Operand::Integer {number:18446744073709551615}",
        stored="true", has_second="true",
        second_target="Target::Argument {argument_index:6}", second_value="81985529216486895",
    )
    fields.update(changes)
    return "DeferredWrite::Field {" + ",".join(f"{k}:{v}" for k, v in fields.items()) + "}"


def control_cases():
    """26 comparator pairs plus six real retirement/preservation pairs."""
    rows = []

    def compare(name, pending=None, mutation="", mode=True):
        rows.append(dict(name="frame_" + name, kind="comparator", mode=mode,
                         pending=_pending() if pending is None else pending,
                         mutation=mutation))

    compare("mode_false", pending="DeferredWrite::None", mode=False,
            mutation="expected.field_writes=true;")
    compare("mode_true", mutation="expected.field_writes=false;")
    compare("none_to_field", pending="DeferredWrite::None",
            mutation="expected.deferred_write=" + _pending() + ";")
    compare("field_to_none", mutation="expected.deferred_write=DeferredWrite::None;")
    compare("object", mutation="expected.deferred_write=" + _pending(object=str(MAX)) + ";")
    for field in ("source", "result"):
        for label, before, after in (
            ("uninitialized", "Operand::Uninitialized", "Operand::Integer {number:0}"),
            ("integer", "Operand::Integer {number:63}", "Operand::Integer {number:64}"),
            ("object", "Operand::Object {object_id:63}", "Operand::Object {object_id:64}"),
            ("tag", "Operand::Integer {number:0}", "Operand::Object {object_id:0}"),
        ):
            compare(field + "_" + label, pending=_pending(**{field: before}),
                    mutation="expected.deferred_write=" + _pending(**{field: after}) + ";")
    for field in ("stored", "has_second"):
        for before, after in (("false", "true"), ("true", "false")):
            compare(field + "_" + before, pending=_pending(**{field: before}),
                    mutation="expected.deferred_write=" + _pending(**{field: after}) + ";")
    for label, before, after in (
        ("null", "Target::Null", "Target::Debug"),
        ("debug", "Target::Debug", "Target::Null"),
        ("local", "Target::Local {local_index:7}", "Target::Local {local_index:6}"),
        ("argument", "Target::Argument {argument_index:6}", "Target::Argument {argument_index:5}"),
        ("named", "Target::Named {object_id:63}", "Target::Named {object_id:64}"),
        ("tag", "Target::Local {local_index:0}", "Target::Named {object_id:0}"),
    ):
        compare("target_" + label, pending=_pending(second_target=before),
                mutation="expected.deferred_write=" + _pending(second_target=after) + ";")
    compare("second_value", pending=_pending(second_value=str(MAX)),
            mutation="expected.deferred_write=" + _pending(second_value="0") + ";")
    compare("inactive_second_target", pending=_pending(has_second="false"),
            mutation="expected.deferred_write=" + _pending(has_second="false", second_target="Target::Null") + ";")
    compare("inactive_second_value", pending=_pending(has_second="false"),
            mutation="expected.deferred_write=" + _pending(has_second="false", second_value="0") + ";")

    for operator, bits in (("logical", 32), ("to_string", 64), ("mid", 32)):
        for fails in (False, True):
            rows.append(dict(name="pending_" + operator + ("_rollback" if fails else "_success"),
                             kind="retirement", operator=operator, bits=bits, fails=fails))
    if len(rows) != 32 or len({row["name"] for row in rows}) != len(rows):
        raise ValueError("unexpected frame-coverage corpus")
    return rows


def _borrowed():
    spec = importlib.util.spec_from_file_location(
        "executor_frame_coverage_mid_fixtures", HERE.parent / "mid-execution/fixtures.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.ns


def _comparator_body(row, control):
    return f"""
 let mut frame:Frame=Frame {{source_length:97,size:IntegerSize::EightBytes,pc:19,end:79,block_count:2,operation_count:1,cache_count:3,returned:true,has_return_value:true,return_value:Operand::Object {{object_id:59}},field_writes:{str(row['mode']).lower()},deferred_write:{row['pending']}}};
 frame.scope.segments[15]=31;frame.body=Span {{unit:7,start:5,end:79}};
 frame.locals[7]=Binding::Cell {{cell_id:63}};frame.arguments[6]=Binding::Shared {{shared_id:62}};
 frame.blocks[7]=Block {{looping:true,parent_end:71,after:72,predicate:73,end:74}};
 frame.lookup_cache[15]=NameCache {{at:11,next:12,object_id:13,entry_index:14}};
 frame.operations[15]=Operation {{opcode:0x78,count:2,value_arity:2,target_arity:2,first_target:Target::Local {{local_index:7}},second_target:Target::Named {{object_id:63}}}};
 frame.operations[15].operands[6]=Operand::Object {{object_id:61}};
 let mut expected:Frame=frame;{row['mutation'] if control else ''}
 let forward:bool=ba_frame(&frame,&expected);let backward:bool=ba_frame(&expected,&frame);
 transition forward==backward {{true -> compared(forward) _ -> (2)}}
 state compared(same:bool)->i32 {{transition same {{true -> (0) _ -> (1)}}}}
"""


def _retirement_row(row):
    operator, fails = row["operator"], row["fails"]
    expected, setup = "", ""
    error = "InvalidState" if fails else "Success"
    frame_setup = "frame.operation_count=1;frame.operations[0]=Operation {opcode:0xa4,value_arity:1};"
    frame_setup += "frame.field_writes=true;frame.deferred_write=" + _pending() + ";"
    if fails:
        # A late full-parent failure exercises ToString/Mid's staged allocation
        # rollback while logical retirement must also leave every field intact.
        frame_setup += "frame.operations[0].count=1;frame.operations[0].operands[0]=Operand::Integer {number:99};"
    if operator == "logical":
        call = "retire_logical(&input,&store,&mut frame,0x93,Operand::Integer {number:7},Operand::Integer {number:7})"
        if not fails:
            expected = f"expected_frame.operations[0].count=1;expected_frame.operations[0].operands[0]=Operand::Integer {{number:{(1 << row['bits']) - 1}}};"
    else:
        if operator == "to_string":
            call = "retire_to_string(&input,&mut store,&mut frame,Operand::Integer {number:16961},Operand::Integer {number:2},Target::Null)"
            kind, lower, raw = "String", "string", b"AB"
        else:
            call = "retire_mid(&input,&mut store,&mut frame,Operand::Integer {number:1145258561},Operand::Integer {number:1},Operand::Integer {number:2},Target::Null)"
            kind, lower, raw = "Buffer", "buffer", b"BC"
        if not fails:
            expected = (
                "expected_store.space.object_count=4;"
                f"expected_store.space.objects[3]=Object {{value:Value::{kind} {{{lower}_storage:{kind}Storage::Owned {{{lower}_owner:3}}}}}};"
                "expected_store.bytes.blocks[3]=ByteBlock {initialized:true,length:2};"
                + "".join(f"expected_store.bytes.blocks[3].bytes[{i}]={value};" for i, value in enumerate(raw))
                + "expected_frame.operations[0].count=1;expected_frame.operations[0].operands[0]=Operand::Object {object_id:3};"
            )
    return dict(name=row["name"], setup=setup, expected=expected, bits=row["bits"],
                operand="Operand::Integer {number:7}", target="Target::Null",
                scalar=False, copy=False, error=error, call=call, frame_setup=frame_setup)


def render_controls(selected):
    """Return standalone source and positive=0/control=1 Suite selections."""
    selected = list(selected)
    if not selected or len({row["name"] for row in selected}) != len(selected):
        raise ValueError("select a nonempty unique set of frame-coverage rows")
    source = COMPARATOR.read_text().split("machine ba_check(")[0]
    names = []
    for row in selected:
        if row["kind"] == "comparator":
            for control in (False, True):
                label = "Suite::" + row["name"] + ("_control" if control else "_positive")
                names.append(label + "=" + str(int(control)))
                source += "\nmachine " + label + "(&mut self)->i32 {" + _comparator_body(row, control) + "}\n"
        elif row["kind"] == "retirement":
            prepared = _retirement_row(row)
            rendered, entries = _borrowed().render_bridge([prepared])
            for control, entry in zip((False, True), entries):
                label = entry.split("=")[0]
                body = _machine(rendered, label)[2]
                old = "write_generic_target(&input,&mut store,&mut frame,Target::Null,Operand::Integer {number:7},false)"
                if body.count(old) != 1:
                    raise ValueError("borrowed retirement invocation changed")
                body = body.replace(old, prepared["call"])
                snapshot = "let mut expected_store:ObjectStore="
                if body.count(snapshot) != 1:
                    raise ValueError("borrowed retirement snapshot changed")
                body = body.replace(snapshot, prepared["frame_setup"] + snapshot)
                if control:
                    old_control = "expected_frame.lookup_cache[15].object_id=72;"
                    if body.count(old_control) != 1:
                        raise ValueError("borrowed retirement control changed")
                    body = body.replace(old_control, "expected_frame.deferred_write=" + _pending(second_value="0") + ";")
                source += "\n" + body + "\n"
                names.append(entry)
        else:
            raise ValueError("unknown frame-coverage row kind")
    if any(row["kind"] == "retirement" for row in selected):
        source += "\nuse execution::logical_execution::retire_logical;\n"
        source += "use execution::to_string_execution::retire_to_string;\n"
        source += "use execution::mid_execution::retire_mid;\n"
    return augment(source), names
