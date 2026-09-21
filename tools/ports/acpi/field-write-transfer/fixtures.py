#!/usr/bin/env python3
"""Independent native request traces and complete transfer-state comparisons."""
import importlib.util
import json
from pathlib import Path
import re
import sys
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
BASE=HERE.parent/'field-writes'
sys.path.insert(0,str(BASE))
import geometry_vectors as geometry

def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

geo=load('geometry_fixture',HERE.parent/'field-access/fixtures.py')
chunk=load('chunk_fixture',HERE.parent/'field-write-chunks/fixtures.py')
BRIDGE=HERE.parent/'interpreter/generic-execution/focused/bridge-atomicity/main.omg'
OLD=0x123456789abcdef0
MAX=(1<<64)-1

def machine(source,name):
    start=source.index('machine '+name+'(')
    body=source.index('{',start);depth=1;end=body+1
    while depth:
        depth+=(source[end]=='{')-(source[end]=='}');end+=1
    return source[start:end]+'\n'

def make(name,offset=3,bits=14,flags=1,size=8,region=300,**extra):
    plan=geometry.plan(offset,bits,flags,region,size)
    payload=[(i*37+11)%256 for i in range((bits+7)//8)] if 0<bits<=2048 else []
    row=dict(name=name,offset=offset,length=bits,flags=flags,size=size,region=region,payload=payload,payload_length=len(payload),geometry=plan,patch='',**extra)
    if 'error' in plan:
        row['failure']=dict(kind='Admission',error='Geometry',cause=plan['error'])
    elif extra.get('bad_length'):
        row['payload_length']=MAX;row['failure']=dict(kind='Admission',error='PayloadLength')
    elif flags&16:
        row['failure']=dict(kind='UnmetLock')
    else:
        data=int.from_bytes(bytes(payload),'little');events=[]
        for index,c in enumerate(plan['chunks']):
            preserve=c['partial'] and ((flags>>5)&3)==0
            if preserve:events.append(dict(kind='Read',correlation=99,serial=len(events),chunk=index,offset=c['offset'],width=c['width'],value=0))
            base=OLD if preserve else MAX if c['partial'] and ((flags>>5)&3)==1 else 0
            value=((base&~c['mask'])|(((data>>c['field_bit'])&((1<<c['bit_count'])-1))<<c['native_bit']))&((1<<(c['width']*8))-1)
            events.append(dict(kind='Write',correlation=99,serial=len(events),chunk=index,offset=c['offset'],width=c['width'],value=value))
        row['events']=events
    return row

def cases():
    rows=[]
    for code in range(5):
        width=[1,1,2,4,8][code]
        for update in range(3):
            for size in [4,8]:
                for shape,offset,bits in [('full',0,width*16),('partial',3,width*8+6)]:
                    rows.append(make(f'w{code}_u{update}_s{size}_{shape}',offset,bits,code|(update<<5),size))
    for bits in [1,31,32,33,63,64,65,2047,2048]:
        rows.append(make('extent_'+str(bits),7,bits))
    for index in range(5):rows.append(make('provider_failure_'+str(index),fail_at=index))
    for code in range(9):
        for mode,offset,bits in [('read',3,14),('write',0,16)]:
            rows.append(make(f'reject_{mode}_{code}',offset,bits,reject=code))
    rows.extend([make('locked',flags=17),make('locked_bad_payload',flags=17,bad_length=True),make('bad_payload',bad_length=True),
                 make('outside',region=1),make('zero',bits=0),make('overflow',offset=MAX,bits=8),make('capacity',bits=2049)])
    for kind in ['Bank','Index']:
        row=make('unsupported_'+kind)
        row['patch']='kind=DeclarationKind::'+kind+';'
        row['failure']=dict(kind='Admission',error='Geometry',cause='UnsupportedKind');row.pop('events');rows.append(row)
    rows.append(dict(name='default',default=True))
    assert len({r['name'] for r in rows})==len(rows)
    return rows

HEAD=geo.HEAD+'''use writes::transfer;
use writes::transfer_model::WriteTransfer;
use writes::transfer_model::TransferState;
use writes::transfer_model::TransferFailure;
use writes::transfer_model::TransferProgress;
use writes::transfer_model::ProviderFailure;
use writes::transfer_model::Request;
use writes::transfer_model::RequestKind;
use writes::transfer_model::Completion;
use writes::transfer_model::Acceptance;
use writes::model::WriteError;
use aml::model::Span;
use aml::field_model::Access;
data Payload [copy] {bytes:[u8;256];}
data Trace [copy] {events:[Request;514];}
'''
for name in ['fx_bytes_equal','fx_byte_equal','fx_span','fx_path','fx_segments','fx_segment','fx_access','fx_connection','fx_field']:
    HEAD+=machine(BRIDGE.read_text(),name)
HEAD+=machine(chunk.HEAD,'same_error')
HEAD+='''machine same_failure(a:TransferFailure,b:TransferFailure)->bool {
 transition a {TransferFailure::InvalidState -> invalid(b) TransferFailure::UnmetLock -> lock(b)
  TransferFailure::Admission {error} -> admission(error,b) TransferFailure::Provider {cause} -> provider(cause,b)}
 state invalid(b:TransferFailure)->bool {transition b {TransferFailure::InvalidState -> (true) _ -> (false)}}
 state lock(b:TransferFailure)->bool {transition b {TransferFailure::UnmetLock -> (true) _ -> (false)}}
 state admission(a:WriteError,b:TransferFailure)->bool {transition b {TransferFailure::Admission {error} -> compare(a,error) _ -> (false)}}
 state compare(a:WriteError,b:WriteError)->bool {let same:bool=same_error(a,b);same}
 state provider(a:ProviderFailure,b:TransferFailure)->bool {transition b {TransferFailure::Provider {cause} -> (a==cause) _ -> (false)}}
}
machine same_transfer(a:&WriteTransfer,b:&WriteTransfer)->bool {
 let failure:bool=same_failure(a.failure,b.failure);let field:bool=fx_field(a.field,b.field);
 let bytes:bool=fx_bytes_equal(&a.payload,&b.payload,0,256,true);
 let plan:bool=good_plan(PlanResult::Ready {plan:a.plan},&b.plan);
 let request:bool=transfer::same_request(a.request,b.request);
 a.state==b.state && failure && field && bytes && plan && request && a.region_bytes==b.region_bytes && a.size==b.size && a.length==b.length && a.correlation==b.correlation && a.issued==b.issued && a.reads==b.reads && a.writes==b.writes
}
machine rejected(current:&mut WriteTransfer,response:Completion)->bool {
 let before:WriteTransfer=current;let result:Acceptance=transfer::complete(current,response);
 let same:bool=same_transfer(&before,current);same && result==Acceptance::Rejected
}
machine flip_kind(kind:RequestKind)->RequestKind {transition kind {RequestKind::Read -> (RequestKind::Write) _ -> (RequestKind::Read)}}
machine reject_probe(current:&mut WriteTransfer,which:u64)->bool {
 let actual:Request=current.request;let mut wrong:Request=actual;
 transition which {
  0 -> kind(current,actual,wrong)
  1 -> correlation(current,actual,wrong)
  2 -> serial(current,actual,wrong)
  3 -> chunk(current,actual,wrong)
  4 -> offset(current,actual,wrong)
  5 -> width(current,actual,wrong)
  6 -> value(current,actual,wrong)
  7 -> wrong_response(current,actual)
  _ -> failure(current,actual,wrong)
 }
 state kind(current:&mut WriteTransfer,actual:Request,wrong:Request)->bool {let mut bad:Request=wrong;bad.kind=flip_kind(wrong.kind);reject_request(current,actual,bad,false)}
 state correlation(current:&mut WriteTransfer,actual:Request,wrong:Request)->bool {let mut bad:Request=wrong;bad.correlation=wrong.correlation^1;reject_request(current,actual,bad,false)}
 state serial(current:&mut WriteTransfer,actual:Request,wrong:Request)->bool {let mut bad:Request=wrong;bad.serial=wrong.serial^1;reject_request(current,actual,bad,false)}
 state chunk(current:&mut WriteTransfer,actual:Request,wrong:Request)->bool {let mut bad:Request=wrong;bad.chunk=wrong.chunk^1;reject_request(current,actual,bad,false)}
 state offset(current:&mut WriteTransfer,actual:Request,wrong:Request)->bool {let mut bad:Request=wrong;bad.offset=wrong.offset^1;reject_request(current,actual,bad,false)}
 state width(current:&mut WriteTransfer,actual:Request,wrong:Request)->bool {let mut bad:Request=wrong;bad.width=wrong.width^1;reject_request(current,actual,bad,false)}
 state value(current:&mut WriteTransfer,actual:Request,wrong:Request)->bool {let mut bad:Request=wrong;bad.value=wrong.value^1;reject_request(current,actual,bad,false)}
 state failure(current:&mut WriteTransfer,actual:Request,wrong:Request)->bool {let mut bad:Request=wrong;bad.serial=wrong.serial^1;reject_request(current,actual,bad,true)}
 state wrong_response(current:&mut WriteTransfer,actual:Request)->bool {transition actual.kind {RequestKind::Read -> wrong_write(current,actual) _ -> wrong_read(current,actual)}}
 state wrong_write(current:&mut WriteTransfer,actual:Request)->bool {let good:bool=rejected(current,Completion::Written {request:actual});good}
 state wrong_read(current:&mut WriteTransfer,actual:Request)->bool {let good:bool=rejected(current,Completion::Word {request:actual});good}
}
machine reject_request(current:&mut WriteTransfer,actual:Request,wrong:Request,failure:bool)->bool {
 transition failure {true -> failed(current,wrong) _ -> kind(current,actual,wrong)}
 state failed(current:&mut WriteTransfer,wrong:Request)->bool {let good:bool=rejected(current,Completion::Failure {request:wrong,cause:ProviderFailure::Denied});good}
 state kind(current:&mut WriteTransfer,actual:Request,wrong:Request)->bool {transition actual.kind {RequestKind::Read -> read(current,wrong) _ -> write(current,wrong)}}
 state read(current:&mut WriteTransfer,wrong:Request)->bool {let good:bool=rejected(current,Completion::Word {request:wrong});good}
 state write(current:&mut WriteTransfer,wrong:Request)->bool {let good:bool=rejected(current,Completion::Written {request:wrong});good}
}
machine drive(current:&mut WriteTransfer,trace:&Trace,index:u64,count:u64,failed:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {
 let good:bool=step(current,trace,index,count,failed);
 transition index<count {true -> drive(current,trace,index+1,count,failed,prior && good) _ -> (prior)}
}
machine step(current:&mut WriteTransfer,trace:&Trace,index:u64,count:u64,failed:u64)->bool {
 transition index<count && index<514 {true -> check(current,trace.events[index],index,failed) _ -> (true)}
 state check(current:&mut WriteTransfer,wanted:Request,index:u64,failed:u64)->bool {
  let progress:TransferProgress=transfer::progress(current);
  transition progress {TransferProgress::Pending {request} -> pending(current,request,wanted,index,failed) _ -> (false)}
 }
 state pending(current:&mut WriteTransfer,actual:Request,wanted:Request,index:u64,failed:u64)->bool {
  let same:bool=transfer::same_request(actual,wanted);
  let accepted:Acceptance=respond(current,actual,index==failed);
  same && accepted==Acceptance::Accepted
 }
}
machine respond(current:&mut WriteTransfer,request:Request,failure:bool)->Acceptance {
 transition failure {true -> failed(current,request) _ -> kind(current,request)}
 state failed(current:&mut WriteTransfer,request:Request)->Acceptance {let result:Acceptance=transfer::complete(current,Completion::Failure {request:request,cause:ProviderFailure::Denied});result}
 state kind(current:&mut WriteTransfer,request:Request)->Acceptance {transition request.kind {RequestKind::Read -> read(current,request) _ -> write(current,request)}}
 state read(current:&mut WriteTransfer,request:Request)->Acceptance {let result:Acceptance=transfer::complete(current,Completion::Word {request:request,value:1311768467463790320});result}
 state write(current:&mut WriteTransfer,request:Request)->Acceptance {let result:Acceptance=transfer::complete(current,Completion::Written {request:request});result}
}
'''

def request(event):
    return 'Request {'+','.join(key+':'+('RequestKind::'+value if key=='kind' else str(value)) for key,value in event.items())+'}'

def failure(value):
    kind=value['kind']
    if kind!='Admission':return 'TransferFailure::'+kind
    error='WriteError::'+value['error']
    if value['error']=='Geometry':error+=' {cause:Error::'+value['cause']+'}'
    return 'TransferFailure::Admission {error:'+error+'}'

def body(row,control=False):
    if row.get('default'):
        text='let mut current:WriteTransfer=WriteTransfer {};let mut expected:WriteTransfer=WriteTransfer {};'
    else:
        size='FourBytes' if row['size']==4 else 'EightBytes'
        text=f'let access:AccessResult=flags::decode_flags({row["flags"]});let field:Field=Field {{bit_offset:{row["offset"]},bit_length:{row["length"]},access:access.access}};let mut payload:Payload=Payload {{}};let mut kind:DeclarationKind=DeclarationKind::Field;'+row['patch']
        for index,value in enumerate(row['payload']):text+=f'payload.bytes[{index}]={value};'
        text+=f'let mut current:WriteTransfer=transfer::begin(kind,&field,{row["region"]},IntegerSize::{size},&payload.bytes,{row["payload_length"]},99);'
        if 'failure' in row:text+='let mut expected:WriteTransfer=WriteTransfer {failure:'+failure(row['failure'])+'};'
        else:
            events=row['events'];failed=row.get('fail_at',MAX);used=events[:failed+1] if failed!=MAX else events
            done=used[:-1] if failed!=MAX else used
            reads=sum(e['kind']=='Read' for e in done);writes=sum(e['kind']=='Write' for e in done)
            text+='let mut expected:WriteTransfer=current;let mut trace:Trace=Trace {};'
            for index,event in enumerate(used):text+=f'trace.events[{index}]='+request(event)+';'
            text+='expected.state=TransferState::'+('Failed' if failed!=MAX else 'Finished')+';'
            if failed!=MAX:text+='expected.failure=TransferFailure::Provider {cause:ProviderFailure::Denied};'
            text+=f'expected.reads={reads};expected.writes={writes};expected.issued={len(used)};expected.request='+request(used[-1])+';'
            text+='let probe:bool='+('reject_probe(&mut current,'+str(row['reject'])+')' if 'reject' in row else 'true')+';'
            text+=f'let traced:bool=drive(&mut current,&trace,0,{len(used)},{failed},true);'
    # Every scenario witnesses the complete 256-byte retained payload, including
    # inactive tails and admission/default records, without perturbing execution.
    if control:text+='expected.payload[255]=expected.payload[255]^1;'
    text+='let exact:bool=same_transfer(&current,&expected);let last:Request=current.request;let terminal:bool=rejected(&mut current,Completion::Written {request:last});'
    text+='let good:bool=exact && terminal'+(' && probe && traced' if not row.get('default') and 'failure' not in row else '')+';transition good {true -> (0) _ -> (1)}\n'
    return text

def render(rows):
    source=HEAD+'data Suite {}\n';entries=[]
    for row in rows:
        for control in [False,True]:
            name='Suite::'+row['name']+('_control' if control else '_positive')
            source+='machine '+name+'(&mut self)->i32 {\n'+body(row,control)+'}\n';entries.append(name+'='+str(int(control)))
    return source,entries

if __name__=='__main__':
    rows=cases();(HERE/'cases.json').write_text(json.dumps(rows,indent=2,sort_keys=True)+'\n');print(len(rows),'transfer pairs')
