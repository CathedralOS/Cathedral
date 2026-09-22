from pathlib import Path
import hashlib,json,re
HERE=Path(__file__).resolve().parent
ROOT=Path('/tmp/cathedral-field-reads-current')
ADMISSION=Path('/tmp/cathedral-field-read-admission-micro')
def sha(text):return hashlib.sha256(text.encode()).hexdigest()
sources={};copies=[]
def read(path):
    text=path.read_text();sources[str(path)]=sha(text);return text
def machine(text,name):
    start=re.search(r'(?m)^(?:pub )?machine '+re.escape(name)+r'\(',text).start()
    brace=text.index('{',start);depth=1;end=brace+1
    while depth:depth+=(text[end]=='{')-(text[end]=='}');end+=1
    return text[start:end]+'\n'
def copied(path,name,changes=()):
    original=machine(read(path),name);changed=original
    for a,b in changes:
        assert a in changed;changed=changed.replace(a,b)
    copies.append(dict(source=str(path),machine=name,original_sha256=sha(original),copied_sha256=sha(changed),substitutions=list(changes)))
    return changed
prior=json.loads(read(ADMISSION/'manifest.json'))
assert all(hashlib.sha256(Path(p).read_bytes()).hexdigest()==v for p,v in prior['sources'].items())
source=read(ADMISSION/'main.omg')
assert sha(source)==prior['main_sha256']
out=source[:source.index('data AdmissionProbe {}')]
pipeline=ROOT/'source/libraries/acpi/pipeline/field_reads.omg'
for name in ['complete','received']:out+=copied(pipeline,name)
out+=copied(ROOT/'source/libraries/acpi/interpreter/execution/engine.omg','complete_decoded_read')
out+=copied(ROOT/'source/libraries/acpi/interpreter/execution/operands.omg','contribute_operand')
model=read(ROOT/'source/libraries/acpi/field_values/model.omg')
out+=model[model.index('pub data ReadError '):]
out+='pub data IntegerResult [copy] { error: u8; value: u64; remainder: u64; }\n'
assembly_path=ROOT/'source/libraries/acpi/field_values/read.omg'
assembly=read(assembly_path);assembly=assembly[assembly.index('data Bytes '):]
changes=[('geometry::plan','geometry_plan'),('buffer_fields::field_to_integer','field_to_integer'),('chunks::extract','extract'),('conversions::integer_to_buffer','integer_to_buffer'),('buffer_fields::copy_bits','copy_bits')]
changed=assembly
for a,b in changes:assert a in changed;changed=changed.replace(a,b)
copies.append(dict(source=str(assembly_path),section='Bytes and all following assembly machines',original_sha256=sha(assembly),copied_sha256=sha(changed),substitutions=changes))
out+=changed
chunks=ROOT/'source/libraries/acpi/field_access/chunks.omg'
for name in ['valid','extract']:out+=copied(chunks,name)
convert=ROOT/'source/libraries/acpi/interpreter/conversions.omg'
for name in ['integer_to_buffer','integer_bytes','integer_byte']:out+=copied(convert,name)
buffer_path=ROOT/'source/libraries/acpi/interpreter/buffer_fields.omg'
buffer=read(buffer_path);buffer=buffer[buffer.index('pub machine source_bit('):]
out+=buffer
copies.append(dict(source=str(buffer_path),section='All buffer-fields machines',original_sha256=sha(buffer),copied_sha256=sha(buffer),substitutions=[]))
out+='''
// Explicit diagnostic-only boundaries. This corpus supplies Integer assembly.
// All untested object validation and allocation fail closed if reached.
machine validate_operand(input:&[u8;1024],length:u64,unit:u64,store:&ObjectStore,operand:Operand)->ExecutionOutcome {
 transition operand {Operand::Integer {number} -> (ExecutionOutcome::Success) Operand::Uninitialized -> (ExecutionOutcome::Uninitialized) _ -> (ExecutionOutcome::UnsupportedOpcode)}
}
machine allocate_mut(space:&mut Namespace,value:Value)->Read {Read {outcome:Outcome::InvalidState}}
machine completion_seed()->Session {
 let mut session:Session=seeded(0);session.runtime.length=38;session.runtime.unit=7;
 let outcome:ExecutionOutcome=prepare(&mut session,1);session
}
machine pending_exact(session:&Session,index:u64)->bool {
 let expected:NativeRead=NativeRead {session:77,serial:index+1,unit:7,field:1,region:0,space:0,base:0,offset:index,width:1};
 let request:bool=same_request(session.transfer.request,expected);
 session.state==State::AwaitingRead && session.outcome==ExecutionOutcome::Success && session.runtime.outcome==ExecutionOutcome::Success && session.issued==index+1 && session.accepted==index && session.transfer.received==index && session.runtime.steps==2 && session.runtime.frames[0].pc==34 && session.runtime.frames[0].operations[0].count==0 && request
}
machine feed(session:&mut Session,index:u64)->bool {
 let before:bool=pending_exact(session,index);let request:NativeRead=session.transfer.request;
 let value:u64=(index*37+19)%256;
 let outcome:ExecutionOutcome=complete(session,Completion::Word {request:request,value:value});
 let remembered:bool=word_saved(session,index,value);
 transition index<4 {true -> more(session,index,before,outcome,remembered) _ -> last(session,before,outcome,remembered)}
 state more(session:&Session,index:u64,before:bool,outcome:ExecutionOutcome,remembered:bool)->bool {let next:bool=pending_exact(session,index+1);before && outcome==ExecutionOutcome::Success && remembered && next}
 state last(session:&Session,before:bool,outcome:ExecutionOutcome,remembered:bool)->bool {let end:bool=finished_read(session);before && outcome==ExecutionOutcome::Success && remembered && end}
}
machine word_saved(session:&Session,index:u64,value:u64)->bool {transition index<257 {true -> (session.transfer.words[index]==value && session.transfer.received==index+1 && session.accepted==index+1) _ -> (false)}}
machine finished_read(session:&Session)->bool {
 let none:bool=no_deferred(session.runtime.frames[0].deferred_read);
 let value:bool=assembled_operand(session.runtime.frames[0].operations[0].operands[0]);
 session.state==State::Running && session.outcome==ExecutionOutcome::Success && session.runtime.outcome==ExecutionOutcome::Success && session.runtime.steps==2 && session.issued==5 && session.accepted==5 && session.transfer.received==5 && session.runtime.frames[0].pc==38 && session.runtime.frames[0].operation_count==1 && session.runtime.frames[0].operations[0].opcode==0xa4 && session.runtime.frames[0].operations[0].count==1 && !session.runtime.finished && !session.result.has_value && none && value
}
machine no_deferred(value:DeferredRead)->bool {transition value {DeferredRead::None -> (true) _ -> (false)}}
machine assembled_operand(value:Operand)->bool {transition value {Operand::Integer {number} -> (number==471001536) _ -> (false)}}
data CompletionProbe {}
machine CompletionProbe::all_words(&mut self)->i32 {
 let mut session:Session=completion_seed();
 let a:bool=feed(&mut session,0);let b:bool=feed(&mut session,1);let c:bool=feed(&mut session,2);let d:bool=feed(&mut session,3);let e:bool=feed(&mut session,4);
 transition a && b && c && d && e {true -> (0) _ -> (1)}
}
machine CompletionProbe::wrong_request(&mut self)->i32 {
 let mut session:Session=completion_seed();let pending:NativeRead=session.transfer.request;let mut wrong:NativeRead=pending;wrong.offset=1;
 let outcome:ExecutionOutcome=complete(&mut session,Completion::Word {request:wrong,value:19});
 let unchanged:bool=pending_exact(&session,0);let consumed:bool=feed(&mut session,0);
 transition outcome==ExecutionOutcome::InvalidState && unchanged && session.transfer.words[0]==19 && consumed {true -> (0) _ -> (1)}
}
machine CompletionProbe::stale_request(&mut self)->i32 {
 let mut session:Session=completion_seed();let old:NativeRead=session.transfer.request;let first:bool=feed(&mut session,0);
 let outcome:ExecutionOutcome=complete(&mut session,Completion::Word {request:old,value:255});
 let unchanged:bool=pending_exact(&session,1);let preserved:bool=session.transfer.words[0]==19 && session.transfer.words[1]==0;let second:bool=feed(&mut session,1);
 transition first && outcome==ExecutionOutcome::InvalidState && unchanged && preserved && second {true -> (0) _ -> (1)}
}
machine CompletionProbe::failure_success_rejected(&mut self)->i32 {
 let mut session:Session=completion_seed();let request:NativeRead=session.transfer.request;
 let outcome:ExecutionOutcome=complete(&mut session,Completion::Failure {request:request,outcome:ExecutionOutcome::Success});
 let unchanged:bool=pending_exact(&session,0);let consumed:bool=feed(&mut session,0);
 transition outcome==ExecutionOutcome::InvalidState && unchanged && consumed {true -> (0) _ -> (1)}
}
'''
(HERE/'main.omg').write_text(out)
build='machine build(builder:&mut Build){builder.application("read-completion-micro");builder.freestanding=true;}\n'
(HERE/'build.omg').write_text(build)
runner=Path('/tmp/cathedral-field-protocol-target/release/cathedral-acpi-checked-runner')
manifest=dict(kind='Diagnostic real Integer Field completion and assembly on directly seeded ready Session; no loader/execution dispatch proof',sources=sources,copies=copies,inherited_manifest=str(ADMISSION/'manifest.json'),stubs=['validate_operand retains only actual Integer/Uninitialized outcomes; all object operands fail UnsupportedOpcode','allocate_mut always fails InvalidState; Buffer allocation is excluded'],seed_extension='runtime.length=38 and runtime.unit=7 supplied before prepare; five words19,56,93,130,167; expected29bit471001536',main_sha256=sha(out),build_sha256=sha(build),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),runner=str(runner),runner_sha256=hashlib.sha256(runner.read_bytes()).hexdigest(),selections=['CompletionProbe::'+n+'=0' for n in ['all_words','wrong_request','stale_request','failure_success_rejected']])
(HERE/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('wrote',len(out),'bytes; real integer geometry/assembly/completion;2 fail-closed excluded-path stubs')
