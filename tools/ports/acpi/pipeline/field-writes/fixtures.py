"""Synthetic AML writes; independent bit arithmetic and full memory/trace checks."""
from pathlib import Path

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[4]
COMPARATOR=ROOT/'tools/ports/acpi/interpreter/generic-execution/focused/bridge-atomicity/main.omg'
MAX=(1<<64)-1

def integer(n):
    if n in (0,1):return bytes([n])
    for width,op in [(1,10),(2,11),(4,12),(8,14)]:
        if n<1<<(width*8):return bytes([op])+n.to_bytes(width,'little')
    raise ValueError(n)
def encoded_length(n):
    if n<64:return bytes([n])
    for following in range(1,4):
        if n<1<<(4+8*following):return bytes([(following<<6)|(n&15)])+(n>>4).to_bytes(following,'little')
    raise ValueError(n)
def package(op,body):
    for count in range(1,5):
        length=encoded_length(len(body)+count)
        if len(length)==count:return bytes(op)+length+body
    raise ValueError(len(body))
def array(name,data,size):
    assert len(data)<=size
    return f'let mut {name}:[u8;{size}];'+''.join(f'{name}[{i}]={v};' for i,v in enumerate(data) if v)+'\n'

def cases():
    rows=[]
    def add(name,source=0x123456789abcdef0,kind='integer',bits=13,offset=3,width=1,update=0,size=64,base=0,locked=False,space=0,region_length=512,fail_at=MAX,quota=16384,divide=False,late_debug=False):
        raw=(source if isinstance(source,bytes) else (source&((1<<size)-1)).to_bytes(size//8,'little'))
        encoded=integer(source) if kind=='integer' else (b'\x0d'+raw+b'\0' if kind=='string' else package([0x11],integer(len(raw))+raw))
        region=b'\x5b\x80REG0'+bytes([space])+integer(base)+integer(region_length)
        flags={1:1,2:2,4:3,8:4}[width]|(update<<5)|(16 if locked else 0)
        members=(b'\0'+encoded_length(offset) if offset else b'')+b'FLD0'+encoded_length(bits)
        if divide:members+=b'FLD1'+encoded_length(bits)
        fields=package([0x5b,0x81],b'REG0'+bytes([flags])+members)
        source_id=3 if divide else 2
        expression=(b'\x78'+integer(10)+integer(3)+b'FLD0'+(b'\x5b\x31' if late_debug else b'FLD1')) if divide else b'\x70SRC0FLD0'
        aml=region+fields+b'\x08SRC0'+encoded+package([0x14],b'MAIN\0\xa4'+expression)
        memory=bytearray((i*29+3)%256 for i in range(512));initial=bytes(memory);events=[];reads=writes=payloads=0
        outcome='Success'
        if locked:outcome='UnresolvedSynchronization'
        elif space or base%width or (offset+bits*(2 if divide else 1)+7)//8>region_length:outcome='UnresolvedRegion'
        transfers=[(1,offset,bits,1,'integer'),(2,offset+bits,bits,3,'integer')] if divide else [(1,offset,bits,source,kind)]
        if outcome=='Success':
            for field,start,length,value,source_kind in transfers:
                if late_debug and field==2:outcome='UnresolvedService';break
                if source_kind=='integer':parts=[value&((1<<size)-1)&((1<<length)-1)]
                elif source_kind=='string':parts=[v&((1<<length)-1) for v in value]
                else:
                    count=max(1,(len(value)*8+length-1)//length);number=int.from_bytes(value,'little')
                    parts=[(number>>(i*length))&((1<<length)-1) for i in range(count)]
                for ordinal,value in enumerate(parts):
                    serial=0;first=(start//(width*8))*width;last=((start+length+width*8-1)//(width*8))*width
                    for chunk,at in enumerate(range(first,last,width)):
                        lo=max(start,at*8);hi=min(start+length,(at+width)*8);count=hi-lo;shift=lo-at*8
                        mask=((1<<count)-1)<<shift;old=int.from_bytes(memory[base+at:base+at+width],'little')
                        previous=old if update==0 else ((1<<(width*8))-1 if update==1 else 0)
                        if count<width*8 and update==0:
                            events.append(dict(kind='Read',field=field,payload=ordinal,serial=serial,chunk=chunk,offset=at,width=width,value=0,response=old));serial+=1
                            if len(events)-1==fail_at:outcome='UnresolvedRegion';break
                            reads+=1
                        written=(previous&~mask)|(((value>>(lo-start))<<shift)&mask)
                        events.append(dict(kind='Write',field=field,payload=ordinal,serial=serial,chunk=chunk,offset=at,width=width,value=written,response=written));serial+=1
                        if len(events)-1==fail_at:outcome='UnresolvedRegion';break
                        memory[base+at:base+at+width]=written.to_bytes(width,'little');writes+=1
                    if outcome!='Success':break
                    payloads+=1
                if outcome!='Success':break
        if outcome=='Success' and kind!='integer' and not divide and len(raw)>quota:outcome='Capacity'
        rows.append(dict(name=name,aml=aml.hex(),size=size,kind='integer' if divide else kind,number=3 if divide else source if kind=='integer' else 0,raw='' if divide else raw.hex(),source_id=source_id,base=base,events=events,initial=initial.hex(),memory=bytes(memory).hex(),reads=reads,writes=writes,payloads=payloads,outcome=outcome,fail_at=fail_at,quota=quota))
    for size in (32,64):
        for width in (1,2,4,8):
            for update in (0,1,2):add(f'integer_{size}_{width}_{update}',size=size,width=width,update=update)
        add(f'buffer_rounds_{size}',source=b'\x12\x34\x56',kind='buffer',bits=7,size=size)
        add(f'string_rounds_{size}',source=b'ABC',kind='string',bits=5,size=size)
        add(f'empty_buffer_{size}',source=b'',kind='buffer',size=size)
        add(f'empty_string_{size}',source=b'',kind='string',size=size)
        add(f'wide_field_{size}',source=7,bits=65,size=size)
        add(f'divide_{size}',divide=True,bits=8,offset=0,size=size)
        add(f'divide_late_failure_{size}',divide=True,late_debug=True,bits=8,offset=0,size=size)
        add(f'quota_after_writes_{size}',source=b'ABC',kind='buffer',bits=8,offset=0,size=size,quota=2)
    for at in range(4):add(f'provider_failure_{at}',fail_at=at,bits=10,offset=3)
    add('locked',locked=True);add('unsupported_space',space=1);add('base_alignment',base=1,width=8)
    add('full_chunk',bits=64,offset=0,width=8)
    for field in ['session','serial','unit','field','region','space','base','payload']:
        add('reject_outer_'+field,bits=8,offset=0)
        rows[-1]['reject']='wrong.'+field+'=wrong.'+field+'^1;'
    for field in ['correlation','serial','chunk','offset','width','value']:
        add('reject_inner_'+field,bits=8,offset=0)
        rows[-1]['reject']='wrong.request.'+field+'=wrong.request.'+field+'^1;'
    add('reject_inner_kind',bits=8,offset=0);rows[-1]['reject']='wrong.request.kind=RequestKind::Read;'
    add('reject_wrong_word',bits=8,offset=0);rows[-1]['reject']='word'
    add('reject_wrong_written',bits=7,offset=3);rows[-1]['reject']='written'
    add('reject_stale_failure',bits=8,offset=0);rows[-1]['reject']='failure'
    add('reject_previous_payload',source=b'ABC',kind='string',bits=8,offset=0);rows[-1]['reject']='previous'
    add('reject_terminal_duplicate',bits=8,offset=0);rows[-1]['reject']='terminal'
    return rows

IMPORTS='''
use pipeline::program::Program;
use pipeline::program::Prepared;
use pipeline::program::prepare_program;
use pipeline::field_writes::Session;
use pipeline::field_writes::State;
use pipeline::field_writes::NativeRequest;
use pipeline::field_writes::Completion;
use pipeline::field_writes::Progress;
use pipeline::field_writes::begin_limited;
use pipeline::field_writes::advance;
use pipeline::field_writes::complete;
use pipeline::field_writes::same_request;
use write_values::transfer_model::Request;
use write_values::transfer_model::RequestKind;
use write_values::transfer_model::Acceptance;
use write_values::transfer_model::ProviderFailure;
use aml::byte_storage::ByteRead;
use aml::byte_storage::ByteOutcome;
use aml::byte_storage::ByteKind;
use aml::byte_storage::read_bytes;
'''

HELPERS='''
data Trace {count:u64;requests:[NativeRequest;512];values:[u64;512];}
machine fw_seed(memory:&mut [u8;512],index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
 {fw_seed_byte(memory,index);transition index<count {true -> fw_seed(memory,index+1,count) _ -> done()} state done(){}}
machine fw_seed_byte(memory:&mut [u8;512],index:u64) {
 transition index<512 {true -> put(memory,index) _ -> done()}
 state put(memory:&mut [u8;512],index:u64) {memory[index]=((index*29+3)%256) as u8;}
 state done(){}
}
machine fw_word(memory:&[u8;512],start:u64,width:u64,index:u64,value:u64)
terminates by(index,width)->Nat::BoundedDistance;
->u64 {let next:u64=fw_byte(memory,start,width,index,value);transition index<width {true -> fw_word(memory,start,width,index+1,next) _ -> (value)}}
machine fw_byte(memory:&[u8;512],start:u64,width:u64,index:u64,value:u64)->u64 {
 transition index<width && index<8 && start<512 && index<512-start {true -> read(memory,start,index,value) _ -> (value)}
 state read(memory:&[u8;512],start:u64,index:u64,value:u64)->u64 {
  let at:u64=start+index;transition at<512 && index<8 {true -> selected(memory,at,index,value) _ -> (value)}
 }
 state selected(memory:&[u8;512],at:u64,index:u64,value:u64)->u64 {let byte:u64=memory[at] as u64;value | (byte << (index*8))}
}
machine fw_bytes(memory:&mut [u8;512],start:u64,width:u64,index:u64,value:u64)
terminates by(index,width)->Nat::BoundedDistance;
 {fw_put(memory,start,width,index,value);transition index<width {true -> fw_bytes(memory,start,width,index+1,value) _ -> done()} state done(){}}
machine fw_put(memory:&mut [u8;512],start:u64,width:u64,index:u64,value:u64) {
 transition index<width && index<8 && start<512 && index<512-start {true -> at(memory,start+index,index,value) _ -> done()}
 state at(memory:&mut [u8;512],at:u64,index:u64,value:u64) {transition at<512 && index<8 {true -> put(memory,at,index,value) _ -> done()}}
 state put(memory:&mut [u8;512],at:u64,index:u64,value:u64) {memory[at]=((value>>(index*8))&255) as u8;}
 state done(){}
}
machine fw_drive(session:&mut Session,memory:&mut [u8;512],trace:&mut Trace,index:u64,limit:u64,fail_at:u64)
terminates by(index,limit)->Nat::BoundedDistance;
->ExecutionOutcome {
 let result:ExecutionOutcome=fw_turn(session,memory,trace,index,limit,fail_at);
 transition {session.state==State::Finished || session.state==State::Failed -> (result)
  index<limit -> fw_drive(session,memory,trace,index+1,limit,fail_at) _ -> (result)}
}
machine fw_turn(session:&mut Session,memory:&mut [u8;512],trace:&mut Trace,index:u64,limit:u64,fail_at:u64)->ExecutionOutcome {
 transition index<limit {true -> poll(session,memory,trace,fail_at) _ -> (ExecutionOutcome::WorkLimit)}
 state poll(session:&mut Session,memory:&mut [u8;512],trace:&mut Trace,fail_at:u64)->ExecutionOutcome {
  let next:Progress=advance(session,1);
  transition next {Progress::Pending {request} -> request(session,memory,trace,request,fail_at)
   Progress::Failure {outcome} -> (outcome) _ -> (ExecutionOutcome::Success)}
 }
 state request(session:&mut Session,memory:&mut [u8;512],trace:&mut Trace,request:NativeRequest,fail_at:u64)->ExecutionOutcome {
  let count:u64=trace.count;let offset:u64=request.request.offset;let width:u64=request.request.width;
  transition count<512 && request.base<=512 && offset<=512-request.base && width<=512-request.base-offset && width<=8 {
   true -> retain(session,memory,trace,request,fail_at,count,request.base+offset,width) _ -> (ExecutionOutcome::Capacity)}
 }
 state retain(session:&mut Session,memory:&mut [u8;512],trace:&mut Trace,request:NativeRequest,fail_at:u64,index:u64,start:u64,width:u64)->ExecutionOutcome {
  trace.requests[index]=request;trace.count=index+1;
  transition request.request.kind {RequestKind::Read -> read(session,memory,trace,request,fail_at,index,start,width) _ -> write(session,memory,trace,request,fail_at,index,start,width)}
 }
 state read(session:&mut Session,memory:&mut [u8;512],trace:&mut Trace,request:NativeRequest,fail_at:u64,index:u64,start:u64,width:u64)->ExecutionOutcome {
  let word:u64=fw_word(memory,start,width,0,0);trace.values[index]=word;
  transition index==fail_at {true -> failure(session,request) _ -> read_done(session,request,word)}
 }
 state read_done(session:&mut Session,request:NativeRequest,word:u64)->ExecutionOutcome {
  let accepted:Acceptance=complete(session,Completion::Word {request:request,value:word});
  transition accepted==Acceptance::Accepted {true -> (session.outcome) _ -> (ExecutionOutcome::InvalidState)}
 }
 state write(session:&mut Session,memory:&mut [u8;512],trace:&mut Trace,request:NativeRequest,fail_at:u64,index:u64,start:u64,width:u64)->ExecutionOutcome {
  trace.values[index]=request.request.value;
  transition index==fail_at {true -> failure(session,request) _ -> write_memory(session,memory,request,start,width)}
 }
 state write_memory(session:&mut Session,memory:&mut [u8;512],request:NativeRequest,start:u64,width:u64)->ExecutionOutcome {
  fw_bytes(memory,start,width,0,request.request.value);
  let accepted:Acceptance=complete(session,Completion::Written {request:request});
  transition accepted==Acceptance::Accepted {true -> (session.outcome) _ -> (ExecutionOutcome::InvalidState)}
 }
 state failure(session:&mut Session,request:NativeRequest)->ExecutionOutcome {
  let accepted:Acceptance=complete(session,Completion::Failure {request:request,cause:ProviderFailure::Denied});
  transition accepted==Acceptance::Accepted {true -> (session.outcome) _ -> (ExecutionOutcome::InvalidState)}
 }
}
machine fw_trace(a:&Trace,b:&Trace,index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let good:bool=fw_event(a,b,index);transition index<count {true -> fw_trace(a,b,index+1,count,prior && good) _ -> (prior && a.count==b.count)}}
machine fw_event(a:&Trace,b:&Trace,index:u64)->bool {
 transition index<512 {true -> at(a,b,index) _ -> (true)}
 state at(a:&Trace,b:&Trace,index:u64)->bool {let same:bool=same_request(a.requests[index],b.requests[index]);same && a.values[index]==b.values[index]}
}
machine fw_memory(a:&[u8;512],b:&[u8;512],i:u64,n:u64,prior:bool)
terminates by(i,n)->Nat::BoundedDistance;
->bool {let good:bool=fw_memory_at(a,b,i);transition i<n {true -> fw_memory(a,b,i+1,n,prior && good) _ -> (prior)}}
machine fw_memory_at(a:&[u8;512],b:&[u8;512],i:u64)->bool {transition i<512 {true -> (a[i]==b[i]) _ -> (true)}}
machine fw_result(session:&Session,kind:u64,number:u64,id:u64,bytes:&[u8;256],length:u64)->bool {
 transition session.result.has_value {true -> value(session,kind,number,id,bytes,length) _ -> (false)}
 state value(session:&Session,kind:u64,number:u64,id:u64,bytes:&[u8;256],length:u64)->bool {
  transition session.result.operand {Operand::Integer {number as actual} -> (kind==1 && actual==number)
   Operand::Object {object_id} -> object(session,kind,id,bytes,length,object_id) _ -> (false)}
 }
 state object(session:&Session,kind:u64,id:u64,bytes:&[u8;256],length:u64,object:u64)->bool {
  let read:ByteRead=read_bytes(&session.program.source,session.program.length,session.program.unit,&session.program.store,object);
  let same:bool=fx_bytes_equal(&read.bytes,bytes,0,256,true);
  object==id && read.outcome==ByteOutcome::Success && read.length==length && same && ((kind==2 && read.kind==ByteKind::Buffer) || (kind==3 && read.kind==ByteKind::String))
 }
}
'''

def render(match=''):
    rows=[r for r in cases() if not match or r['name'] in match.split(',')];assert rows
    # Existing complete canonical ObjectStore comparator only; no read Session helper.
    source=COMPARATOR.read_text().split('machine ba_check(')[0]
    source=source.replace(' let arrays:bool=ba_frame_arrays(a,b,0,16,true);',' let arrays:bool=ba_frame_arrays(a,b,0,16,true);let pending:bool=fw_deferred(a.deferred_write,b.deferred_write);')
    source=source.replace(' scope && body && result && arrays &&',' pending && a.field_writes==b.field_writes && scope && body && result && arrays &&')
    source+=IMPORTS+HELPERS+(HERE/'rejection-helpers.omg').read_text()
    names=[]
    for row in rows:
        for control in (False,True):
            label='Suite::'+row['name']+('_control' if control else '_positive');names.append(label+'='+str(int(control)))
            source+=f'\nmachine {label}(&mut self)->i32 {{\n'+array('input',bytes.fromhex(row['aml']),1024)
            source+=f'let prepared:Prepared=prepare_program(input,{len(bytes.fromhex(row["aml"]))},7,128,128);\n'
            source+='let mut program:Program=prepared.program;let expected_store:ObjectStore=ObjectStore {space:program.store.space,bytes:program.store.bytes};\n'
            source+='let mut path:Path=Path {absolute:true,count:1};path.segments[0]=1313423693;let arguments:[Value;7];\n'
            source+=f'let mut session:Session=begin_limited(program,path,&arguments,0,IntegerSize::{"FourBytes" if row["size"]==32 else "EightBytes"},1024,17,64,{row["quota"]});\n'
            if 'reject' in row:
                source+=rejection_body(row,control)
                continue
            source+='let mut memory:[u8;512];fw_seed(&mut memory,0,512);let mut expected_memory:[u8;512];fw_seed(&mut expected_memory,0,512);\n'
            source+=''.join(f'expected_memory[{i}]={v};' for i,(old,v) in enumerate(zip(bytes.fromhex(row['initial']),bytes.fromhex(row['memory']))) if old!=v)+'\n'
            source+=f'let mut trace:Trace=Trace {{}};let mut expected:Trace=Trace {{count:{len(row["events"])}}};\n'
            for i,e in enumerate(row['events']):
                outer=f'session:17,serial:{i+1},unit:7,field:{e["field"]},region:0,base:{row["base"]},payload:{e["payload"]}'
                inner=f'kind:RequestKind::{e["kind"]},correlation:17,serial:{e["serial"]},chunk:{e["chunk"]},offset:{e["offset"]},width:{e["width"]},value:{e["value"]}'
                source+=f'expected.requests[{i}]=NativeRequest '+'{'+outer+',request:Request {'+inner+'}};'+f'expected.values[{i}]={e["response"]};\n'
            if control:source+='expected.values[511]=1;\n'
            source+=f'let outcome:ExecutionOutcome=fw_drive(&mut session,&mut memory,&mut trace,0,1024,{row["fail_at"]});\n'
            source+='let all_memory:bool=fw_memory(&memory,&expected_memory,0,512,true);let all_trace:bool=fw_trace(&trace,&expected,0,512,true);let store_same:bool=fx_store(&session.program.store,&expected_store);\n'
            if row['outcome']=='Success':
                raw=bytes.fromhex(row['raw']) if row['kind']!='integer' else b''
                source+=array('result_bytes',raw,256)
                value=row['number']&((1<<row['size'])-1)
                kind={'integer':1,'buffer':2,'string':3}[row['kind']]
                source+=f'let result:bool=fw_result(&session,{kind},{value},{row["source_id"]},&result_bytes,{len(raw)});\n'
                final='session.state==State::Finished && result'
            else:final='session.state==State::Failed && !session.result.has_value'
            source+=f'transition outcome==ExecutionOutcome::{row["outcome"]} && session.outcome==ExecutionOutcome::{row["outcome"]} && {final} && all_memory && all_trace && store_same && session.issued=={len(row["events"])} && session.accepted=={len(row["events"])} && session.reads=={row["reads"]} && session.writes=={row["writes"]} && session.payloads=={row["payloads"]} {{true -> (0) _ -> (1)}}\n}}\n'
    return rows,source,names


def rejection_body(row,control):
    kind=row['reject']
    s='let pending:Progress=advance(&mut session,1024);let first:NativeRequest=session.transfer.request;let ready:bool=session.state==State::AwaitingRequest;\n'
    if kind in ('previous','terminal'):
        s+='let accepted:Acceptance=complete(&mut session,Completion::Written {request:first});\n'
        if kind=='terminal':
            s+='let finished:Progress=advance(&mut session,1024);let staged:bool=accepted==Acceptance::Accepted && session.state==State::Finished;\n'
        else:
            s+='let staged:bool=accepted==Acceptance::Accepted && session.state==State::AwaitingRequest && session.transfer.request.payload==1;\n'
        s+='let wrong:NativeRequest=first;\n'
    else:
        s+='let staged:bool=true;let mut wrong:NativeRequest=first;\n'
        if kind not in ('word','written','failure'):s+=kind+'\n'
        elif kind=='failure':s+='wrong.serial=wrong.serial^1;\n'
    s+='let mut expected_session:Session=fw_snapshot(&session);\n'
    if control:s+='expected_session.program.source[1023]=expected_session.program.source[1023]^1;\n'
    response='Completion::Written {request:wrong}'
    if kind=='word':response='Completion::Word {request:wrong,value:77}'
    elif kind=='failure':response='Completion::Failure {request:wrong,cause:ProviderFailure::Denied}'
    s+='let rejected:Acceptance=complete(&mut session,'+response+');let same:bool=fw_session(&session,&expected_session);\n'
    s+='transition ready && staged && rejected==Acceptance::Rejected && same {true -> (0) _ -> (1)}\n}\n'
    return s
