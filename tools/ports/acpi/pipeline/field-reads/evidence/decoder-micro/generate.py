import hashlib,json,re
from pathlib import Path
ROOT=Path('/tmp/cathedral-field-reads-current')
HERE=Path(__file__).resolve().parent
prefix_path=Path('/tmp/cathedral-field-read-equality-session-micro/main.omg')
seed_path=Path('/tmp/cathedral-field-read-admission-micro/main.omg')
sha=lambda b:hashlib.sha256(b).hexdigest()
copies=[]
sources={}
def machine(path,name):
    path=Path(path)
    raw=path.read_text();sources[str(path)]=sha(path.read_bytes())
    match=re.search(r'^(?:pub )?machine '+re.escape(name)+r'\(',raw,re.M)
    assert match,(path,name)
    start=raw.index('{',match.end());depth=1;end=start+1
    while depth:
        depth += (raw[end]=='{')-(raw[end]=='}');end+=1
    body=raw[match.start():end]
    copies.append(dict(source=str(path),machine=name,sha256=sha(body.encode())))
    return body+'\n'
prefix=prefix_path.read_text().split('pub machine begin(',1)[0]
assert 'machine ' not in prefix
source=prefix
if not re.search(r'data OperationHeader\b',prefix):
    source+='pub data OperationHeader [copy] {value_arity:u64;target_arity:u64;count:u64;}\n'
groups=[
 ('source/libraries/acpi/interpreter/execution/decode_execution.omg',['decode_generic_value','decode_commit_cursor']),
 ('source/libraries/acpi/interpreter/execution/generic_values.omg',['object_outcome','read_value']),
 ('source/libraries/acpi/aml/object_references.omg',['unwrap_transparent','or_unwrap','or_reference_walk','or_reference_bounded_step','or_reference_step']),
 ('source/libraries/acpi/interpreter/execution/operands.omg',['operation_header']),
 ('source/libraries/acpi/interpreter/execution/engine.omg',['pending_read','advance_read_runtime','execution_run_turn']),
]
for path,names in groups:
    for name in names:source+=machine(ROOT/path,name)
