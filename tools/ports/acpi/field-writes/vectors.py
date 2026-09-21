"""Original detached single-pass write vectors and independent bit-interval oracle."""
import geometry_vectors as geometry
MAX=(1<<64)-1

def make(row,pattern=0,missing=(),supplied=None,payload_length=None,poison=False,payload_override=None):
 row=dict(row,pattern=pattern,poison=poison);plan=row.pop('expected');row['geometry']=plan
 length=(row['length']+7)//8 if row['length']<=2048 else 0
 payload=[(i*91+0xa6)&255 if pattern==0 else 0 if pattern==1 else 255 for i in range(length)]
 if payload_override is not None:payload=list(payload_override);assert len(payload)==length
 previous=[];expected={}
 if 'error' in plan:count=0;expected=dict(error='Geometry',cause=plan['error'])
 else:
  count=plan['count'];records=[];logical=int.from_bytes(bytes(payload),'little')
  for index,chunk in enumerate(plan['chunks']):
   width=chunk['width'];wm=(1<<(8*width))-1
   old=sum((((chunk['offset']+i)*37+11)&255)<<(8*i)for i in range(width))
   previous.append(None if index in missing else old | (MAX^wm if poison else 0))
   update=(row['flags']>>5)&3
   if chunk['partial'] and update==0 and previous[-1] is None and not expected:expected=dict(error='Chunk',cause='NeedsPrevious',index=index)
   base=old if update==0 and chunk['partial'] else MAX if update==1 and chunk['partial'] else 0
   value=(logical>>chunk['field_bit'])&((1<<chunk['bit_count'])-1)
   records.append(dict(offset=chunk['offset'],width=width,value=((base&~chunk['mask'])|(value<<chunk['native_bit']))&wm))
  if not expected:expected=dict(records=records,count=count,locked=bool(row['flags']&16))
 supplied=count if supplied is None else supplied
 payload_length=length if payload_length is None else payload_length
 if 'error'not in plan:
  if payload_length!=length or payload_length>256:expected=dict(error='PayloadLength')
  elif supplied!=count or supplied>257:expected=dict(error='CountMismatch')
 row.update(payload=payload,payload_length=payload_length,previous=previous,supplied=supplied,expected=expected)
 return row

def cases():
 base=geometry.cases();rows=[make(row,poison=True)for row in base]
 for code in range(5):
  for update in range(3):
   for length in [1,64,65,2047,2048]:
    flags=code|(update<<5)|16;row=dict(name=f'mode_{code}_{update}_{length}',offset=3,length=length,flags=flags,region=300,size=8,patch='',expected=geometry.plan(3,length,flags,300))
    rows.append(make(row,missing=range(257)if update else (),poison=True))
 for pattern in [1,2]:
  for code in [1,2,3,4]:
   row=dict(name=f'flat_{pattern}_{code}',offset=7,length=65,flags=code,region=128,size=8,patch='',expected=geometry.plan(7,65,code,128));rows.append(make(row,pattern=pattern,poison=True))
 for name,offset,length,missing in [('late_missing',1,16,[2]),('first_missing',1,16,[0]),('full_absent',0,64,range(257)),('interior_absent',1,16,[1])]:
  row=dict(name=name,offset=offset,length=length,flags=1,region=128,size=8,patch='',expected=geometry.plan(offset,length,1,128));rows.append(make(row,missing=missing))
 for name in ['geometry_0_0_1','geometry_2_1_17','capacity_1']:
  row=next(r for r in base if r['name']==name)
  for value in sorted({0,1,row['expected']['count']-1,row['expected']['count']+1,257,258,1<<63,MAX}):
   if value!=row['expected']['count']:
    case=make(row,supplied=value);case['name']='count_'+name+'_'+str(value);rows.append(case)
  logical=(row['length']+7)//8
  for value in sorted({0,logical-1,logical+1,256,257,1<<63,MAX}):
   if value!=logical:
    case=make(row,payload_length=value,supplied=MAX);case['name']='payload_'+name+'_'+str(value);rows.append(case)
 for name in ['flags_97','offset_max','forged_kind','connection','zero_0']:
  row=next(r for r in base if r['name']==name);case=make(row,supplied=MAX,payload_length=MAX);case['name']='precedence_'+name;rows.append(case)
 return rows+public_cases()

def public_cases():
 rows=[]
 def add(name,code,update,offset,length,size,source):
  flags=code|(update<<5)|16;raw=source.to_bytes((length+7)//8,'little')if isinstance(source,int)and source.bit_length()<=length else ((source&((1<<length)-1)).to_bytes((length+7)//8,'little')if isinstance(source,int)else source)
  row=dict(name=name,offset=offset,length=length,flags=flags,region=300,size=size,patch='',expected=geometry.plan(offset,length,flags,300,size));case=make(row,payload_override=raw,poison=True);case['public_source']=source if isinstance(source,int)else dict(buffer=source.hex());rows.append(case)
 for code in range(5):
  for update in range(3):
   for size in [4,8]:
    for offset,length in [(0,1),(3,65),(7,128)]:add(f'public_integer_{code}_{update}_{size}_{offset}_{length}',code,update,offset,length,size,0x89abcdef if size==4 else 0xfedcba9876543210)
 for code in [1,2,3,4]:
  for update in range(3):
   for length in [8,64,2048]:add(f'public_buffer_{code}_{update}_{length}',code,update,3,length,8,bytes((i*91+0xa6)&255 for i in range(length//8)))
 return rows
