"""Mid opcode and atomic retirement cases; expectations use independent slicing."""
import importlib.util,json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
NAMED=HERE.parent/'named-store-execution'
spec=importlib.util.spec_from_file_location('named_fixture',NAMED/'fixtures.py');ns=importlib.util.module_from_spec(spec);spec.loader.exec_module(ns)
GENERIC=ns.GENERIC
integer,pkg,ret,op,name=ns.integer,ns.pkg,ns.ret,ns.op,ns.name
MAX=(1<<64)-1
string=lambda raw:b'\x0d'+raw+b'\0'
buffer=lambda raw:pkg(0x11,integer(len(raw))+raw)

def execution_cases():
 rows=[]
 def add(label,source,raw,kind=2,index=1,length=3,bits=64,target=b'\0',body=None,destination=None,setup=None,error='Success',count=None,index_value=None,length_value=None,declarations=b'',after=None):
  encoded_index=integer(index) if index_value is None else index_value
  encoded_length=integer(length) if length_value is None else length_value
  globals=b'\x08'+name('SRC')+source+b'\x08'+name('DST')+(integer(7) if destination is None else destination)+b'\x08'+name('IDX')+encoded_index+b'\x08'+name('LEN')+encoded_length
  expression=op(0x9e,name('SRC'),name('IDX'),name('LEN'),target)
  table=globals+declarations+pkg(0x14,name('MAIN')+b'\0'+(ret(expression) if body is None else body))
  rows.append(dict(name=label,table=table.hex(),kind=kind,number=0,bytes=raw.hex(),bits=bits,after=({'DST':7} if destination is None else {}) if after is None else after,object_count=count,error=error,note='',setup=setup or [],mutate_source=False))
 for bits in (32,64):
  for typ,encoded,raw,kind in [('buffer',buffer(b'ABCDE'),b'ABCDE',2),('string',string(b'ABCDE'),b'ABCDE',3),('integer',integer(0x4847464544434241),(0x4847464544434241&((1<<bits)-1)).to_bytes(bits//8,'little'),2)]:
   for index,length in [(0,0),(0,MAX),(1,3),(4,MAX),(5,2),(MAX,1),(2,1)]:
    ix=index&((1<<bits)-1);ln=length&((1<<bits)-1)
    add(f'{typ}_{bits}_{index}_{length}',encoded,raw[ix:ix+ln],kind,index,length,bits,count=6)
  add(f'empty_buffer_{bits}',buffer(b''),b'',bits=bits,count=6)
  add(f'empty_string_{bits}',string(b''),b'',3,bits=bits,count=6)
  add(f'local_target_{bits}',string(b'ABCDE'),b'BCD',3,bits=bits,target=b'\x60',body=op(0x9e,name('SRC'),integer(1),integer(3),b'\x60')+ret(b'\x60'),count=7)
  add(f'string_named_integer_expression_{bits}',string(b'X2AY'),b'2A',3,index=1,length=2,bits=bits,target=name('DST'),after={'DST':42},count=6)
  add(f'buffer_named_integer_expression_{bits}',buffer(b'ABCDE'),b'BCD',bits=bits,target=name('DST'),after={'DST':int.from_bytes(b'BCD','little')},count=6)
  add(f'buffer_named_string_{bits}',buffer(b'ABCDE'),b'42 43 44',3,bits=bits,destination=string(b'old'),body=op(0x9e,name('SRC'),integer(1),integer(3),name('DST'))+ret(name('DST')),count=6)
  add(f'implicit_hex_index_{bits}',string(b'0123456789ABCDEFGHIJ'),b'AB',3,index=10,length=2,bits=bits,index_value=string(b'A'),count=6)
  add(f'implicit_buffer_length_{bits}',buffer(b'ABCDE'),b'BC',index=1,length=2,bits=bits,length_value=buffer(b'\x02'),count=6)
 add('nested_result',string(b'ABCDE'),b'CD',3,body=ret(op(0x9e,op(0x9e,name('SRC'),integer(1),integer(3),b'\0'),integer(1),integer(2),b'\0')),count=7)
 add('integer_inline_source',integer(0),b'BC',body=ret(op(0x9e,integer(0x44434241),integer(1),integer(2),b'\0')),count=6)
 for kind in ['Named','Local','Arg']:
  add('transparent_'+kind.lower(),integer(0),b'BCD',3,destination=string(b'ABCDE'),setup=[f'program.store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};'],count=6)
 for kind in ['RefOf','Index']:
  add('explicit_'+kind.lower(),integer(0),b'',setup=[f'program.store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};'],error='UnsupportedValue',count=5)
 add('package_source',pkg(0x12,b'\0'),b'',error='UnsupportedValue',count=5)
 add('debug_target',string(b'ABCDE'),b'',target=b'\x5b\x31',error='UnresolvedService',count=5)
 add('full_arena',string(b'ABCDE'),b'',setup=['program.store.space.object_count=64;'],error='Capacity',count=64)
 add('one_slot_local_failure',string(b'ABCDE'),b'',target=b'\x60',setup=['program.store.space.object_count=63;'],error='Capacity',count=63)
 add('empty_index',string(b'ABCDE'),b'',index_value=string(b''),error='Empty',count=5)
 add('empty_length',string(b'ABCDE'),b'',length_value=buffer(b''),error='Empty',count=5)
 add('source_cycle',integer(0),b'',setup=['program.store.space.objects[0].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};'],error='InvalidState',count=5)
 add('target_conversion_failure',string(b'ABCDE'),b'',target=name('DST'),index=1,length=1,setup=['program.store.space.objects[1].value=Value::Package {};'],error='UnsupportedValue',count=5,after={})
 add('unequal_buffer_target',buffer(b'ABCDE'),b'',target=name('DST'),destination=buffer(b'12345'),error='UnsupportedValue',count=5)
 return rows

def bridge_cases():
 rows=[]
 def add(label,setup='',expected='',source='Operand::Object {object_id:0}',target='Target::Null',index='Operand::Integer {number:1}',length='Operand::Integer {number:1}',bits=64,error='Success',data=b'A',string_result=True,frame_setup=''):
  if error=='Success':
   value='Value::String {string_storage:StringStorage::Owned {string_owner:3}}' if string_result else 'Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:3}}'
   expected='expected_store.space.object_count=4;expected_store.space.objects[3]=Object {value:'+value+'};expected_store.bytes.blocks[3]=ByteBlock {initialized:true,length:'+str(len(data))+'};'+''.join(f'expected_store.bytes.blocks[3].bytes[{i}]={v};' for i,v in enumerate(data))+expected+'expected_frame.operations[0].count=1;expected_frame.operations[0].operands[0]=Operand::Object {object_id:3};'
  rows.append(dict(name=label,setup='store.space.objects[0].value=Value::String {string_storage:StringStorage::Source {string_source:Span {unit:7,end:2}}};'+setup,expected=expected,operand=source,target=target,index=index,length=length,bits=bits,error=error,scalar=False,copy=False,frame_setup=frame_setup))
 for bits in (32,64):
  add(f'null_{bits}',bits=bits)
  add(f'named_integer_{bits}',bits=bits,target='Target::Named {object_id:1}',expected='expected_store.space.objects[1].value=Value::Integer {number:10};expected_store.bytes.blocks[1]=ByteBlock {};')
  add(f'named_string_{bits}',bits=bits,target='Target::Named {object_id:1}',setup='store.space.objects[1].value=Value::String {string_storage:StringStorage::Owned {string_owner:1}};store.bytes.blocks[1]=ByteBlock {initialized:true,length:2};store.bytes.blocks[1].bytes[0]=88;store.bytes.blocks[1].bytes[1]=89;store.bytes.blocks[1].bytes[255]=91;',expected='expected_store.bytes.blocks[1]=ByteBlock {initialized:true,length:1};expected_store.bytes.blocks[1].bytes[0]=65;')
  add(f'self_source_{bits}',bits=bits,target='Target::Named {object_id:0}',expected='expected_store.space.objects[0].value=Value::String {string_storage:StringStorage::Owned {string_owner:0}};expected_store.bytes.blocks[0]=ByteBlock {initialized:true,length:1};expected_store.bytes.blocks[0].bytes[0]=65;')
  add(f'inline_integer_{bits}',bits=bits,source='Operand::Integer {number:1145258561}',data=b'B',string_result=False)
  add(f'empty_result_{bits}',bits=bits,index=f'Operand::Integer {{number:{MAX}}}',data=b'')
 for mode,target,frame_setup,slot,extra in [
  ('local_new','Target::Local {local_index:0}','','4','expected_store.space.object_count=5;expected_frame.locals[0]=Binding::Cell {cell_id:4};'),
  ('local_cell','Target::Local {local_index:0}','frame.locals[0]=Binding::Cell {cell_id:2};','2','expected_frame.locals[0]=Binding::Cell {cell_id:2};'),
  ('argument_shared','Target::Argument {argument_index:0}','frame.arguments[0]=Binding::Shared {shared_id:0};','4','expected_store.space.object_count=5;expected_frame.arguments[0]=Binding::Cell {cell_id:4};')]:
  value='Value::String {string_storage:StringStorage::Owned {string_owner:'+slot+'}}'
  expected=extra+'expected_store.space.objects['+slot+'].value='+value+';expected_store.bytes.blocks['+slot+']=ByteBlock {initialized:true,length:1};expected_store.bytes.blocks['+slot+'].bytes[0]=65;'
  add(mode,target=target,frame_setup=frame_setup,expected=expected)
 for ref in ['RefOf','Index']:
  add('argument_'+ref.lower(),setup='store.space.objects[2].value=Value::Reference {kind:ReferenceKind::'+ref+',object_id:1};',frame_setup='frame.arguments[0]=Binding::Shared {shared_id:2};',target='Target::Argument {argument_index:0}',expected='expected_store.space.objects[1].value=Value::String {string_storage:StringStorage::Owned {string_owner:1}};expected_store.bytes.blocks[1]=ByteBlock {initialized:true,length:1};expected_store.bytes.blocks[1].bytes[0]=65;')
 for kind in ['Named','Local','Arg']:
  add('transparent_'+kind.lower(),setup=f'store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:2}};store.space.objects[2].value=Value::String {{string_storage:StringStorage::Source {{string_source:Span {{unit:7,end:2}}}}}};')
 for kind in ['RefOf','Index']:
  add('explicit_'+kind.lower(),setup=f'store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:2}};',error='UnsupportedValue')
 add('cycle',setup='store.space.objects[0].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};',error='ReferenceCycle')
 add('full_arena',setup='store.space.object_count=64;',error='Capacity')
 add('count_max',setup=f'store.space.object_count={MAX};',error='InvalidState')
 add('object_max',source=f'Operand::Object {{object_id:{MAX}}}',error='InvalidState')
 add('uninitialized',source='Operand::Uninitialized',error='Uninitialized')
 add('invalid_target',target=f'Target::Named {{object_id:{MAX}}}',error='MissingObject')
 add('target_fresh_slot',target='Target::Named {object_id:3}',error='MissingObject')
 add('local_cell_fresh_slot',target='Target::Local {local_index:0}',frame_setup='frame.locals[0]=Binding::Cell {cell_id:3};',error='MissingObject')
 add('argument_fresh_slot',target='Target::Argument {argument_index:0}',frame_setup='frame.arguments[0]=Binding::Shared {shared_id:3};',error='InvalidState')
 add('argument_referent_fresh_slot',target='Target::Argument {argument_index:0}',setup='store.space.objects[2].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:3};',frame_setup='frame.arguments[0]=Binding::Shared {shared_id:2};',error='MissingObject')
 add('debug',target='Target::Debug',error='UnresolvedService')
 add('invalid_local',target='Target::Local {local_index:8}',error='InvalidTarget')
 add('invalid_argument',target='Target::Argument {argument_index:7}',error='InvalidTarget')
 add('parent_full',frame_setup='frame.operations[0].count=1;frame.operations[0].operands[0]=Operand::Integer {number:99};',error='InvalidState')
 add('parent_count_max',frame_setup=f'frame.operation_count={MAX};',error='InvalidState')
 add('index_uninitialized',index='Operand::Uninitialized',error='Uninitialized')
 add('length_uninitialized',length='Operand::Uninitialized',error='Uninitialized')
 add('index_precedence',source='Operand::Uninitialized',index=f'Operand::Object {{object_id:{MAX}}}',error='InvalidState')
 add('malformed_source_before_empty',setup='store.space.objects[0].value=Value::String {string_storage:StringStorage::Owned {string_owner:0}};store.bytes.blocks[0]=ByteBlock {initialized:true,length:2};store.bytes.blocks[0].bytes[0]=65;store.bytes.blocks[0].bytes[1]=255;',index=f'Operand::Integer {{number:{MAX}}}',error='Encoding')
 add('wrong_owner',setup='store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:1}};',error='InvalidState')
 return rows

def render_rows(group,rows):
 if group in ['execution','generic','to_integer']:
  source,names=ns.base.render(rows)
  return source.replace(f'row.number={MAX+1};','row.number=0;'),names
 if group=='integer':
  old=ns.base.old;first=old.render(rows[0],False)
  source=first[:first.index('machine test_result()')]+'data Suite {}\n';names=[]
  for row in rows:
   rendered=old.render(row,False);body=rendered[rendered.index('machine test_result()'):rendered.index('data Main {}')]
   for control in [False,True]:
    value=body
    if control:
     numeric=row['expected'] is not None and row['error']=='Success'
     marker=f'result.value.number == {row["expected"]}' if numeric else f'result.outcome == ExecutionOutcome::{row["error"]}'
     replacement=f'result.value.number == {(row["expected"]+1)&((1<<row["bits"])-1)}' if numeric else f'result.outcome == ExecutionOutcome::{"BadEncoding" if row["error"]=="Success" else "Success"}'
     assert value.count(marker)==1;value=value.replace(marker,replacement)
    name='Suite::'+row['name']+('_control' if control else '_positive');names.append(name+'='+str(int(control)));source+=value.replace('machine test_result()',f'machine {name}(&mut self)',1)
  return source,names
 if group=='pipeline':
  source=pipeline.IMPORTS+'data Suite {}\n';names=[]
  for row in rows:
   for control in [False,True]:
    name='Suite::'+row['name']+('_control' if control else '_positive');names.append(name+'='+str(int(control)));source+=pipeline.render(row,control,name)
  return source,names
 source,names=ns.render_bridge(rows)
 source+='\nuse execution::mid_execution::retire_mid;\n'
 source=source.replace('let mut expected_store:ObjectStore=', 'frame.operation_count=1;frame.operations[0]=Operation {opcode:0xa4,value_arity:1};let mut expected_store:ObjectStore=')
 for row in rows:
  old=f'write_generic_target(&input,&mut store,&mut frame,{row["target"]},{row["operand"]},false)'
  new=f'retire_mid(&input,&mut store,&mut frame,{row["operand"]},{row["index"]},{row["length"]},{row["target"]})'
  # Function body scoped replacements avoid collisions between equal source/target rows.
  for suffix in ['positive','control']:
   start=source.index('machine Suite::'+row['name']+'_'+suffix+'(')
   end=source.find('\nmachine Suite::',start+1)
   end=len(source) if end<0 else end
   body=source[start:end].replace(old,new).replace('let mut expected_store:ObjectStore=',row['frame_setup']+'let mut expected_store:ObjectStore=')
   source=source[:start]+body+source[end:]
 return source,names

def render(group,match=''):
 rows=execution_cases() if group=='execution' else bridge_cases()
 rows=[r for r in rows if not match or r['name'] in match.split(',')];assert rows
 source,names=render_rows(group,rows);return rows,source,names


def load_fixture(label,path):
 spec=importlib.util.spec_from_file_location(label,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
pipeline=load_fixture('mid_pipeline',HERE.parent.parent/'pipeline/fixtures.py')
to_integer=load_fixture('mid_to_integer',HERE.parent/'to-integer-execution/fixtures.py')
def rows(group):
 return {'execution':execution_cases,'bridge':bridge_cases,'integer':ns.base.old.cases,'generic':ns.base.cases,'pipeline':pipeline.cases,'to_integer':to_integer.execution_cases}[group]()

if __name__=='__main__':
 for group in ['execution','bridge']:
  rows,source,names=render(group);(HERE/(group+'-cases.json')).write_text(json.dumps(rows,indent=2)+'\n');print(group,len(rows),'pairs')
