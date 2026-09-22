"""Actual AML compositions across independently authored executor extensions."""
import importlib.util
from pathlib import Path

HERE=Path(__file__).resolve().parent
def load(label,path):
    spec=importlib.util.spec_from_file_location(label,path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
base=load('executor_mixed_mid',HERE.parent/'mid-execution/fixtures.py')
integer,pkg,ret,op,name=base.integer,base.pkg,base.ret,base.op,base.name
MAX=(1<<64)-1
def encoded(kind,value):
    if kind=='integer':return integer(value)
    if kind=='string':return b'\x0d'+value+b'\0'
    return pkg(0x11,integer(len(value))+value)
def call(code,*args):return op(code,*args)
def to_string(value,length=MAX,target=b'\0'):return call(0x9c,value,integer(length),target)
def concatenate(a,b,target=b'\0'):return call(0x73,a,b,target)
def mid(value,start,length,target=b'\0'):return call(0x9e,value,integer(start),integer(length),target)

def cases():
    rows=[]
    for bits in (32,64):
        def add(label,body,kind='integer',number=0,raw=b'',count=5,result_object=None,error='Success',after=None,text=('string',b'tail'),destination=b'?',source=b'AB\0Z',destination_after=None):
            declarations=b'\x08SRC0'+encoded('buffer',source)+b'\x08DST0'+encoded('buffer',destination)+b'\x08TXT0'+encoded(*text)+b'\x08NUM0'+integer(2)
            table=declarations+pkg(0x14,b'MAIN\0'+body)
            rows.append(dict(name=f'{label}_{bits}',table=table.hex(),kind={'integer':1,'buffer':2,'string':3}[kind],number=number,bytes=raw.hex(),bits=bits,
                after=after or {},object_count=count,error=error,note='',setup=[],mutate_source=False,result_object=result_object,destination_after=destination_after))
        left=to_string(concatenate(b'SRC0',b'TXT0'),2)
        right=to_string(mid(b'SRC0',0,2))
        add('nested_string_relation',ret(call(0x93,left,right)),number=(1<<bits)-1,count=9)
        add('empty_string_then_concat',ret(concatenate(to_string(b'SRC0',0),b'TXT0')),kind='string',raw=b'tail',count=7,result_object=6)
        add('mid_string_then_conversion',ret(to_string(mid(b'TXT0',1,2))),kind='string',raw=b'ai',count=7,result_object=6)
        add('replacement_then_concat',to_string(b'SRC0',2,b'DST0')+ret(concatenate(b'DST0',b'TXT0')),kind='string',raw=b'ABtail',count=7,result_object=6,
            destination_after=dict(object_id=1,kind='string',owner=1,length=2,bytes=b'AB'.hex(),zero_padding=True))
        add('buffer_store_then_conversion',concatenate(b'SRC0',b'SRC0',b'DST0')+ret(to_string(b'DST0')),kind='string',raw=b'AB',count=7,result_object=6,destination=b'?'*8,
            destination_after=dict(object_id=1,kind='buffer',owner=1,length=8,bytes=b'AB\0ZAB\0Z'.hex(),zero_padding=True))
        condition=call(0x93,to_string(b'SRC0',2),b'TXT0')
        branch=pkg(0xa0,condition+ret(mid(b'SRC0',1,2)))+pkg(0xa1,ret(integer(0)))
        add('logical_branch_then_mid',branch,kind='buffer',raw=b'B\0',count=7,result_object=6,text=('string',b'AB'))
        add('earlier_statement_before_failure',call(0x70,integer(9),b'NUM0')+to_string(b'SRC0',2,b'DST0')+ret(call(0x90,integer(0),b'TXT0')),
            count=6,error='Empty',after={'NUM0':9},text=('buffer',b''),
            destination_after=dict(object_id=1,kind='string',owner=1,length=2,bytes=b'AB'.hex(),zero_padding=True))
        add('integer_concat_then_conversion',ret(to_string(concatenate(integer(0x4241),integer(0x4443)))),kind='string',raw=b'AB',count=7,result_object=6)
    return rows

def render(selected):
    source,names=base.ns.base.render(selected)
    overflow='row.number=18446744073709551616;'
    count=sum(row['error']=='Success' and row['kind'] in (1,4,5) and row['number']==MAX for row in selected)
    assert source.count(overflow)==count
    source=source.replace(overflow,'row.number=0;')
    needle='ge_same_bytes(&snapshot.bytes,&row.bytes,0,row.byte_count,true)'
    assert source.count(needle)==1
    source=source.replace(needle,'ge_same_bytes(&snapshot.bytes,&row.bytes,0,256,true)')
    needle='matched && transport,bytes,after,count,mutation'
    assert source.count(needle)==1
    source=source.replace(needle,'matched && transport && ei_mixed_identity(result.operand,row.patch),bytes,after,count,mutation')
    needle='let after:bool=ge_after(&program,row);'
    assert source.count(needle)==1
    source=source.replace(needle,'let scalar_after:bool=ge_after(&program,row);let destination_after:bool=ei_mixed_destination(&program,row.patch);let after:bool=scalar_after && destination_after;')
    source+='\nmachine ei_mixed_identity(operand:Operand,patch:u64)->bool {transition patch {'
    for index,row in enumerate(selected):
        source+=f'{index+1} -> '+('(true) ' if row['result_object'] is None else f'object(operand,{row["result_object"]}) ')
    source+='_ -> (false)} state object(operand:Operand,wanted:u64)->bool {transition operand {Operand::Object {object_id} -> (object_id==wanted) _ -> (false)}}}\n'
    source+=destination_verifier(selected)
    return source,names


def destination_verifier(selected):
    # ToString prepares initialized zero output, then gv_install_bytes installs
    # its full snapshot. Buffer Store uses object_conversions.to_buffer's
    # logical copy into initialized zero storage before named_value_store.put.
    # Thus these particular operations promise zero backing beyond the extent;
    # this fixture does not assume every arbitrary Owned payload has zero padding.
    source='\nuse aml::model::ObjectStore;\nuse aml::model::ByteBlock;\nuse aml::model::StringStorage;\n'
    source+='machine ei_mixed_destination(program:&Program,patch:u64)->bool {transition patch {'
    for index,row in enumerate(selected):
        source+=f'{index+1} -> '+(f'row_{index+1}(program) ' if row['destination_after'] else '(true) ')
    source+='_ -> (false)}\n'
    for index,row in enumerate(selected):
        expected=row['destination_after']
        if expected is None:continue
        raw=bytes.fromhex(expected['bytes'])
        assert len(raw)==expected['length']<=256 and expected['zero_padding']
        source+=f'state row_{index+1}(program:&Program)->bool {{let mut expected:[u8;256];'
        source+=''.join(f'expected[{at}]={value};' for at,value in enumerate(raw) if value)
        kind={'buffer':2,'string':3}[expected['kind']]
        source+=f'let good:bool=ei_mixed_destination_value(&program.store,{expected["object_id"]},{kind},{expected["owner"]},{len(raw)},&expected);good}}\n'
    source+='}\n'
    source+='''machine ei_mixed_destination_value(store:&ObjectStore,id:u64,kind:u64,owner:u64,length:u64,expected:&[u8;256])->bool {
 let count:u64=store.space.object_count;
 transition count<=64 && id<count && id<64 {true -> value(store,id,kind,owner,length,expected) _ -> (false)}
 state value(store:&ObjectStore,id:u64,kind:u64,owner:u64,length:u64,expected:&[u8;256])->bool {
  transition store.space.objects[id].value {
   Value::String {string_storage} -> string(store,id,kind,owner,length,expected,string_storage)
   Value::Buffer {buffer_storage} -> buffer(store,id,kind,owner,length,expected,buffer_storage)
   _ -> (false)}
 }
 state string(store:&ObjectStore,id:u64,kind:u64,owner:u64,length:u64,expected:&[u8;256],storage:StringStorage)->bool {
  transition storage {StringStorage::Owned {string_owner} -> admitted(store,id,kind==3 && string_owner==owner,length,expected) _ -> (false)}
 }
 state buffer(store:&ObjectStore,id:u64,kind:u64,owner:u64,length:u64,expected:&[u8;256],storage:BufferStorage)->bool {
  transition storage {BufferStorage::Owned {buffer_owner} -> admitted(store,id,kind==2 && buffer_owner==owner,length,expected) _ -> (false)}
 }
 state admitted(store:&ObjectStore,id:u64,metadata:bool,length:u64,expected:&[u8;256])->bool {
  transition metadata && id<64 {true -> backing(store,id,length,expected) _ -> (false)}
 }
 state backing(store:&ObjectStore,id:u64,length:u64,expected:&[u8;256])->bool {
  let snapshot:ByteBlock=store.bytes.blocks[id];let bytes:bool=ge_same_bytes(&snapshot.bytes,expected,0,256,true);
  snapshot.initialized && snapshot.length==length && bytes
 }
}
'''
    return source
