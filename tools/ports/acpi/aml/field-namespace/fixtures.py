"""Original normal Field loader inputs and explicit complete-state expectations."""
import argparse
import json
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[4]
MAX=(1<<64)-1

def integer(n):
 if n in (0,1):return bytes([n])
 if n==MAX:return b'\xff'
 for width,tag in [(1,0x0a),(2,0x0b),(4,0x0c),(8,0x0e)]:
  if n < 1<<(width*8):return bytes([tag])+n.to_bytes(width,'little')
 raise ValueError(n)
def length(n):
 if n<64:return bytes([n])
 for following in range(1,4):
  if n<1<<(4+following*8):return bytes([(following<<6)|(n&15)])+bytes((n>>(4+8*i))&255 for i in range(following))
 raise ValueError(n)
def envelope(payload):
 for size in range(1,5):
  prefix=length(len(payload)+size)
  if len(prefix)==size:return prefix+payload
 raise ValueError(len(payload))
def path_wire(text):
 prefix=b''
 if text.startswith('\\'):prefix=b'\\';text=text[1:]
 while text.startswith('^'):prefix+=b'^';text=text[1:]
 parts=text.split('.')if text else[]
 return prefix+(b'\0'if not parts else b'\x2e'if len(parts)==2 else b'\x2f'+bytes([len(parts)])if len(parts)>2 else b'')+b''.join(p.ljust(4,'_').encode()for p in parts)
def path(text,absolute=None):
 root=text.startswith('\\');parents=0
 if root:text=text[1:]
 while text.startswith('^'):parents+=1;text=text[1:]
 parts=text.split('.')if text else[]
 return dict(absolute=root if absolute is None else absolute,parents=parents,segments=[int.from_bytes(p.ljust(4,'_').encode(),'little')for p in parts])
def span(start=0,end=0,unit=7):return dict(unit=unit,start=start,end=end)
def access(flags=1,attribute=0,mode=0,extended=False,access_length=0):
 return dict(flags=flags,kind=['Any','Byte','Word','DWord','QWord','Buffer'][flags&15],locked=bool(flags&16),update=['Preserve','WriteAsOnes','WriteAsZeros'][(flags>>5)&3],attribute=attribute,attribute_mode=mode,extended=extended,access_length=access_length)
def region(name='REG0',base=4096,size=256,space=0):return b'\x5b\x80'+path_wire(name)+bytes([space])+integer(base)+integer(size)
def normal(name='REG0',flags=1,elements=b'FLD0\x08'):
 return b'\x5b\x81'+envelope(path_wire(name)+bytes([flags])+elements)
def field_expected(wire,begin=0,region_name='REG0',scope='',flags=1,fields=None,unit=7):
 prefix_size=(wire[2]>>6)+1
 list_start=begin+2+prefix_size+len(path_wire(region_name))+1
 declaration=dict(kind='Field',scope=path(scope,True),primary_name=path(region_name),secondary_name=path(''),bank_value=0,flags=flags,source=span(begin,begin+len(wire),unit),field_list=span(list_start,begin+len(wire),unit))
 out=[]
 for item in fields or [dict(name='FLD0',offset=0,bits=8,start=0,end=5)]:
  out.append(dict(kind='field',region=item.get('region',0),declaration=declaration,field=dict(name=path('.'.join(filter(None,[scope,item['name']])),True),bit_offset=item['offset'],bit_length=item['bits'],access=item.get('access',access(flags)),connection=item.get('connection',dict(kind='None')),source=span(list_start+item['start'],list_start+item['end'],unit))))
 return out

def base(objects=1,entries=None):
 return dict(objects=objects,entries=entries or [dict(path='',level=True),dict(path='REG0',object=0)],values={0:dict(kind='region',base=4096,length=256,space=0,scope='')})
