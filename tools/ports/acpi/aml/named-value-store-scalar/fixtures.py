"""Unchanged 305 object-source regressions plus scalar/admission composition."""
import copy,json
from pathlib import Path
import baseline_fixtures as old
MAX=old.MAX

def scalar_cases():
 rows=[]
 template=old.cases()[0]
 def add(name,target='Integer',number=MAX,bits=64,extent=3,owned=True,error=None,**kw):
  row=copy.deepcopy(template);row.update(name='scalar_'+name,target=target,number=number,bits=bits,old=list(b'Z'*extent),owned=owned,default=False,api='scalar');row.update(kw)
  row['expected']=old.oracle(target,'Integer',number,b'',bits,extent)if error is None else dict(error=error)
  rows.append(row)
 for bits in [32,64]:
  for n in [0,1<<32,MAX]:add(f'integer_{bits}_{n}',number=n,bits=bits)
  for owned in [False,True]:
   for extent in [0,3,256]:
    for n in [1<<32,MAX]:add(f'string_{bits}_{owned}_{extent}_{n}',target='String',number=n,bits=bits,extent=extent,owned=owned)
   for extent in [0,1,3,4,5,8,9,256]:
    for n in [1<<32,MAX]:add(f'buffer_{bits}_{owned}_{extent}_{n}',target='Buffer',number=n,bits=bits,extent=extent,owned=owned)
 add('slot0_only',count=1,mutation='s.space.objects[1].value=Value::Device;')
 add('last_slot_no_allocation',count=64,destination=63)
 add('named_alias',mutation='s.space.entries[1].object=0;s.space.entries[1].alias=true;')
 add('snapshot_original_value',old_number=MAX,number=MAX,bits=32,number_from_destination=True)
 add('inactive_bytes_ignored',length=MAX,unit=MAX)
 add('entry_count_ignored',mutation='s.space.count=18446744073709551615;')
 # Retain independent named-store destination failures unchanged, now with no source ID.
 failures=[r for r in old.cases()if r['name'].startswith(('count_','destination_'))and not r['name'].startswith('destination_before_source')and 'error'in r['expected']]
 for r in failures:
  row=copy.deepcopy(r);row['name']='scalar_'+r['name'];row['api']='scalar';rows.append(row)
 for name,target,mutation,error in [
  ('changed_kind','String','s.space.objects[0].value=Value::Device;','UnsupportedValue'),
  ('changed_extent','Buffer','s.bytes.blocks[0].length=18446744073709551615;','Capacity'),
  ('changed_owner','String','s.space.objects[0].value=Value::String {string_storage:StringStorage::Owned {string_owner:1}};','InvalidState')]:
  add(name,target=target,mutation=mutation,error=error,prior_admission=True)
 return rows

def admission_cases():
 rows=[]
 # Select all distinct malformed destination scenarios, and each valid type/storage/extent.
 source=old.cases();names=['integer_reset','empty_target_String_False','empty_target_String_True','empty_target_Buffer_False_Integer','empty_target_Buffer_True_Integer','string_self_owned','string_self_source','buf_int_64_True_256_18446744073709551615','buf_int_64_False_256_18446744073709551615','last_slot','entry_metadata_ignored','integer_unused_source_metadata']
 selected=[r for r in source if r['name']in names or (r['name'].startswith(('count_','destination_'))and not r['name'].startswith('destination_before_source')and'error'in r['expected'])]
 for r in selected:
  row=copy.deepcopy(r);row['name']='admission_'+r['name'];row['api']='admission'
  # Empty Buffer is an admitted destination; the scalar/source conversion profile excludes it later.
  row['admission_error']=r['expected'].get('error')if r['name'].startswith(('count_','destination_'))else None
  if not row['admission_error']:row['extent']=0 if row['target']=='Integer'else len(row['old'])
  rows.append(row)
 row=copy.deepcopy(source[0]);row.update(name='admission_default_failure',api='admission',default=True,admission_error='InvalidState');rows.append(row)
 return rows

def cases():
 baseline=old.cases();assert baseline==json.loads(Path(__file__).with_name('baseline_cases.json').read_text());scalars=scalar_cases();rechecks=[]
 for r in scalars:
  if r.get('prior_admission'):
   row=copy.deepcopy(r);row.update(name='object_after_admission_'+r['name'],api='object_after_admission');rechecks.append(row)
 rows=baseline+scalars+admission_cases()+rechecks;assert len(set(r['name']for r in rows))==len(rows);return rows

