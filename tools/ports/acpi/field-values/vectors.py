"""Original read-assembly vectors; independent logical bit extraction oracle."""
import geometry_vectors as geometry
MAX=(1<<64)-1

def byte_at(index,pattern):return 0 if pattern==1 else 255 if pattern==2 else (index*91+0xa6)&255 if pattern==3 else (index*37+11)&255

def make(row,pattern=0,poison=False,supplied=None):
 row=dict(row,pattern=pattern,poison=poison);plan=row.pop('expected');row['geometry']=plan
 if 'error'in plan:count=0;words=[];expected=dict(error='Geometry',cause=plan['error'])
 else:
  count=plan['count'];words=[]
  for chunk in plan['chunks']:
   value=sum(byte_at(chunk['offset']+i,pattern)<<(8*i)for i in range(chunk['width']))
   if poison:value|=MAX^((1<<(chunk['width']*8))-1)
   words.append(value)
  value=sum(((byte_at((row['offset']+i)//8,pattern)>>((row['offset']+i)%8))&1)<<i for i in range(row['length']))
  expected=dict(shape=plan['shape'],lock=bool(row['flags']&16),size=row['size'])
  if plan['shape']=='Integer':expected['value']=value
  else:expected.update(length=(row['length']+7)//8,bytes=list(value.to_bytes(256,'little')))
 supplied=count if supplied is None else supplied
 if 'error'not in plan and supplied!=count:expected=dict(error='CountMismatch')
 row.update(words=words,supplied=supplied,expected=expected)
 return row

def cases():
 base=geometry.cases();rows=[make(r)for r in base]
 for code in range(5):
  for size in [4,8]:
   for length in [1,32,33,64,65,2047,2048]:
    row=dict(name=f'pattern_{code}_{size}_{length}',offset=3,length=length,flags=code|16,region=300,size=size,patch='',expected=geometry.plan(3,length,code|16,300,size))
    rows.append(make(row,pattern=3,poison=True))
 for pattern in [1,2]:
  for code in [1,2,3,4]:
   row=dict(name=f'flat_{pattern}_{code}',offset=7,length=65,flags=code,region=128,size=8,patch='',expected=geometry.plan(7,65,code,128))
   rows.append(make(row,pattern=pattern,poison=True))
 for name in ['geometry_0_0_1','geometry_2_1_17','capacity_1']:
  row=next(r for r in base if r['name']==name)
  for supplied in sorted({0,1,row['expected']['count']-1,row['expected']['count']+1,257,258,1<<63,MAX}):
   if supplied==row['expected']['count']:continue
   case=make(row,supplied=supplied);case['name']='count_'+name+'_'+str(supplied);rows.append(case)
 for name in ['flags_97','offset_max','forged_kind','connection','zero_0']:
  row=next(r for r in base if r['name']==name);case=make(row,supplied=MAX);case['name']='precedence_'+name;rows.append(case)
 return rows
