"""ToString execution and whole-state retirement cases with independent bytes."""
import importlib.util
import json
from pathlib import Path

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[4]
def load(label,path):
    spec=importlib.util.spec_from_file_location(label,path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
mid=load('to_string_mid_fixtures',HERE.parent/'mid-execution/fixtures.py')
ns=mid.ns
NAMED,GENERIC=mid.NAMED,mid.GENERIC
integer,pkg,ret,op,name=mid.integer,mid.pkg,mid.ret,mid.op,mid.name
MAX=(1<<64)-1
string=lambda raw:b'\x0d'+raw+b'\0'
buffer=lambda raw:pkg(0x11,integer(len(raw))+raw)

def execution_cases():
    rows=[]
    def add(label,source,raw=b'A',maximum=MAX,bits=64,target=b'\0',destination=None,body=None,length_value=None,setup=None,error='Success',count=5,after=None,methods=b'',result_object=4):
        length_value=integer(maximum) if length_value is None else length_value
        globals=b'\x08'+name('SRC')+source+b'\x08'+name('DST')+(integer(7) if destination is None else destination)+b'\x08'+name('LEN')+length_value
        expression=op(0x9c,name('SRC'),name('LEN'),target)
        table=globals+pkg(0x14,name('MAIN')+b'\0'+(ret(expression) if body is None else body))+methods
        rows.append(dict(name=label,table=table.hex(),kind=3,number=0,bytes=raw.hex(),bits=bits,after=after or {},object_count=count,error=error,note='',setup=setup or [],mutate_source=False,result_object=result_object))
    for bits in (32,64):
        for label,data,maximum,want in [
            ('nul',b'A\0\xff',MAX,b'A'),('empty',b'',MAX,b''),
            ('zero',b'\xff',0,b''),('prefix',b'AB\xff',2,b'AB'),
            ('full',b'Z'*256,MAX,b'Z'*256),('first_nul',b'\0\xff',MAX,b''),
            ('no_nul',b'ABCDE',MAX,b'ABCDE'),('length_wrap',b'AB',1<<32,b'' if bits==32 else b'AB')]:
            add(f'{label}_{bits}',buffer(data),want,maximum,bits,after={'DST':7})
        add(f'integer_source_{bits}',integer(0x4847464544434241),b'ABCDEFGH'[:bits//8],bits=bits)
        add(f'string_source_{bits}',string(b'ABC'),b'ABC',bits=bits)
        add(f'empty_string_source_{bits}',string(b''),b'',bits=bits)
        add(f'inline_integer_{bits}',integer(0),b'AB',bits=bits,body=ret(op(0x9c,integer(0x4241),integer(MAX),b'\0')))
        add(f'hex_length_{bits}',buffer(b'0123456789ABCDEF'),b'0123456789',bits=bits,length_value=string(b'A'))
        add(f'buffer_length_{bits}',buffer(b'ABCDE'),b'AB',bits=bits,length_value=buffer(b'\x02'))
        add(f'prefix_length_{bits}',buffer(b'ABCDE'),b'A',bits=bits,length_value=string(b'1Z'))
        add(f'no_hex_prefix_length_{bits}',buffer(b'ABCDE'),b'',bits=bits,length_value=string(b'0x10'))
        add(f'named_integer_expression_{bits}',buffer(b'AB'),b'AB',bits=bits,target=name('DST'))
        add(f'named_buffer_replaced_{bits}',buffer(b'AB'),b'AB',bits=bits,destination=buffer(b'old longer'),body=op(0x9c,name('SRC'),integer(MAX),name('DST'))+ret(name('DST')),result_object=1)
        add(f'self_target_{bits}',buffer(b'AB\0X'),b'AB',bits=bits,target=name('SRC'))
        add(f'local_target_{bits}',buffer(b'ABC'),b'AB',bits=bits,body=op(0x9c,name('SRC'),integer(2),b'\x60')+ret(b'\x60'),count=6,result_object=5)
        add(f'nested_{bits}',buffer(b'ABCDE'),b'A',bits=bits,body=ret(op(0x9c,op(0x9c,name('SRC'),integer(3),b'\0'),integer(1),b'\0')),count=6,result_object=5)
        for label,source,length_value,error in [
            ('encoding',buffer(b'A\xff'),integer(MAX),'Encoding'),
            ('utf8',buffer(b'\xc3\xa9'),integer(MAX),'Encoding'),
            ('empty_length',buffer(b'A'),buffer(b''),'Empty'),
            ('empty_string_length',buffer(b'A'),string(b''),'Empty'),
            ('package',pkg(0x12,b'\0'),integer(MAX),'UnsupportedValue'),
            ('string_buffer_capacity',string(b'A'*256),integer(0),'Capacity')]:
            add(f'{label}_{bits}',source,b'',bits=bits,length_value=length_value,error=error,count=4)
    for kind in ['Named','Local','Arg']:
        add('transparent_'+kind.lower(),integer(0),b'AB',destination=buffer(b'AB'),setup=[f'program.store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};'])
    for kind in ['RefOf','Index']:
        add('explicit_'+kind.lower(),integer(0),b'',setup=[f'program.store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};'],error='UnsupportedValue',count=4)
    add('full_arena',buffer(b'A'),b'',setup=['program.store.space.object_count=64;'],error='Capacity',count=64)
    add('last_slot_local',buffer(b'A'),b'',target=b'\x60',setup=['program.store.space.object_count=63;'],error='Capacity',count=63)
    add('debug_target',buffer(b'A'),b'',target=b'\x5b\x31',error='UnresolvedService',count=4)
    add('source_cycle',integer(0),b'',setup=['program.store.space.objects[0].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};'],error='InvalidState',count=4)
    add('wrong_owner_before_zero',buffer(b'A'),b'',maximum=0,setup=['program.store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:1}};'],error='InvalidState',count=4)
    add('late_source_identity',buffer(b'old'),b'AB',body=ret(op(0x9c,name('SRC'),name('CAL'),b'\0')),methods=pkg(0x14,name('CAL')+b'\0'+op(0x9d,integer(0x4241),name('SRC'))+ret(integer(2))),after={'SRC':0x4241},count=6,result_object=5)
    add('callee_argument_target',buffer(b'ABC'),b'ABC',body=ret(name('CAL')+name('SRC')),methods=pkg(0x14,name('CAL')+b'\x01'+op(0x9c,b'\x68',integer(MAX),b'\x68')+ret(b'\x68')),count=7,result_object=6)
    return rows

def owned(slot,data,text=False,store='store',fresh=False):
    kind='String' if text else 'Buffer';field='string' if text else 'buffer'
    value=f'Value::{kind} {{{field}_storage:{kind}Storage::Owned {{{field}_owner:{slot}}}}}'
    obj=f'{store}.space.objects[{slot}]'+('=Object {value:'+value+'};' if fresh else '.value='+value+';')
    return obj+f'{store}.bytes.blocks[{slot}]=ByteBlock {{initialized:true,length:{len(data)}}};'+''.join(f'{store}.bytes.blocks[{slot}].bytes[{i}]={v};' for i,v in enumerate(data) if v)

def bridge_cases():
    rows=[]
    def add(label,setup='',expected='',source='Operand::Object {object_id:0}',target='Target::Null',maximum=f'Operand::Integer {{number:{MAX}}}',bits=64,error='Success',data=b'A',frame_setup='',result_id=3):
        if error=='Success':
            expected=f'expected_store.space.object_count={result_id+1};'+owned(result_id,data,True,'expected_store',True)+expected+f'expected_frame.operations[0].count=1;expected_frame.operations[0].operands[0]=Operand::Object {{object_id:{result_id}}};'
        rows.append(dict(name=label,setup=owned(0,b'A\0\xff')+setup,expected=expected,operand=source,target=target,maximum=maximum,bits=bits,error=error,scalar=False,copy=False,frame_setup=frame_setup))
    for bits in (32,64):
        add(f'null_{bits}',bits=bits)
        add(f'zero_{bits}',bits=bits,maximum='Operand::Integer {number:0}',data=b'')
        add(f'inline_{bits}',bits=bits,source='Operand::Integer {number:5208208757389214273}',data=b'ABCDEFGH'[:bits//8])
        add(f'object_integer_{bits}',bits=bits,setup='store.space.objects[0].value=Value::Integer {number:5208208757389214273};',data=b'ABCDEFGH'[:bits//8])
        add(f'length_wrap_{bits}',bits=bits,maximum='Operand::Integer {number:4294967296}',data=b'' if bits==32 else b'A')
        for typ,value in [('integer','Value::Integer {number:7}'),('package','Value::Package {}'),('reference','Value::Reference {kind:ReferenceKind::RefOf,object_id:2}'),('uninitialized','Value::Uninitialized')]:
            add(f'named_{typ}_{bits}',bits=bits,setup=f'store.space.objects[1].value={value};',target='Target::Named {object_id:1}',expected=owned(1,b'A',True,'expected_store'))
        for text in (False,True):
            add(f'named_{"string" if text else "buffer"}_{bits}',bits=bits,setup=owned(1,b'old longer',text)+'store.bytes.blocks[1].bytes[255]=91;',target='Target::Named {object_id:1}',expected=owned(1,b'A',True,'expected_store'))
        add(f'self_{bits}',bits=bits,target='Target::Named {object_id:0}',expected=owned(0,b'A',True,'expected_store'))
    for label,data,maximum,want,error in [
        ('empty',b'',MAX,b'','Success'),('full',b'Z'*256,MAX,b'Z'*256,'Success'),
        ('invalid_before_nul',b'A\xff\0',MAX,b'','Encoding'),('ignored_after_limit',b'A\xff',1,b'A','Success'),
        ('invalid_ignored_zero',b'\xff',0,b'','Success'),('initial_nul',b'\0\xff',MAX,b'','Success')]:
        add(label,setup=owned(0,data),maximum=f'Operand::Integer {{number:{maximum}}}',data=want,error=error)
    add('string_source',setup=owned(0,b'ABC',True),data=b'ABC')
    add('string_255',setup=owned(0,b'A'*255,True),data=b'A'*255)
    add('string_256_before_zero',setup=owned(0,b'A'*256,True),maximum='Operand::Integer {number:0}',error='Capacity')
    add('string_admission_before_zero',setup=owned(0,b'A\xff',True),maximum='Operand::Integer {number:0}',error='Encoding')
    add('source_padding',setup='store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:5,buffer_initializer:Span {unit:7,end:2}}};',data=b'2A')
    add('initializer_dominates',setup='store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:1,buffer_initializer:Span {unit:7,end:2}}};',data=b'2A')
    for label,data,text,want,error in [('hex',b'1Z',True,b'A','Success'),('prefix',b'0x2',True,b'','Success'),('buffer',b'\x01',False,b'A','Success'),('empty_string',b'',True,b'','Empty'),('empty_buffer',b'',False,b'','Empty')]:
        add('length_'+label,setup=owned(1,data,text),maximum='Operand::Object {object_id:1}',data=want,error=error)
    for kind in ['Named','Local','Arg']:
        add('transparent_'+kind.lower(),setup=owned(2,b'A\0\xff')+f'store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:2}};')
    for kind in ['RefOf','Index']:
        add('explicit_'+kind.lower(),setup=f'store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:2}};',error='UnsupportedValue')
    for mode,target,frame_setup,slot,extra in [
        ('local_new','Target::Local {local_index:0}','','4','expected_store.space.object_count=5;expected_frame.locals[0]=Binding::Cell {cell_id:4};'),
        ('local_cell','Target::Local {local_index:0}','frame.locals[0]=Binding::Cell {cell_id:2};','2',''),
        ('argument_plain','Target::Argument {argument_index:0}','frame.arguments[0]=Binding::Shared {shared_id:0};','4','expected_store.space.object_count=5;expected_frame.arguments[0]=Binding::Cell {cell_id:4};')]:
        add(mode,target=target,frame_setup=frame_setup,expected=extra+owned(slot,b'A',True,'expected_store'))
    for kind in ['RefOf','Index']:
        add('argument_'+kind.lower(),setup=f'store.space.objects[2].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};',frame_setup='frame.arguments[0]=Binding::Shared {shared_id:2};',target='Target::Argument {argument_index:0}',expected=owned(1,b'A',True,'expected_store'))
    add('last_slot_null',setup='store.space.object_count=63;',result_id=63)
    add('last_slot_local_rollback',setup='store.space.object_count=63;',target='Target::Local {local_index:0}',error='Capacity')
    add('full_arena',setup='store.space.object_count=64;',error='Capacity')
    add('namespace_capacity',setup='store.space.count=33;',error='Capacity')
    add('count_max',setup=f'store.space.object_count={MAX};',error='InvalidState')
    add('id_max',source=f'Operand::Object {{object_id:{MAX}}}',error='InvalidState')
    add('cycle',setup='store.space.objects[0].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};',error='ReferenceCycle')
    add('uninitialized',source='Operand::Uninitialized',error='Uninitialized')
    add('uninitialized_object',setup='store.space.objects[0].value=Value::Uninitialized;',error='Uninitialized')
    add('length_uninitialized',maximum='Operand::Uninitialized',error='Uninitialized')
    add('buffer_encoding_after_length',setup=owned(0,b'\xff'),maximum='Operand::Uninitialized',error='Uninitialized')
    add('string_encoding_before_length',setup=owned(0,b'\xff',True),maximum='Operand::Uninitialized',error='Encoding')
    add('source_precedes_length',source='Operand::Uninitialized',maximum=f'Operand::Object {{object_id:{MAX}}}',error='Uninitialized')
    add('malformed_before_zero',setup='store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:1}};',maximum='Operand::Integer {number:0}',error='InvalidState')
    add('source_unit_before_zero',setup='store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:1,buffer_initializer:Span {unit:8,end:1}}};',maximum='Operand::Integer {number:0}',error='Bounds')
    for label,target,frame_setup,setup,error in [
        ('fresh_named','Target::Named {object_id:3}','','','MissingObject'),
        ('max_named',f'Target::Named {{object_id:{MAX}}}','','','MissingObject'),
        ('fresh_local','Target::Local {local_index:0}','frame.locals[0]=Binding::Cell {cell_id:3};','','MissingObject'),
        ('fresh_argument','Target::Argument {argument_index:0}','frame.arguments[0]=Binding::Shared {shared_id:3};','','InvalidState'),
        ('fresh_referent','Target::Argument {argument_index:0}','frame.arguments[0]=Binding::Shared {shared_id:2};','store.space.objects[2].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:3};','MissingObject'),
        ('debug','Target::Debug','','','UnresolvedService'),('invalid_local','Target::Local {local_index:8}','','','InvalidTarget'),
        ('invalid_argument','Target::Argument {argument_index:7}','','','InvalidTarget'),
        ('field_target','Target::Named {object_id:1}','','store.space.objects[1].value=Value::FieldUnit {};','UnresolvedRegion'),
        ('method_target','Target::Named {object_id:1}','','store.space.objects[1].value=Value::Method {};','UnsupportedValue'),
        ('malformed_target','Target::Named {object_id:1}','','store.space.objects[1].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:2}};','InvalidState')]:
        add(label,target=target,frame_setup=frame_setup,setup=setup,error=error)
    add('parent_full',frame_setup='frame.operations[0].count=1;frame.operations[0].operands[0]=Operand::Integer {number:99};',error='InvalidState')
    add('parent_count_max',frame_setup=f'frame.operation_count={MAX};',error='InvalidState')
    return rows

def render_rows(group,selected):
    if group not in ['execution','bridge']:
        return mid.render_rows({'mid':'execution','mid_bridge':'bridge'}.get(group,group),selected)
    if group=='execution':
        source,names=ns.base.render(selected)
        source=source.replace('ge_same_bytes(&snapshot.bytes,&row.bytes,0,row.byte_count,true)','ge_same_bytes(&snapshot.bytes,&row.bytes,0,256,true)')
        source=source.replace('matched && transport,bytes,after,count,mutation','matched && transport && ts_identity(result.operand,row.patch),bytes,after,count,mutation')
        source+='\nmachine ts_identity(operand:Operand,patch:u64)->bool {transition patch {'
        for i,row in enumerate(selected):source+=f'{i+1} -> object(operand,{row["result_object"]}) '
        source+='_ -> (false)} state object(operand:Operand,wanted:u64)->bool {transition operand {Operand::Object {object_id} -> (object_id==wanted) _ -> (false)}}}\n'
        return source,names
    source,names=ns.render_bridge(selected)
    source+='\nuse execution::to_string_execution::retire_to_string;\n'
    source=source.replace('let mut expected_store:ObjectStore=','frame.operation_count=1;frame.operations[0]=Operation {opcode:0xa4,value_arity:1};let mut expected_store:ObjectStore=')
    for row in selected:
        old=f'write_generic_target(&input,&mut store,&mut frame,{row["target"]},{row["operand"]},false)'
        new=f'retire_to_string(&input,&mut store,&mut frame,{row["operand"]},{row["maximum"]},{row["target"]})'
        for suffix in ['positive','control']:
            start=source.index('machine Suite::'+row['name']+'_'+suffix+'(');end=source.find('\nmachine Suite::',start+1)
            end=len(source) if end<0 else end
            body=source[start:end];assert body.count(old)==1
            body=body.replace(old,new).replace('let mut expected_store:ObjectStore=',row['frame_setup']+'let mut expected_store:ObjectStore=')
            source=source[:start]+body+source[end:]
    return source,names

def rows(group):
    if group=='execution':return execution_cases()
    if group=='bridge':return bridge_cases()
    return mid.rows({'mid':'execution','mid_bridge':'bridge'}.get(group,group))

if __name__=='__main__':
    for group in ['execution','bridge']:
        selected=rows(group);source,names=render_rows(group,selected)
        assert len({r['name'] for r in selected})==len(selected)
        (HERE/(group+'-cases.json')).write_text(json.dumps(selected,indent=2)+'\n')
        print(group,len(selected),'authored behavior/control pairs; not execution proof')