def cases():
 rows=[]
 def add(name,wire,expected=None,initial=None,error='Success',terms=64,values=64,definitions=False):
  rows.append(dict(name=name,wire=wire.hex(),expected=expected or [],initial=initial or base(),error=error,terms=terms,values=values,definitions=definitions))
 def field(path_name,value):return dict(path=path_name,value=value)
 raw=normal();one=field_expected(raw)[0]
 add('normal_field',raw,[field('FLD0',one)])
 add('zero_width_metadata',normal(elements=b'ZERO\0'),[field('ZERO',field_expected(normal(elements=b'ZERO\0'),fields=[dict(name='ZERO',offset=0,bits=0,start=0,end=5)])[0])])
 raw=normal(flags=0x31,elements=b'F000\x05\0\x03F001\x08\x01\x42\x77F002\x10\x03\x04\x0e\x20F003\x40\x04')
 fields=[dict(name='F000',offset=0,bits=5,start=0,end=5),dict(name='F001',offset=8,bits=8,start=7,end=12),dict(name='F002',offset=16,bits=16,start=15,end=20,access=access(0x32,0x77,1)),dict(name='F003',offset=32,bits=64,start=24,end=30,access=access(0x34,0x0e,0,True,0x20))]
 add('offset_access_and_extended',raw,[field(f['name'],v)for f,v in zip(fields,field_expected(raw,flags=0x31,fields=fields))])
 raw=normal(elements=b'\x02CON0A000\x08\x02\x11'+envelope(integer(2)+b'\xaa\xbb')+b'A001\x10')
 liststart=8
 fields=[dict(name='A000',offset=0,bits=8,start=5,end=10,connection=dict(kind='Name',connection_name=path('CON0'))),dict(name='A001',offset=8,bits=16,start=17,end=22,connection=dict(kind='Buffer',buffer_encoding=span(liststart+11,liststart+17),size_known=True,declared_size=2,initializer=span(liststart+15,liststart+17)))]
 # Prefix is calculated independently from the known declaration's byte width.
 add('connection_name_then_buffer',raw,[field(f['name'],v)for f,v in zip(fields,field_expected(raw,fields=fields))])
 raw=normal(elements=b'\x02\x11'+envelope(b'\x60\xaa')+b'DYN0\x08')
 fields=[dict(name='DYN0',offset=0,bits=8,start=5,end=10,connection=dict(kind='Buffer',buffer_encoding=span(9,13),size_known=False,declared_size=0,initializer=span(0,0,0)))]
 add('connection_deferred_buffer',raw,[field('DYN0',field_expected(raw,fields=fields)[0])])
 # Existing level gives the surrounding Scope its canonical declaration path.
 initial=base(entries=[dict(path='',level=True),dict(path='REG0',object=0),dict(path='DEV0',level=True,level_kind='Device')])
 fieldraw=normal();raw=b'\x10'+envelope(path_wire('DEV0')+fieldraw);start=len(raw)-len(fieldraw)
 add('ancestor_region_scope',raw,[field('DEV0.FLD0',field_expected(fieldraw,begin=start,scope='DEV0')[0])],initial)
 for label,name in [('absolute_region','\\REG0'),('parent_region','^REG0')]:
  fieldraw=normal(name);raw=b'\x10'+envelope(path_wire('DEV0')+fieldraw);start=len(raw)-len(fieldraw)
  add(label,raw,[field('DEV0.FLD0',field_expected(fieldraw,begin=start,region_name=name,scope='DEV0')[0])],initial)
 alias=base(entries=[dict(path='',level=True),dict(path='REG0',object=0),dict(path='ALIS',object=0,alias=True)])
 raw=normal('ALIS');add('region_alias',raw,[field('FLD0',field_expected(raw,region_name='ALIS')[0])],alias)
 # Field rebind follows the existing loader policy; earlier aliases retain old IDs.
 initial=base(objects=2,entries=[dict(path='',level=True),dict(path='REG0',object=0),dict(path='FLD0',object=1),dict(path='FALS',object=1,alias=True)])
 initial['values'][1]=dict(kind='integer',number=77)
 raw=normal();add('field_rebinding_retains_alias',raw,[field('FLD0',field_expected(raw)[0])],initial)
 raw=normal(elements=b'SAME\x08SAME\x10')
 fs=[dict(name='SAME',offset=0,bits=8,start=0,end=5),dict(name='SAME',offset=8,bits=16,start=5,end=10)]
 add('duplicate_field_names_pin_policy',raw,[field('SAME',v)for v in field_expected(raw,fields=fs)])
 # A Field binding must preserve an existing namespace level at the same path.
 initial=base(entries=[dict(path='',level=True),dict(path='REG0',object=0),dict(path='FLD0',level=True,level_kind='Device')])
 raw=normal();add('field_binding_preserves_level',raw,[field('FLD0',field_expected(raw)[0])],initial)
 # Region identity is captured once even when the first named field replaces its name.
 raw=normal(elements=b'REG0\x08F001\x08')
 fs=[dict(name='REG0',offset=0,bits=8,start=0,end=5),dict(name='F001',offset=8,bits=8,start=5,end=10)]
 add('field_list_replaces_region_name',raw,[field(f['name'],v)for f,v in zip(fs,field_expected(raw,fields=fs))])
 first=normal();rebind=region(base=8192);second=normal('REG0',elements=b'NEW0\x08');raw=first+rebind+second
 a=field_expected(first)[0];b=field_expected(second,begin=len(first)+len(rebind),fields=[dict(name='NEW0',offset=0,bits=8,start=0,end=5,region=2)])[0]
 add('region_rebinding_retains_original_identity',raw,[field('FLD0',a),field('REG0',dict(kind='region',base=8192,length=256,space=0,scope='')),field('NEW0',b)])
 add('empty_field_list',normal(elements=b''),[])
 add('empty_list_zero_value_budget',normal(elements=b''),[],values=0)
 add('exact_loader_turn_budget',normal(),[field('FLD0',one)],terms=2)
 add('loader_turn_exhaustion_rolls_back',normal(),error='WorkLimit',terms=1)
 add('field_element_exhaustion',normal(elements=b'F000\x08F001\x08'),error='WorkLimit',values=1)
 add('late_malformed_element',normal(elements=b'F000\x08\x01\x02'),error='Truncated')
 add('invalid_flags',normal(flags=0x60),error='BadEncoding')
 add('invalid_field_name',normal(elements=b'1BAD\x08'),error='InvalidName')
 add('invalid_pkg_reserved_bits',b'\x5b\x81\x50\x00',error='BadEncoding')
 add('truncated_envelope',normal()[:-1],error='BadEncoding')
 add('missing_region',normal('MISS'),error='MissingObject')
 wrong=base();wrong['values'][0]=dict(kind='integer',number=9)
 add('wrong_region_kind',normal(),initial=wrong,error='UnsupportedSyntax')
 wrong=base();wrong['values'][0]=dict(kind='reference',reference_kind='RefOf',object_id=0)
 add('reference_not_region',normal(),initial=wrong,error='UnsupportedSyntax')
 add('later_opcode_failure_rolls_back',normal()+b'\x72',error='UnsupportedSyntax')
 add('index_field_stays_unsupported',b'\x5b\x86'+envelope(b'REG0REG0\x01FLD0\x08'),error='UnsupportedSyntax')
 add('bank_field_stays_unsupported',b'\x5b\x87'+envelope(b'REG0REG0\x01\x01FLD0\x08'),error='UnsupportedSyntax')
 full=base(objects=63)
 add('last_object_slot',normal(),[field('FLD0',one)],initial=full)
 add('object_exhaustion_atomic_declaration',normal(elements=b'F000\x08F001\x08'),initial=full,error='Capacity')
 add('objects_already_full',normal(),initial=base(objects=64),error='Capacity')
 fs=[dict(name=f'A{i:03}',offset=i,bits=1,start=i*5,end=i*5+5)for i in range(30)]
 raw=normal(elements=b''.join(f['name'].encode()+b'\x01'for f in fs))
 add('last_namespace_slot',raw,[field(f['name'],v)for f,v in zip(fs,field_expected(raw,fields=fs))])
 add('namespace_exhaustion_atomic_declaration',normal(elements=b''.join(f'A{i:03}'.encode()+b'\x01'for i in range(31))),error='Capacity')
 add('descriptor_capacity',normal(elements=b''.join(f'A{i:03}'.encode()+b'\x01'for i in range(33))),error='Capacity')
 # Method capture and an installed Field are both rolled back by a later bad term.
 method=b'\x14'+envelope(b'MAIN\0\xa4\x01')
 add('method_and_field_rollback',method+normal()+b'\x72',error='UnsupportedSyntax',definitions=True)
 # Logical field extent metadata is not an access plan or a region grant.
 raw=normal(elements=b'\0'+length(0xfffffff)+b'WIDE'+length(0xfffffff))
 fs=[dict(name='WIDE',offset=0xfffffff,bits=0xfffffff,start=5,end=13)]
 add('large_metadata_extent',raw,[field('WIDE',field_expected(raw,fields=fs)[0])])
 return rows

