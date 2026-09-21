"""Independent primary conversion oracle and exact canonical-store deltas."""
from collections import Counter
MAX=(1<<64)-1
KINDS=['Named','RefOf','Local','Arg','Index','Unresolved']
def oracle(target,source,number,data,bits,extent):
 number &= (1<<bits)-1
 if target=='Integer':
  if source=='Integer':return dict(number=number)
  if not data:return dict(error='Empty')
  if source=='Buffer':return dict(number=int.from_bytes(data[:bits//8],'little'))
  prefix=''
  for ch in data[:bits//4]:
   if chr(ch)not in'0123456789abcdefABCDEF':break
   prefix+=chr(ch)
  return dict(number=int(prefix,16)if prefix else 0)
 if target=='String':
  result=(f'{number:0{bits//4}X}'.encode()if source=='Integer'else b' '.join(f'{b:02X}'.encode()for b in data)if source=='Buffer'else data)
  return dict(error='Capacity')if len(result)>256 else dict(bytes=list(result))
 if extent==0:return dict(error='Bounds')
 if source=='Buffer':return dict(error='UnsupportedValue')
 if source=='String'and not data:return dict(error='Empty')
 result=number.to_bytes(bits//8,'little')if source=='Integer'else data+b'\0'
 return dict(bytes=list(result[:extent]+bytes(max(0,extent-len(result)))))

def cases():
 rows=[]
 def add(name,target='Integer',source='Integer',number=0,data=b'',old=b'OLD',owned=True,source_owned=True,bits=64,error=None,**kw):
  row=dict(name=name,target=target,source=source,number=number,data=list(data),old=list(old),owned=owned,source_owned=source_owned,bits=bits,old_number=kw.pop('old_number',123),destination=kw.pop('destination',0),source_id=kw.pop('source_id',1),count=kw.pop('count',2),length=kw.pop('length',1024),unit=kw.pop('unit',7),mutation=kw.pop('mutation',''),default=kw.pop('default',False));assert not kw
  result=oracle(target,source,number,data,bits,len(old))if error is None else dict(error=error)
  row['expected']=result;rows.append(row)
 add('integer_reset',number=MAX,bits=32)
 for bits in [32,64]:
  for n in [0,1<<32,MAX]:add(f'int_int_{bits}_{n}',number=n,bits=bits)
  for owned in [False,True]:
   for n in [0,1,4,8,9,256]:add(f'int_buf_{bits}_{owned}_{n}',source='Buffer',data=bytes([150])*n,source_owned=owned,bits=bits)
   for i,data in enumerate([b'',b'0xFF',b'F',b'12G',b' ',b'F'*17]):add(f'int_str_{bits}_{owned}_{i}',source='String',data=data,source_owned=owned,bits=bits)
  for owned in [False,True]:
   for n in [0,1<<32,MAX]:add(f'str_int_{bits}_{owned}_{n}',target='String',number=n,owned=owned,bits=bits)
   for src_owned in [False,True]:
    for n in [0,1,85,86]:add(f'str_buf_{bits}_{owned}_{src_owned}_{n}',target='String',source='Buffer',data=bytes([150])*n,owned=owned,source_owned=src_owned,bits=bits)
    for n in [0,3,256]:add(f'str_str_{bits}_{owned}_{src_owned}_{n}',target='String',source='String',data=b'A'*n,owned=owned,source_owned=src_owned,bits=bits)
   for extent in [1,5,9,256]:
    for n in [1<<32,MAX]:add(f'buf_int_{bits}_{owned}_{extent}_{n}',target='Buffer',old=b'Z'*extent,number=n,owned=owned,bits=bits)
 for owned in [False,True]:
  for src_owned in [False,True]:
   for extent in [1,256]:
    for n in [0,1,255,256]:add(f'buf_str_{owned}_{src_owned}_{extent}_{n}',target='Buffer',source='String',data=b'A'*n,old=b'Z'*extent,owned=owned,source_owned=src_owned)
   for n in [0,1,256]:add(f'buf_buf_{owned}_{src_owned}_{n}',target='Buffer',source='Buffer',data=b'A'*n,owned=owned,source_owned=src_owned)
 for bits in [32,64]:
  add('integer_self_'+str(bits),number=MAX,bits=bits,old_number=MAX,source_id=0)
 for owned in [False,True]:
  for n in [0,3,256]:add('string_self_'+('owned'if owned else'source')+(''if n==3 else'_'+str(n)),target='String',source='String',data=b'A'*n,old=b'A'*n,owned=owned,source_id=0)
  add('buffer_self_'+str(owned),target='Buffer',source='Buffer',data=b'OLD',old=b'OLD',owned=owned,source_id=0)
  add('empty_target_String_'+str(owned),target='String',number=MAX,old=b'',owned=owned)
  for src in ['Integer','Buffer','String']:add('empty_target_Buffer_'+str(owned)+'_'+src,target='Buffer',source=src,data=b'ABC',old=b'',owned=owned)
 add('source_shared_span',target='String',source='String',old=b'XYZ',data=b'XYZ',owned=False,source_owned=False,mutation='s.space.objects[1].value=s.space.objects[0].value;')
 add('named_alias_identity',number=55,mutation='s.space.entries[1].object=0;s.space.entries[1].alias=true;')
 add('last_slot',number=MAX,destination=63,count=64)
 add('entry_metadata_ignored',number=55,mutation='s.space.count=18446744073709551615;')
 add('integer_unused_source_metadata',number=MAX,length=MAX,unit=MAX)
 add('owned_unused_source_metadata',target='String',source='String',data=b'ABC',length=MAX,unit=MAX)
 for count in [0,65,1<<63,MAX]:add('count_'+str(count),count=count,error='InvalidState')
 for id in [2,64,1<<63,MAX]:
  add('destination_'+str(id),destination=id,error='InvalidState')
  add('source_'+str(id),source_id=id,error='InvalidState')
 for kind in KINDS:
  add('destination_ref_'+kind,mutation='s.space.objects[0].value=Value::Reference {kind:ReferenceKind::'+kind+',object_id:1};',error='UnsupportedValue')
  add('source_ref_'+kind,mutation='s.space.objects[1].value=Value::Reference {kind:ReferenceKind::'+kind+',object_id:0};',error='UnsupportedValue')
 for label,value in [('Uninitialized','Value::Uninitialized'),('Package','Value::Package {first:0,count:0}'),('NameReference','Value::NameReference {name:Path {},scope:Path {}}'),('BufferField','Value::BufferField {backing_object:1,bit_length:8}'),('Method','Value::Method {flags:0,body:Span {}}'),('OperationRegion','Value::OperationRegion {space:0,base:0,length:1,scope:Path {}}'),('Event','Value::Event')]:
  add('destination_'+label,mutation=f's.space.objects[0].value={value};',error='UnsupportedValue')
  add('source_'+label,mutation=f's.space.objects[1].value={value};',error='UnsupportedValue')
 for target in ['String','Buffer']:
  for label,change,error in [('owner',f's.space.objects[0].value=Value::{target} {{{"string_storage:StringStorage::Owned {string_owner:1}"if target=="String"else"buffer_storage:BufferStorage::Owned {buffer_owner:1}"}}};','InvalidState'),('uninitialized','s.bytes.blocks[0].initialized=false;','InvalidState'),('capacity','s.bytes.blocks[0].length=257;','Capacity'),('max','s.bytes.blocks[0].length=18446744073709551615;','Capacity')]:
   add('destination_'+target+'_'+label,target=target,mutation=change,error=error)
   add('destination_before_source_'+target+'_'+label,target=target,mutation=change,source_id=MAX,error=error)
  add('destination_'+target+'_span',target=target,owned=False,length=0,error='Bounds')
  add('destination_'+target+'_unit',target=target,owned=False,unit=8,error='Bounds')
  add('source_'+target+'_owner',source=target,data=b'A',mutation=f's.space.objects[1].value=Value::{target} {{{"string_storage:StringStorage::Owned {string_owner:0}"if target=="String"else"buffer_storage:BufferStorage::Owned {buffer_owner:0}"}}};',error='InvalidState')
  add('source_'+target+'_uninitialized',source=target,data=b'A',mutation='s.bytes.blocks[1].initialized=false;',error='InvalidState')
  add('source_'+target+'_max',source=target,data=b'A',mutation='s.bytes.blocks[1].length=18446744073709551615;',error='Capacity')
 for owned in [False,True]:
  for data in [b'A\0',b'A\x80',b'A'*255+b'\0']:
   key=str(owned)+'_'+data.hex()[:8]+'_'+str(len(data))
   add('destination_encoding_'+key,target='String',old=data,owned=owned,error='Encoding')
   add('source_encoding_'+key,source='String',data=data,source_owned=owned,error='Encoding')
 add('destination_before_source',target='String',old=b'A\0',source_id=MAX,error='Encoding')
 add('empty_buffer_bad_source_id',target='Buffer',old=b'',source_id=MAX,error='InvalidState')
 add('empty_buffer_bad_source_encoding',target='Buffer',source='String',old=b'',data=b'A\0',error='Bounds')
 add('empty_string_bad_owner_before_Empty',target='Buffer',source='String',data=b'',mutation='s.space.objects[1].value=Value::String {string_storage:StringStorage::Owned {string_owner:0}};',error='InvalidState')
 add('destination_unsupported_before_bad_source',source_id=MAX,mutation='s.space.objects[0].value=Value::Device;',error='UnsupportedValue')
 add('default_failure',default=True,error='InvalidState')
 assert len({r['name']for r in rows})==len(rows)
 return rows

def fill(place,data,source=False,start=0):
 if not data:return ''
 value=Counter(data).most_common(1)[0][0]
 text=f'fill_{"input"if source else"bytes"}(&mut {place},{start},{start+len(data)},{value});'
 return text+''.join(f'{place}[{start+i}]={b};'for i,b in enumerate(data)if b!=value)
def value(kind,owned,at,data):
 field='string'if kind=='String'else'buffer';storage=kind+'Storage'
 payload=f'{storage}::Owned {{{field}_owner:{at}}}'if owned else f'{storage}::Source {{'+(''if kind=='String'else f'declared_size:{len(data)},')+f'{field}_'+('source'if kind=='String'else'initializer')+f':Span {{unit:7,start:{at*512},end:{at*512+len(data)}}}'+'}'
 return f'Value::{kind} {{{field}_storage:{payload}}}'
def setup(kind,number,data,owned,at):
 if kind=='Integer':return f's.space.objects[{at}].value=Value::Integer {{number:{number}}};'
 text=f's.space.objects[{at}].value='+value(kind,owned,at,data)+';'
 if owned:text+=f's.bytes.blocks[{at}].initialized=true;s.bytes.blocks[{at}].length={len(data)};'+fill(f's.bytes.blocks[{at}].bytes',data)
 else:text+=fill('input',data,True,at*512)
 return text

def body(row,control=False):
 text='let mut input:[u8;1024];let mut s:ObjectStore=seed();'
 text+=setup(row['target'],row['old_number'],row['old'],row['owned'],0)
 text+=setup(row['source'],row['number'],row['data'],row['source_owned'],1)
 text+=f's.space.object_count={row["count"]};'+row['mutation']+'let mut expected:ObjectStore=s;'
 expected=row['expected'];error=expected.get('error');dst=row['destination']
 if not error:
  if row['target']=='Integer':text+=f'expected.space.objects[{dst}].value=Value::Integer {{number:{expected["number"]}}};expected.bytes.blocks[{dst}]=ByteBlock {{}};'
  else:
   data=expected['bytes'];text+=f'expected.space.objects[{dst}].value='+value(row['target'],True,dst,data)+f';expected.bytes.blocks[{dst}]=ByteBlock {{initialized:true,length:{len(data)}}};'+fill(f'expected.bytes.blocks[{dst}].bytes',data)
 want=f'StoreResult::Failure {{reason:ConversionFailure::{error}}}'if error else f'StoreResult::Stored {{object:{dst}}}'
 if control:
  if error:want='StoreResult::Stored {object:0}'
  else:
   mode=sum(row['name'].encode())%4
   if mode==0:text+=f'expected.bytes.blocks[{dst}].bytes[255]=expected.bytes.blocks[{dst}].bytes[255]^1;'
   elif mode==1:text+=f'expected.space.objects[{dst}].next=expected.space.objects[{dst}].next^1;'
   elif mode==2:text+=f'expected.space.objects[{dst}].value=Value::Device;'
   else:want=f'StoreResult::Stored {{object:{dst^1}}}'
 if row['default']:
  text+=f'let result:StoreResult;let same:bool=result_same(result,{want});transition same {{true -> (0) _ -> (1)}}';return text
 text+=f'let answer:i32=check(&input,{row["length"]},{row["unit"]},s,expected,{dst},{row["source_id"]},IntegerSize::{"FourBytes"if row["bits"]==32 else"EightBytes"},{want});answer'
 return text

IMPORTS='use aml::model::ObjectStore;\nuse aml::model::Namespace;\nuse aml::model::Value;\nuse aml::model::Object;\nuse aml::model::Entry;\nuse aml::model::Path;\nuse aml::model::Span;\nuse aml::model::ByteBlock;\nuse aml::model::StringStorage;\nuse aml::model::BufferStorage;\nuse aml::model::ReferenceKind;\nuse aml::model::Outcome;\nuse aml::model::Read;\nuse aml::named_value_store::store_value;\nuse aml::named_value_store::StoreResult;\nuse aml::object_conversions::ConversionFailure;\nuse integer_helpers::integers::IntegerSize;\n'
HELPERS='machine seed()->ObjectStore {let mut s:ObjectStore=ObjectStore {};s.space.count=2;s.space.object_count=2;let mut bytes:[u8;256];seed_bytes(&mut bytes,0,256);seed_slots(&mut s,bytes,0,64);seed_entries(&mut s,0,32);s}\nmachine seed_bytes(a:&mut [u8;256],i:u64,n:u64) terminates by(i,n)->Nat::BoundedDistance; {seed_byte(a,i);transition i<n {true -> seed_bytes(a,i+1,n) _ -> {}}}\nmachine seed_byte(a:&mut [u8;256],i:u64){transition i<256 {true -> put(a,i) _ -> {}} state put(a:&mut [u8;256],i:u64){a[i]=(i^165) as u8;}}\nmachine seed_slots(s:&mut ObjectStore,bytes:[u8;256],i:u64,n:u64) terminates by(i,n)->Nat::BoundedDistance; {seed_slot(s,bytes,i);transition i<n {true -> seed_slots(s,bytes,i+1,n) _ -> {}}}\nmachine seed_slot(s:&mut ObjectStore,bytes:[u8;256],i:u64){transition i<64 {true -> put(s,bytes,i) _ -> {}} state put(s:&mut ObjectStore,bytes:[u8;256],i:u64){s.space.objects[i]=Object {value:Value::Integer {number:1000+i},has_next:true,next:18446744073709551615-i};s.bytes.blocks[i]=ByteBlock {initialized:true,length:18446744073709551615-i,bytes:bytes};}}\nmachine seed_entries(s:&mut ObjectStore,i:u64,n:u64) terminates by(i,n)->Nat::BoundedDistance; {seed_entry(s,i);transition i<n {true -> seed_entries(s,i+1,n) _ -> {}}}\nmachine seed_entry(s:&mut ObjectStore,i:u64){transition i<32 {true -> put(s,i) _ -> {}} state put(s:&mut ObjectStore,i:u64){s.space.entries[i]=Entry {path:Path {parents:18446744073709551615,count:i},has_object:true,object:i,alias:true};s.space.entries[i].path.segments[15]=3735928559;}}\nmachine long_chain(s:&mut ObjectStore){s.space.object_count=63;s.space.objects[0].value=Value::Package {first:1,count:62};long_nodes(s,1,63);}\nmachine long_nodes(s:&mut ObjectStore,i:u64,n:u64) terminates by(i,n)->Nat::BoundedDistance; {long_node(s,i);transition i<n {true -> long_nodes(s,i+1,n) _ -> {}}}\nmachine long_node(s:&mut ObjectStore,i:u64){transition i>0 && i<63 {true -> put(s,i) _ -> {}} state put(s:&mut ObjectStore,i:u64){s.space.objects[i]=Object {value:Value::Integer {number:i},has_next:i<62,next:i+1};}}\nmachine span_same(a:Span,b:Span)->bool {a.unit==b.unit && a.start==b.start && a.end==b.end}\nmachine path_same(a:Path,b:Path)->bool {let same:bool=segments(a,b,0,16,true);same && a.absolute==b.absolute && a.parents==b.parents && a.count==b.count}\nmachine segments(a:Path,b:Path,i:u64,n:u64,good:bool) terminates by(i,n)->Nat::BoundedDistance; ->bool {let same:bool=segment(a,b,i);transition i<n {true -> segments(a,b,i+1,n,good && same) _ -> (good)}}\nmachine segment(a:Path,b:Path,i:u64)->bool {transition i<16 {true -> (a.segments[i]==b.segments[i]) _ -> (true)}}\nmachine string_same(a:StringStorage,b:StringStorage)->bool {transition a {StringStorage::Source {string_source} -> source(string_source,b) StringStorage::Owned {string_owner} -> owned(string_owner,b)} state source(a:Span,b:StringStorage)->bool {transition b {StringStorage::Source {string_source} -> compare(a,string_source) _ -> (false)}} state compare(a:Span,b:Span)->bool {let good:bool=span_same(a,b);good} state owned(a:u64,b:StringStorage)->bool {transition b {StringStorage::Owned {string_owner} -> (a==string_owner) _ -> (false)}}}\nmachine buffer_same(a:BufferStorage,b:BufferStorage)->bool {transition a {BufferStorage::Source {declared_size,buffer_initializer} -> source(declared_size,buffer_initializer,b) BufferStorage::Owned {buffer_owner} -> owned(buffer_owner,b)} state source(size:u64,span:Span,b:BufferStorage)->bool {transition b {BufferStorage::Source {declared_size,buffer_initializer} -> compare(size,span,declared_size,buffer_initializer) _ -> (false)}} state compare(a:u64,s:Span,b:u64,t:Span)->bool {let good:bool=span_same(s,t);good && a==b} state owned(a:u64,b:BufferStorage)->bool {transition b {BufferStorage::Owned {buffer_owner} -> (a==buffer_owner) _ -> (false)}}}\nmachine value_same(a:Value,b:Value)->bool {transition a {Value::Uninitialized -> uninitialized(b) Value::Integer {number} -> integer(number,b) Value::String {string_storage} -> string(string_storage,b) Value::Buffer {buffer_storage} -> buffer(buffer_storage,b) Value::BufferField {backing_object,bit_offset,bit_length} -> bufferfield(backing_object,bit_offset,bit_length,b) Value::Package {first,count} -> package(first,count,b) Value::NameReference {name,scope} -> namereference(name,scope,b) Value::Reference {kind,object_id} -> reference(kind,object_id,b) Value::Method {flags,body} -> method(flags,body,b) Value::Device -> device(b) Value::Processor {id,address,length} -> processor(id,address,length,b) Value::PowerResource {system_level,order} -> powerresource(system_level,order,b) Value::ThermalZone -> thermalzone(b) Value::Mutex {sync_level} -> mutex(sync_level,b) Value::Event -> event(b) Value::OperationRegion {space,base,length,scope} -> operationregion(space,base,length,scope,b) }\nstate uninitialized(b:Value)->bool {transition b {Value::Uninitialized -> compare_uninitialized() _ -> (false)}}\nstate compare_uninitialized()->bool {true}\nstate integer(anumber:u64,b:Value)->bool {transition b {Value::Integer {number} -> compare_integer(anumber,number) _ -> (false)}}\nstate compare_integer(anumber:u64,number:u64)->bool {anumber==number}\nstate string(astring_storage:StringStorage,b:Value)->bool {transition b {Value::String {string_storage} -> compare_string(astring_storage,string_storage) _ -> (false)}}\nstate compare_string(astring_storage:StringStorage,string_storage:StringStorage)->bool {let cstring_storage:bool=string_same(astring_storage,string_storage);cstring_storage}\nstate buffer(abuffer_storage:BufferStorage,b:Value)->bool {transition b {Value::Buffer {buffer_storage} -> compare_buffer(abuffer_storage,buffer_storage) _ -> (false)}}\nstate compare_buffer(abuffer_storage:BufferStorage,buffer_storage:BufferStorage)->bool {let cbuffer_storage:bool=buffer_same(abuffer_storage,buffer_storage);cbuffer_storage}\nstate bufferfield(abacking_object:u64,abit_offset:u64,abit_length:u64,b:Value)->bool {transition b {Value::BufferField {backing_object,bit_offset,bit_length} -> compare_bufferfield(abacking_object,abit_offset,abit_length,backing_object,bit_offset,bit_length) _ -> (false)}}\nstate compare_bufferfield(abacking_object:u64,abit_offset:u64,abit_length:u64,backing_object:u64,bit_offset:u64,bit_length:u64)->bool {abacking_object==backing_object && abit_offset==bit_offset && abit_length==bit_length}\nstate package(afirst:u64,acount:u64,b:Value)->bool {transition b {Value::Package {first,count} -> compare_package(afirst,acount,first,count) _ -> (false)}}\nstate compare_package(afirst:u64,acount:u64,first:u64,count:u64)->bool {afirst==first && acount==count}\nstate namereference(aname:Path,ascope:Path,b:Value)->bool {transition b {Value::NameReference {name,scope} -> compare_namereference(aname,ascope,name,scope) _ -> (false)}}\nstate compare_namereference(aname:Path,ascope:Path,name:Path,scope:Path)->bool {let cname:bool=path_same(aname,name);let cscope:bool=path_same(ascope,scope);cname && cscope}\nstate reference(akind:ReferenceKind,aobject_id:u64,b:Value)->bool {transition b {Value::Reference {kind,object_id} -> compare_reference(akind,aobject_id,kind,object_id) _ -> (false)}}\nstate compare_reference(akind:ReferenceKind,aobject_id:u64,kind:ReferenceKind,object_id:u64)->bool {akind==kind && aobject_id==object_id}\nstate method(aflags:u8,abody:Span,b:Value)->bool {transition b {Value::Method {flags,body} -> compare_method(aflags,abody,flags,body) _ -> (false)}}\nstate compare_method(aflags:u8,abody:Span,flags:u8,body:Span)->bool {let cbody:bool=span_same(abody,body);aflags==flags && cbody}\nstate device(b:Value)->bool {transition b {Value::Device -> compare_device() _ -> (false)}}\nstate compare_device()->bool {true}\nstate processor(aid:u8,aaddress:u32,alength:u8,b:Value)->bool {transition b {Value::Processor {id,address,length} -> compare_processor(aid,aaddress,alength,id,address,length) _ -> (false)}}\nstate compare_processor(aid:u8,aaddress:u32,alength:u8,id:u8,address:u32,length:u8)->bool {aid==id && aaddress==address && alength==length}\nstate powerresource(asystem_level:u8,aorder:u16,b:Value)->bool {transition b {Value::PowerResource {system_level,order} -> compare_powerresource(asystem_level,aorder,system_level,order) _ -> (false)}}\nstate compare_powerresource(asystem_level:u8,aorder:u16,system_level:u8,order:u16)->bool {asystem_level==system_level && aorder==order}\nstate thermalzone(b:Value)->bool {transition b {Value::ThermalZone -> compare_thermalzone() _ -> (false)}}\nstate compare_thermalzone()->bool {true}\nstate mutex(async_level:u8,b:Value)->bool {transition b {Value::Mutex {sync_level} -> compare_mutex(async_level,sync_level) _ -> (false)}}\nstate compare_mutex(async_level:u8,sync_level:u8)->bool {async_level==sync_level}\nstate event(b:Value)->bool {transition b {Value::Event -> compare_event() _ -> (false)}}\nstate compare_event()->bool {true}\nstate operationregion(aspace:u8,abase:u64,alength:u64,ascope:Path,b:Value)->bool {transition b {Value::OperationRegion {space,base,length,scope} -> compare_operationregion(aspace,abase,alength,ascope,space,base,length,scope) _ -> (false)}}\nstate compare_operationregion(aspace:u8,abase:u64,alength:u64,ascope:Path,space:u8,base:u64,length:u64,scope:Path)->bool {let cscope:bool=path_same(ascope,scope);aspace==space && abase==base && alength==length && cscope}\n}\nmachine object_same(a:Object,b:Object)->bool {let same:bool=value_same(a.value,b.value);same && a.has_next==b.has_next && a.next==b.next}\nmachine entry_same(a:Entry,b:Entry)->bool {let same:bool=path_same(a.path,b.path);same && a.has_level==b.has_level && a.has_object==b.has_object && a.level==b.level && a.object==b.object && a.alias==b.alias}\nmachine entries(a:&ObjectStore,b:&ObjectStore,i:u64,n:u64,good:bool) terminates by(i,n)->Nat::BoundedDistance; ->bool {let same:bool=entry_equal_at(a,b,i);transition i<n {true -> entries(a,b,i+1,n,good && same) _ -> (good)}}\nmachine entry_equal_at(a:&ObjectStore,b:&ObjectStore,i:u64)->bool {transition i<32 {true -> compare(a.space.entries[i],b.space.entries[i]) _ -> (true)} state compare(a:Entry,b:Entry)->bool {let same:bool=entry_same(a,b);same}}\nmachine objects(a:&ObjectStore,b:&ObjectStore,i:u64,n:u64,good:bool) terminates by(i,n)->Nat::BoundedDistance; ->bool {let same:bool=slot(a,b,i);transition i<n {true -> objects(a,b,i+1,n,good && same) _ -> (good)}}\nmachine slot(a:&ObjectStore,b:&ObjectStore,i:u64)->bool {transition i<64 {true -> compare(a.space.objects[i],b.space.objects[i],a.bytes.blocks[i],b.bytes.blocks[i]) _ -> (true)} state compare(a:Object,b:Object,x:ByteBlock,y:ByteBlock)->bool {let same:bool=object_same(a,b);let bytes:bool=block_bytes(&x.bytes,&y.bytes,0,256,true);same && bytes && x.initialized==y.initialized && x.length==y.length}}\nmachine block_bytes(a:&[u8;256],b:&[u8;256],i:u64,n:u64,good:bool) terminates by(i,n)->Nat::BoundedDistance; ->bool {let same:bool=block_byte(a,b,i);transition i<n {true -> block_bytes(a,b,i+1,n,good && same) _ -> (good)}}\nmachine block_byte(a:&[u8;256],b:&[u8;256],i:u64)->bool {transition i<256 {true -> (a[i]==b[i]) _ -> (true)}}\n\nmachine fill_bytes(a:&mut [u8;256],i:u64,n:u64,value:u8) terminates by(i,n)->Nat::BoundedDistance; {fill_byte(a,i,n,value);transition i<n {true -> fill_bytes(a,i+1,n,value) _ -> {}}}\nmachine fill_byte(a:&mut [u8;256],i:u64,n:u64,value:u8){transition i<n && i<256 {true -> put(a,i,value) _ -> {}}state put(a:&mut [u8;256],i:u64,value:u8){a[i]=value;}}\nmachine fill_input(a:&mut [u8;1024],i:u64,n:u64,value:u8) terminates by(i,n)->Nat::BoundedDistance; {fill_input_byte(a,i,n,value);transition i<n {true -> fill_input(a,i+1,n,value) _ -> {}}}\nmachine fill_input_byte(a:&mut [u8;1024],i:u64,n:u64,value:u8){transition i<n && i<1024 {true -> put(a,i,value) _ -> {}}state put(a:&mut [u8;1024],i:u64,value:u8){a[i]=value;}}\nmachine result_same(a:StoreResult,b:StoreResult)->bool {transition a {StoreResult::Failure {reason} -> failed(reason,b) StoreResult::Stored {object} -> stored(object,b)} state failed(a:ConversionFailure,b:StoreResult)->bool {transition b {StoreResult::Failure {reason} -> (a==reason) _ -> (false)}} state stored(a:u64,b:StoreResult)->bool {transition b {StoreResult::Stored {object} -> (a==object) _ -> (false)}}}\nmachine check(input:&[u8;1024],length:u64,unit:u64,store:ObjectStore,expected:ObjectStore,destination:u64,source:u64,size:IntegerSize,want:StoreResult)->i32 {\n let mut actual:ObjectStore=store;let result:StoreResult=store_value(input,length,unit,&mut actual,destination,source,size);\n let e:bool=entries(&actual,&expected,0,32,true);let o:bool=objects(&actual,&expected,0,64,true);let r:bool=result_same(result,want);\n transition e && o && r && actual.space.count==expected.space.count && actual.space.object_count==expected.space.object_count {true -> (0) _ -> (1)}\n}\nmachine const_slots(a:&ObjectStore,b:&ObjectStore,i:u64,n:u64,good:bool) terminates by(i,n)->Nat::BoundedDistance; ->bool {let same:bool=const_slot(a,b,i);transition i<n {true -> const_slots(a,b,i+1,n,good && same) _ -> (good)}}\nmachine const_slot(a:&ObjectStore,b:&ObjectStore,i:u64)->bool {transition i<64 {true -> compare(a.space.objects[i],b.space.objects[i],a.bytes.blocks[i],b.bytes.blocks[i]) _ -> (true)} state compare(a:Object,b:Object,x:ByteBlock,y:ByteBlock)->bool {let same:bool=object_same(a,b);same && x.initialized==y.initialized && x.length==y.length && x.bytes[0]==y.bytes[0] && x.bytes[255]==y.bytes[255]}}\nmachine const_check(input:&[u8;1024],length:u64,unit:u64,store:ObjectStore,expected:ObjectStore,destination:u64,source:u64,size:IntegerSize,want:StoreResult)->i32 {\n let mut actual:ObjectStore=store;let result:StoreResult=store_value(input,length,unit,&mut actual,destination,source,size);\n let e:bool=entries(&actual,&expected,0,32,true);let o:bool=const_slots(&actual,&expected,0,64,true);let r:bool=result_same(result,want);\n transition e && o && r && actual.space.count==expected.space.count && actual.space.object_count==expected.space.object_count {true -> (0) _ -> (1)}\n}\n'