source+=machine(seed_path,'seeded')
source+='''
// Explicit failing stubs for routes excluded by the selected Field operand.
machine validate_operand(input:&[u8;1024],length:u64,unit:u64,store:&ObjectStore,operand:Operand)->ExecutionOutcome {ExecutionOutcome::UnsupportedValue}
machine contribute_operand(frame:&mut Frame,operand:Operand)->ExecutionOutcome {ExecutionOutcome::UnsupportedValue}
// This synthetic turn bypasses loader, name resolution and other opcode paths.
machine execution_turn(input:&[u8;1024],store:&mut ObjectStore,definitions:&[MethodDefinition;64],runtime:&mut Runtime)->ExecutionOutcome {
 let result:ExecutionOutcome=decode_generic_value(input,store,&mut runtime.frames[0],38,OperandResult {operand:Operand::Object {object_id:1}});result
}
machine initial()->Session {let mut session:Session=seeded(0);session.runtime.steps=1;session.runtime.frames[0].deferred_read=DeferredRead::None;session.runtime.unit=7;session.runtime.length=38;session}
machine expected(frame:&Frame,next:u64)->bool {
 transition frame.deferred_read {DeferredRead::Field {object,at,next as actual} -> (object==1 && at==34 && actual==next && frame.pc==34 && frame.operation_count==1 && frame.operations[0].count==0) _ -> (false)}
}
machine absent(frame:&Frame)->bool {transition frame.deferred_read {DeferredRead::None -> (true) _ -> (false)}}
data DecoderProbe {}
machine DecoderProbe::direct(&mut self)->i32 {
 let session:Session=initial();let mut frame:Frame=session.runtime.frames[0];
 let result:ExecutionOutcome=decode_generic_value(&session.program.source,&session.program.store,&mut frame,38,OperandResult {operand:Operand::Object {object_id:1}});
 let ok:bool=expected(&frame,38);transition result==ExecutionOutcome::Success && ok {true -> (0) _ -> (1)}
}
machine DecoderProbe::changed_expectation(&mut self)->i32 {
 let session:Session=initial();let mut frame:Frame=session.runtime.frames[0];
 let result:ExecutionOutcome=decode_generic_value(&session.program.source,&session.program.store,&mut frame,38,OperandResult {operand:Operand::Object {object_id:1}});
 let ok:bool=expected(&frame,37);transition result==ExecutionOutcome::Success && ok {true -> (0) _ -> (1)}
}
machine DecoderProbe::runtime(&mut self)->i32 {
 let session:Session=initial();let input:[u8;1024]=session.program.source;let mut store:ObjectStore=session.program.store;let definitions:[MethodDefinition;64]=session.program.definitions.entries;let mut runtime:Runtime=session.runtime;
 let result:ExecutionOutcome=advance_read_runtime(&input,&mut store,&definitions,&mut runtime);
 let ok:bool=expected(&runtime.frames[0],38);transition result==ExecutionOutcome::Success && runtime.outcome==ExecutionOutcome::Success && runtime.steps==2 && ok {true -> (0) _ -> (1)}
}
machine DecoderProbe::session(&mut self)->i32 {
 let mut session:Session=initial();
 let result:ExecutionOutcome=advance_read_runtime(&session.program.source,&mut session.program.store,&session.program.definitions.entries,&mut session.runtime);
 let ok:bool=expected(&session.runtime.frames[0],38);transition result==ExecutionOutcome::Success && session.runtime.outcome==ExecutionOutcome::Success && session.runtime.steps==2 && ok {true -> (0) _ -> (1)}
}
machine DecoderProbe::pending_poll(&mut self)->i32 {
 let mut session:Session=initial();
 let result:ExecutionOutcome=advance_read_runtime(&session.program.source,&mut session.program.store,&session.program.definitions.entries,&mut session.runtime);
 let polled:ExecutionOutcome=advance_read_runtime(&session.program.source,&mut session.program.store,&session.program.definitions.entries,&mut session.runtime);
 let ok:bool=expected(&session.runtime.frames[0],38);transition result==ExecutionOutcome::Success && polled==ExecutionOutcome::Success && session.runtime.outcome==ExecutionOutcome::Success && session.runtime.steps==2 && ok {true -> (0) _ -> (1)}
}
machine DecoderProbe::parent_full(&mut self)->i32 {
 let mut session:Session=initial();session.runtime.frames[0].operations[0].count=1;
 let result:ExecutionOutcome=advance_read_runtime(&session.program.source,&mut session.program.store,&session.program.definitions.entries,&mut session.runtime);
 let none:bool=absent(&session.runtime.frames[0]);transition result==ExecutionOutcome::InvalidState && session.runtime.outcome==ExecutionOutcome::InvalidState && session.runtime.steps==2 && session.runtime.frames[0].pc==34 && none {true -> (0) _ -> (1)}
}
'''
build='machine build(builder:&mut Build){builder.package("field-read-decoder-micro");builder.freestanding=true;}\n'
(HERE/'main.omg').write_text(source);(HERE/'build.omg').write_text(build)
selections=['DecoderProbe::'+name+'='+str(want) for name,want in [('direct',0),('changed_expectation',1),('runtime',0),('session',0),('pending_poll',0),('parent_full',0)]]
manifest=dict(kind='Standalone copied decoder and read Runtime wrappers; synthetic execution_turn, no loader/names/real dispatch/provider; unreachable validation and contribution explicitly fail.',sources=sources,copies=copies,data_prefix_sha256=sha(prefix.encode()),data_prefix_source=str(prefix_path),main_sha256=sha(source.encode()),build_sha256=sha(build.encode()),generator_sha256=sha(Path(__file__).read_bytes()),selections=selections,stubs=['validate_operand always UnsupportedValue','contribute_operand always UnsupportedValue','execution_turn directly invokes exact decoder on Field slot1'],substitutions=[])
(HERE/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('Generated',len(copies),'exact machine copies;',len(selections),'diagnostic selections')