IMPORTS='''use aml::model::Outcome;
use aml::model::Namespace;
use aml::model::Entry;
use aml::model::Object;
use aml::model::Value;
use aml::model::ReferenceKind;
use aml::model::Path;
use aml::model::Span;
use aml::model::LevelKind;
use aml::model::LoadState;
use aml::model::ObservedLoad;
use aml::model::MethodDefinitions;
use aml::model::MethodDefinition;
use aml::loader::load_with_definitions;
use aml::field_model::Field;
use aml::field_model::Access;
use aml::field_model::AccessType;
use aml::field_model::UpdateRule;
use aml::field_model::Connection;
use aml::field_model::Declaration;
use aml::field_model::DeclarationKind;
'''

def omega(v):
 if isinstance(v,bool):return str(v).lower()
 if isinstance(v,int):return str(v)
 raise TypeError(v)
def span_expr(v):return 'Span {'+','.join(f'{k}:{v[k]}'for k in ['unit','start','end'])+'}'
def path_expr(v,bindings,name):
 bindings.append(f'let mut {name}:Path=Path {{absolute:{omega(v["absolute"])},parents:{v["parents"]},count:{len(v["segments"])}}};')
 for i,s in enumerate(v['segments']):bindings.append(f'{name}.segments[{i}]={s};')
 return name

