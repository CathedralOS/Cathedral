from pathlib import Path
import hashlib,json,re

HERE=Path(__file__).resolve().parent
ROOT=Path('/tmp/cathedral-field-reads-current')
BASE=Path('/tmp/cathedral-field-read-equality-session-micro/main.omg')
TINY=Path('/tmp/cathedral-field-read-check-inputs/current-tiny1/main.omg')
def sha(text):return hashlib.sha256(text.encode()).hexdigest()
sources={}
copies=[]
def read(path):
    text=path.read_text();sources[str(path)]=sha(text);return text
def machine(text,name):
    start=re.search(r'(?m)^(?:pub )?machine '+re.escape(name)+r'\(',text).start()
    brace=text.index('{',start);depth=1;end=brace+1
    while depth:
        depth+=(text[end]=='{')-(text[end]=='}');end+=1
    return text[start:end]+'\n'
def copied(path,name,changes=()):
    original=machine(read(path),name);changed=original
    for a,b in changes:
        assert a in changed;changed=changed.replace(a,b)
    copies.append(dict(source=str(path),machine=name,original_sha256=sha(original),copied_sha256=sha(changed),substitutions=list(changes)))
    return changed
base=read(BASE);declarations=base[:base.index('pub machine begin(')]
assert len(re.findall(r'(?m)^pub data ',declarations))==79
out=declarations
pipeline=ROOT/'source/libraries/acpi/pipeline/field_reads.omg'
out+=copied(pipeline,'prepare',[('geometry::plan','geometry_plan')])
for name in ['issue','fail','same_request']:out+=copied(pipeline,name)
geometry=ROOT/'source/libraries/acpi/field_access/geometry.omg'
geo=read(geometry);geo=geo[geo.index('data Builder '):]
changed=geo.replace('flags::decode_flags','decode_flags').replace('pub machine plan(','pub machine geometry_plan(')
copies.append(dict(source=str(geometry),section='Builder and all following geometry machines',original_sha256=sha(geo),copied_sha256=sha(changed),substitutions=[['flags::decode_flags','decode_flags'],['pub machine plan(','pub machine geometry_plan(']]))
out+=changed
flags=ROOT/'source/libraries/acpi/aml/field_flags.omg'
for name in ['decode_flags','access_kind','update_kind']:out+=copied(flags,name)
out+=copied(ROOT/'source/libraries/acpi/field_access/chunks.omg','low_mask')
out+=copied(ROOT/'source/libraries/acpi/interpreter/integers.omg','integer_bits')
tiny=read(TINY)
input_seed=re.search(r'let mut input:\[u8;1024\];[^\n]+',tiny).group()
names={name:int.from_bytes(name.encode(),'little') for name in ['REG0','FLD_','MAIN']}
out+='''
// Diagnostic direct seed of the first tiny fixture after its deferred read.
// AML loading, engine startup and decoder execution are deliberately absent.
machine seeded(poison:u64)->Session {
'''+input_seed+f'''
 let root:Path=Path {{absolute:true}};
 let mut region_name:Path=Path {{count:1}};region_name.segments[0]={names['REG0']};
 let mut field_name:Path=Path {{absolute:true,count:1}};field_name.segments[0]={names['FLD_']};
 let mut method_name:Path=Path {{absolute:true,count:1}};method_name.segments[0]={names['MAIN']};
 let declaration:Declaration=Declaration {{kind:DeclarationKind::Field,scope:root,primary_name:region_name,flags:1,source:Span {{unit:7,start:11,end:26}},field_list:Span {{unit:7,start:19,end:26}}}};
 let field:Field=Field {{name:field_name,bit_offset:5,bit_length:29,access:Access {{flags:1,kind:AccessType::Byte,update:UpdateRule::Preserve}},source:Span {{unit:7,start:21,end:26}}}};
 let mut session:Session=Session {{state:State::Running,program:Program {{loaded:true,source:input,length:38,unit:7}},correlation:77,fuel_limit:1024,result_objects:64,result_bytes:16384}};
 session.program.store.space.count=4;session.program.store.space.object_count=3;
 session.program.store.space.entries[0]=Entry {{path:root,has_level:true,level:LevelKind::Scope}};
 let mut absolute_region:Path=region_name;absolute_region.absolute=true;
 session.program.store.space.entries[1]=Entry {{path:absolute_region,has_object:true,object:0}};
 session.program.store.space.entries[2]=Entry {{path:field_name,has_object:true,object:1}};
 session.program.store.space.entries[3]=Entry {{path:method_name,has_object:true,object:2}};
 session.program.store.space.objects[0]=Object {{value:Value::OperationRegion {{space:0,base:0,length:512,scope:root}}}};
 session.program.store.space.objects[1]=Object {{value:Value::FieldUnit {{binding:FieldBinding::Region {{region_object:0}},declaration:declaration,field:field}}}};
 session.program.store.space.objects[2]=Object {{value:Value::Method {{body:Span {{unit:7,start:33,end:38}}}}}};
 session.program.definitions.entries[2]=MethodDefinition {{present:true,object_id:2,scope:root,body:Span {{unit:7,start:33,end:38}}}};
 session.runtime=Runtime {{count:1,steps:2}};
 session.runtime.frames[0]=Frame {{scope:root,body:Span {{unit:7,start:33,end:38}},source_length:38,size:IntegerSize::FourBytes,pc:34,end:38,operation_count:1,field_reads:true,deferred_read:DeferredRead::Field {{object:1,at:34,next:38}}}};
 session.runtime.frames[0].operations[0]=Operation {{opcode:0xa4,value_arity:1}};
 transition poison {{1 -> wrong_unit(session,declaration,field) 2 -> wrong_kind(session,declaration,field) 3 -> wrong_access(session,declaration,field) _ -> (session)}}
 state wrong_unit(session:Session,declaration:Declaration,field:Field)->Session {{let mut out:Session=session;let mut d:Declaration=declaration;d.source.unit=8;out.program.store.space.objects[1].value=Value::FieldUnit {{binding:FieldBinding::Region {{region_object:0}},declaration:d,field:field}};out}}
 state wrong_kind(session:Session,declaration:Declaration,field:Field)->Session {{let mut out:Session=session;let mut d:Declaration=declaration;d.kind=DeclarationKind::Bank;out.program.store.space.objects[1].value=Value::FieldUnit {{binding:FieldBinding::Region {{region_object:0}},declaration:d,field:field}};out}}
 state wrong_access(session:Session,declaration:Declaration,field:Field)->Session {{let mut out:Session=session;let mut f:Field=field;f.access.attribute=1;out.program.store.space.objects[1].value=Value::FieldUnit {{binding:FieldBinding::Region {{region_object:0}},declaration:declaration,field:f}};out}}
}}
data AdmissionProbe {{}}
machine AdmissionProbe::ready(&mut self)->i32 {{
 let mut session:Session=seeded(0);let outcome:ExecutionOutcome=prepare(&mut session,1);
 let request:NativeRead=NativeRead {{session:77,serial:1,unit:7,field:1,region:0,space:0,base:0,offset:0,width:1}};
 let same:bool=same_request(session.transfer.request,request);
 transition outcome==ExecutionOutcome::Success && session.state==State::AwaitingRead && session.outcome==ExecutionOutcome::Success && session.runtime.outcome==ExecutionOutcome::Success && session.runtime.steps==2 && session.issued==1 && session.accepted==0 && session.transfer.received==0 && session.transfer.plan.bit_offset==5 && session.transfer.plan.bit_length==29 && session.transfer.plan.count==5 && session.transfer.plan.width==1 && session.transfer.plan.end==5 && same {{true -> (0) _ -> (1)}}
}}
machine AdmissionProbe::wrong_unit(&mut self)->i32 {{let mut session:Session=seeded(1);let outcome:ExecutionOutcome=prepare(&mut session,1);let fault:bool=invalid_fault(session.fault);transition outcome==ExecutionOutcome::InvalidState && session.state==State::Failed && session.outcome==ExecutionOutcome::InvalidState && session.runtime.outcome==ExecutionOutcome::InvalidState && session.issued==0 && session.accepted==0 && fault {{true -> (0) _ -> (1)}}}}
machine AdmissionProbe::wrong_kind(&mut self)->i32 {{let mut session:Session=seeded(2);let outcome:ExecutionOutcome=prepare(&mut session,1);let fault:bool=invalid_fault(session.fault);transition outcome==ExecutionOutcome::InvalidState && session.state==State::Failed && session.outcome==ExecutionOutcome::InvalidState && session.runtime.outcome==ExecutionOutcome::InvalidState && session.issued==0 && fault {{true -> (0) _ -> (1)}}}}
machine AdmissionProbe::wrong_access(&mut self)->i32 {{let mut session:Session=seeded(3);let outcome:ExecutionOutcome=prepare(&mut session,1);let fault:bool=metadata_fault(session.fault);transition outcome==ExecutionOutcome::UnresolvedRegion && session.state==State::Failed && session.outcome==ExecutionOutcome::UnresolvedRegion && session.runtime.outcome==ExecutionOutcome::UnresolvedRegion && session.issued==0 && fault {{true -> (0) _ -> (1)}}}}
machine invalid_fault(fault:Fault)->bool {{transition fault {{Fault::InvalidState -> (true) _ -> (false)}}}}
machine metadata_fault(fault:Fault)->bool {{transition fault {{Fault::Geometry {{cause}} -> (cause==Error::UnsupportedMetadata) _ -> (false)}}}}
'''
(HERE/'main.omg').write_text(out)
build='machine build(builder:&mut Build){builder.application("read-admission-micro");builder.freestanding=true;}\n'
(HERE/'build.omg').write_text(build)
manifest=dict(kind='Diagnostic copied Field prepare/issue/fail over directly seeded nested Session; real geometry helpers; no loader/engine/decoder execution proof',sources=sources,data_declaration_count=79,data_prefix_sha256=sha(declarations),copies=copies,substitutions='Only two module qualifier/name substitutions listed per copied segment; no geometry stubs',seed='Direct metadata decoded by inspection from the exact first tiny AML bytes; Runtime represents retained Field read at PC34->38',main_sha256=sha(out),build_sha256=sha(build),selections=['AdmissionProbe::'+name+'=0' for name in ['ready','wrong_unit','wrong_kind','wrong_access']])
(HERE/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('wrote',len(out),'bytes with79 exact data declarations,2 geometry builder declarations and real geometry')