IMPORTS=old.IMPORTS+'use aml::named_value_store::store_integer;\nuse aml::named_value_store::admit_destination;\nuse aml::named_value_store::DestinationResult;\nuse aml::implicit_conversions::ConversionTarget;\n'
HELPERS=old.HELPERS
# The comparison scope is identical to the baseline, including complete store bytes.
start=old.HELPERS.index('machine check(');end=old.HELPERS.index('machine const_slots(')
scalar=old.HELPERS[start:end].replace('machine check(', 'machine check_scalar(').replace('store_value(input,length,unit,&mut actual,destination,source,size)','store_integer(input,length,unit,&mut actual,destination,source,size)')
start=old.HELPERS.index('machine const_check(')
scalar_const=old.HELPERS[start:].replace('machine const_check(', 'machine const_check_scalar(').replace('store_value(input,length,unit,&mut actual,destination,source,size)','store_integer(input,length,unit,&mut actual,destination,source,size)')
HELPERS+=scalar+scalar_const+'''
machine selected_number(value:Value)->u64 {transition value {Value::Integer {number} -> (number) _ -> (0)}}
machine destination_same(a:DestinationResult,b:DestinationResult)->bool {
 transition a {DestinationResult::Failure {reason} -> failure(reason,b) DestinationResult::Target {target,extent} -> target(target,extent,b)}
 state failure(a:ConversionFailure,b:DestinationResult)->bool {transition b {DestinationResult::Failure {reason} -> (a==reason) _ -> (false)}}
 state target(a:ConversionTarget,n:u64,b:DestinationResult)->bool {transition b {DestinationResult::Target {target,extent} -> (a==target && n==extent) _ -> (false)}}
}
machine check_admission(input:&[u8;1024],length:u64,unit:u64,store:ObjectStore,destination:u64,want:DestinationResult)->i32 {
 let expected:ObjectStore=store;let result:DestinationResult=admit_destination(input,length,unit,&store,destination);
 let e:bool=entries(&store,&expected,0,32,true);let o:bool=objects(&store,&expected,0,64,true);let r:bool=destination_same(result,want);
 transition e && o && r && store.space.count==expected.space.count && store.space.object_count==expected.space.object_count {true -> (0) _ -> (1)}
}
'''

def body(row,control=False):
 api=row.get('api')
 if api is None:return old.body(row,control)
 if api in ['scalar','object_after_admission']:
  clone=copy.deepcopy(row)
  if api=='scalar':clone['source_id']=row['number']
  text=old.body(clone,control)
  if api=='scalar':text=text.replace('=check(', '=check_scalar(')
  if row.get('number_from_destination'):
   text=text.replace('let mut expected:ObjectStore=s;','let snapshot:u64=selected_number(s.space.objects[0].value);let mut expected:ObjectStore=s;').replace(','+str(row['number'])+',IntegerSize',',snapshot,IntegerSize')
  if row.get('prior_admission'):
   marker=row['mutation'];prefix='let prior:DestinationResult=admit_destination(&input,'+str(row['length'])+','+str(row['unit'])+',&s,'+str(row['destination'])+');let admitted:bool=destination_same(prior,DestinationResult::Target {target:ConversionTarget::'+row['target']+',extent:'+str(len(row['old']))+'});'
   text=text.replace(marker,prefix+marker).removesuffix('answer')+'transition admitted {true -> (answer) _ -> (2)}'
  return text
 text='let mut input:[u8;1024];let mut s:ObjectStore=seed();'+old.setup(row['target'],row['old_number'],row['old'],row['owned'],0)+old.setup(row['source'],row['number'],row['data'],row['source_owned'],1)+f's.space.object_count={row["count"]};'+row['mutation']
 error=row['admission_error']
 want='DestinationResult::Failure {reason:ConversionFailure::'+error+'}'if error else'DestinationResult::Target {target:ConversionTarget::'+row['target']+',extent:'+str(row['extent']+(1 if control else 0))+'}'
 if error and control:want='DestinationResult::Target {target:ConversionTarget::Integer,extent:0}'
 if row['default']:return text+'let result:DestinationResult;let good:bool=destination_same(result,'+want+');transition good {true -> (0) _ -> (1)}'
 return text+f'let answer:i32=check_admission(&input,{row["length"]},{row["unit"]},s,{row["destination"]},{want});answer'