def value_expr(v,bindings,key):
 kind=v['kind']
 if kind=='integer':return f'Value::Integer {{number:{v["number"]}}}'
 if kind=='region':
  scope=path_expr(path(v['scope'],True),bindings,key+'_scope')
  return f'Value::OperationRegion {{space:{v["space"]},base:{v["base"]},length:{v["length"]},scope:{scope}}}'
 if kind=='reference':return f'Value::Reference {{kind:ReferenceKind::{v["reference_kind"]},object_id:{v["object_id"]}}}'
 if kind=='field':
  d=v['declaration'];f=v['field'];a=f['access'];c=f['connection']
  scope=path_expr(d['scope'],bindings,key+'_scope');primary=path_expr(d['primary_name'],bindings,key+'_primary');secondary=path_expr(d['secondary_name'],bindings,key+'_secondary');name=path_expr(f['name'],bindings,key+'_name')
  declaration='Declaration {'+f'kind:DeclarationKind::{d["kind"]},scope:{scope},primary_name:{primary},secondary_name:{secondary},bank_value:{d["bank_value"]},flags:{d["flags"]},source:{span_expr(d["source"])},field_list:{span_expr(d["field_list"])}'+'}'
  acc='Access {'+','.join(f'{k}:{("AccessType::"+val)if k=="kind" else ("UpdateRule::"+val)if k=="update" else omega(val)}'for k,val in a.items())+'}'
  connection='Connection::'+c['kind']
  if c['kind']=='Name':connection+=' {connection_name:'+path_expr(c['connection_name'],bindings,key+'_connection')+'}'
  elif c['kind']=='Buffer':connection+=' {'+','.join(f'{k}:{span_expr(val)if isinstance(val,dict)else omega(val)}'for k,val in c.items()if k!='kind')+'}'
  descriptor='Field {'+f'name:{name},bit_offset:{f["bit_offset"]},bit_length:{f["bit_length"]},access:{acc},connection:{connection},source:{span_expr(f["source"])}'+'}'
  return 'Value::FieldUnit {'+f'region_object:{v["region"]},declaration:{declaration},field:{descriptor}'+'}'
 raise ValueError(kind)

def fixture_body(row,control=False):
 b=['let mut source:[u8;1024];']+[f'source[{i}]={v};'for i,v in enumerate(bytes.fromhex(row['wire']))if v]
 b+=['let mut initial:Namespace=initial_space();',f'initial.count={len(row["initial"]["entries"])};initial.object_count={row["initial"]["objects"]};']
 for i,e in enumerate(row['initial']['entries']):
  p=path_expr(path(e['path'],True),b,'initial_path_'+str(i))
  b.append(f'initial.entries[{i}]=Entry {{path:{p},has_level:{omega(e.get("level",False))},level:LevelKind::{e.get("level_kind","Scope")},has_object:{omega("object"in e)},object:{e.get("object",0)},alias:{omega(e.get("alias",False))}}};')
 for i,v in row['initial']['values'].items():b.append(f'initial.objects[{i}]=Object {{value:{value_expr(v,b,"initial_value_"+str(i))}}};')
 b.append('let mut expected:Namespace=initial;')
 entries=[e['path']for e in row['initial']['entries']];levels={e['path']:e.get('level_kind','Scope') for e in row['initial']['entries']if e.get('level',False)};count=row['initial']['objects']
 for item in row['expected']:
  value=value_expr(item['value'],b,'expected_'+str(count));p=path_expr(path(item['path'],True),b,'expected_path_'+str(count))
  if item['path'] in entries:index=entries.index(item['path'])
  else:index=len(entries);entries.append(item['path'])
  level=levels.get(item['path']);b.append(f'expected.objects[{count}]=Object {{value:{value}}};expected.entries[{index}]=Entry {{path:{p},has_level:{omega(level is not None)},level:LevelKind::{level or "Scope"},has_object:true,object:{count}}};');count+=1
 b.append(f'expected.count={len(entries)};expected.object_count={count};')
 b.append('let mut definitions:MethodDefinitions;definitions.entries[63]=MethodDefinition {present:true,object_id:63,flags:7,body:Span {unit:99,start:8,end:9}};')
 b.append(f'let observed:ObservedLoad=load_with_definitions(&source,{len(bytes.fromhex(row["wire"]))},7,initial,definitions,{row["terms"]},{row["values"]});')
 if control:b.append('expected.objects[63].next=expected.objects[63].next^1;')
 b.append(f'let result:i32=compare(&observed.load.space,&expected,&observed.definitions,&definitions,observed.load.outcome,Outcome::{row["error"]});result')
 return '\n'.join(b)

HELPERS='''machine initial_space()->Namespace {let mut s:Namespace;seed(&mut s,0,64);s}
machine seed(s:&mut Namespace,index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
{seed_one(s,index);transition index<count {true -> seed(s,index+1,count) _ -> {}}}
machine seed_one(s:&mut Namespace,index:u64) {transition index<64 {true -> put(s,index) _ -> {}}
 state put(s:&mut Namespace,index:u64) {s.objects[index]=Object {value:Value::Integer {number:1000+index},has_next:true,next:18446744073709551615-index};}}
machine paths(a:Path,b:Path)->bool {let all:bool=segments(a,b,0,16,true);a.absolute==b.absolute && a.parents==b.parents && a.count==b.count && all}
machine segments(a:Path,b:Path,index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let same:bool=segment(a,b,index);transition index<count {true -> segments(a,b,index+1,count,prior && same) _ -> (prior)}}
machine segment(a:Path,b:Path,index:u64)->bool {transition index<16 {true -> (a.segments[index]==b.segments[index]) _ -> (true)}}
machine spans(a:Span,b:Span)->bool {a.unit==b.unit && a.start==b.start && a.end==b.end}
machine accesses(a:Access,b:Access)->bool {a.flags==b.flags && a.kind==b.kind && a.locked==b.locked && a.update==b.update && a.attribute==b.attribute && a.attribute_mode==b.attribute_mode && a.extended==b.extended && a.access_length==b.access_length}
machine connections(a:Connection,b:Connection)->bool {transition a {
 Connection::None -> none(b) Connection::Name {connection_name} -> name(connection_name,b)
 Connection::Buffer {buffer_encoding,size_known,declared_size,initializer} -> buffer(buffer_encoding,size_known,declared_size,initializer,b)}
 state none(b:Connection)->bool {transition b {Connection::None -> (true) _ -> (false)}}
 state name(a:Path,b:Connection)->bool {transition b {Connection::Name {connection_name} -> name_same(a,connection_name) _ -> (false)}}
 state name_same(a:Path,b:Path)->bool {let result:bool=paths(a,b);result}
 state buffer(a:Span,known:bool,size:u64,bytes:Span,b:Connection)->bool {transition b {Connection::Buffer {buffer_encoding,size_known,declared_size,initializer} -> buffer_same(a,known,size,bytes,buffer_encoding,size_known,declared_size,initializer) _ -> (false)}}
 state buffer_same(a:Span,ak:bool,az:u64,ai:Span,b:Span,bk:bool,bz:u64,bi:Span)->bool {let encoding:bool=spans(a,b);let init:bool=spans(ai,bi);encoding && init && ak==bk && az==bz}}
machine declarations(a:Declaration,b:Declaration)->bool {let scope:bool=paths(a.scope,b.scope);let primary:bool=paths(a.primary_name,b.primary_name);let secondary:bool=paths(a.secondary_name,b.secondary_name);let source:bool=spans(a.source,b.source);let list:bool=spans(a.field_list,b.field_list);scope && primary && secondary && source && list && a.kind==b.kind && a.bank_value==b.bank_value && a.flags==b.flags}
machine fields(a:Field,b:Field)->bool {let name:bool=paths(a.name,b.name);let acc:bool=accesses(a.access,b.access);let connection:bool=connections(a.connection,b.connection);let source:bool=spans(a.source,b.source);name && acc && connection && source && a.bit_offset==b.bit_offset && a.bit_length==b.bit_length}
machine values(a:Value,b:Value)->bool {transition a {
 Value::Integer {number} -> integer(number,b)
 Value::OperationRegion {space,base,length,scope} -> region(space,base,length,scope,b)
 Value::FieldUnit {region_object,declaration,field} -> field(region_object,declaration,field,b)
 Value::Reference {kind,object_id} -> reference(kind,object_id,b)
 _ -> (false)}
 state integer(a:u64,b:Value)->bool {transition b {Value::Integer {number} -> (a==number) _ -> (false)}}
 state region(a:u8,base_a:u64,length_a:u64,scope_a:Path,b:Value)->bool {transition b {Value::OperationRegion {space,base,length,scope} -> region_same(a,base_a,length_a,scope_a,space,base,length,scope) _ -> (false)}}
 state region_same(a:u8,ab:u64,al:u64,ap:Path,b:u8,bb:u64,bl:u64,bp:Path)->bool {let same:bool=paths(ap,bp);a==b && ab==bb && al==bl && same}
 state field(a:u64,ad:Declaration,af:Field,b:Value)->bool {transition b {Value::FieldUnit {region_object,declaration,field} -> field_same(a,ad,af,region_object,declaration,field) _ -> (false)}}
 state field_same(a:u64,ad:Declaration,af:Field,b:u64,bd:Declaration,bf:Field)->bool {let ds:bool=declarations(ad,bd);let fs:bool=fields(af,bf);a==b && ds && fs}
 state reference(a:ReferenceKind,id:u64,b:Value)->bool {transition b {Value::Reference {kind,object_id} -> (a==kind && id==object_id) _ -> (false)}}}
machine objects(a:&Namespace,b:&Namespace,index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let same:bool=object_one(a,b,index);transition index<count {true -> objects(a,b,index+1,count,prior && same) _ -> (prior)}}
machine object_one(a:&Namespace,b:&Namespace,index:u64)->bool {transition index<64 {true -> one(a.objects[index],b.objects[index]) _ -> (true)} state one(a:Object,b:Object)->bool {let same:bool=values(a.value,b.value);same && a.has_next==b.has_next && a.next==b.next}}
machine entries(a:&Namespace,b:&Namespace,index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let same:bool=entry_one(a,b,index);transition index<count {true -> entries(a,b,index+1,count,prior && same) _ -> (prior)}}
machine entry_one(a:&Namespace,b:&Namespace,index:u64)->bool {transition index<32 {true -> one(a.entries[index],b.entries[index]) _ -> (true)} state one(a:Entry,b:Entry)->bool {let same:bool=paths(a.path,b.path);same && a.has_level==b.has_level && a.level==b.level && a.has_object==b.has_object && a.object==b.object && a.alias==b.alias}}
machine definitions(a:&MethodDefinitions,b:&MethodDefinitions,index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let same:bool=definition_one(a,b,index);transition index<count {true -> definitions(a,b,index+1,count,prior && same) _ -> (prior)}}
machine definition_one(a:&MethodDefinitions,b:&MethodDefinitions,index:u64)->bool {transition index<64 {true -> one(a.entries[index],b.entries[index]) _ -> (true)} state one(a:MethodDefinition,b:MethodDefinition)->bool {let scope:bool=paths(a.scope,b.scope);let body:bool=spans(a.body,b.body);scope && body && a.present==b.present && a.object_id==b.object_id && a.flags==b.flags}}
machine compare(a:&Namespace,b:&Namespace,ad:&MethodDefinitions,bd:&MethodDefinitions,actual:Outcome,expected:Outcome)->i32 {let es:bool=entries(a,b,0,32,true);let os:bool=objects(a,b,0,64,true);let ds:bool=definitions(ad,bd,0,64,true);transition es && os && ds && a.count==b.count && a.object_count==b.object_count && actual==expected {true -> (0) _ -> (1)}}
'''
def render(rows):
 text=IMPORTS+HELPERS+'data Suite {}\n';names=[]
 for row in rows:
  for control in [False,True]:
   name='Suite::'+row['name']+('_control'if control else'_positive');names.append(name+'='+str(int(control)))
   text+='machine '+name+'(&mut self)->i32 {\n'+fixture_body(row,control)+'\n}\n'
 return text,names
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args()
 rows=cases();source,names=render(rows)
 outputs={'main.omg':source,'selections.txt':'\n'.join(names)+'\n','cases.json':json.dumps(rows,indent=2,sort_keys=True)+'\n'}
 for name,text in outputs.items():
  if a.check:assert (HERE/name).read_text()==text,name
  else:(HERE/name).write_text(text)
 print(f'{len(rows)} independently authored normal Field loader scenario/control pairs')
