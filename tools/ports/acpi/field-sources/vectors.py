"""Independent primary-profile ordinal oracle; no I/O or complete Store claim."""
MAX=(1<<64)-1

def expected(row):
 width=row['width'];kind=row['kind'];ordinal=row['ordinal']
 if width==0:return dict(error='Bounds')
 if width>2048:return dict(error='Capacity')
 if row['object_count']>64 or row['object']>=min(64,row['object_count']):return dict(error='InvalidState')
 if kind not in ['Integer','Buffer','String']:return dict(error='UnsupportedValue')
 if row.get('forced_error'):return dict(error=row['forced_error'])
 data=row['data']
 if kind=='Integer':total=1;number=row['number']&((1<<row['bits'])-1)
 else:
  if row['owned']:
   if len(data)>256:return dict(error='Capacity')
  else:
   if row['source_length']>1024:return dict(error='Capacity')
   if row['unit']!=7 or row['source_length']<len(data):return dict(error='Bounds')
   if len(data)>256 or row['declared']>256:return dict(error='Capacity')
  if kind=='String':
   if any(byte==0 or byte>=128 for byte in data):return dict(error='Encoding')
   total=len(data);number=data[ordinal]if ordinal<total else 0
  else:
   if not row['owned']:data=data+[0]*max(0,row['declared']-len(data))
   total=max(1,(len(data)*8+width-1)//width);number=(int.from_bytes(bytes(data),'little')>>(ordinal*width))if ordinal<total else 0
 if ordinal>=total:return dict(total=total,end=True)
 number&=(1<<width)-1
 return dict(total=total,ordinal=ordinal,length=(width+7)//8,bytes=list(number.to_bytes(256,'little')))

def cases():
 rows=[]
 def add(name,kind='Buffer',data=b'\xa6\x39\xf0',number=0,owned=False,bits=64,width=9,ordinal=0,declared=None,extra='',error=None,object_count=1,object=0,source_length=None,unit=7):
  data=list(data);r=dict(name=name,kind=kind,data=data,number=number,owned=owned,bits=bits,width=width,ordinal=ordinal,declared=len(data)if declared is None else declared,extra=extra,forced_error=error,object_count=object_count,object=object,source_length=len(data)if source_length is None else source_length,unit=unit);r['expected']=expected(r);rows.append(r)
 for bits in [32,64]:
  for width in [1,8,31,32,33,64,65,2048]:
   for ordinal in [0,1,MAX]:add(f'integer_{bits}_{width}_{ordinal}','Integer',number=0xfedcba9889abcdef,bits=bits,width=width,ordinal=ordinal)
 for owned in [False,True]:
  for length in [0,1,3,256]:
   for width in [1,9,64,2048]:
    data=bytes((i*91+166)&255 for i in range(length));total=max(1,(length*8+width-1)//width)
    for ordinal in sorted({0,total//2,total-1,total,MAX}):add(f'buffer_{length}_{width}_{ordinal}_{owned}',data=data,owned=owned,width=width,ordinal=ordinal)
  for text in [b'',b'A',b'Az!',b'A'*256]:
   for width in [1,8,65,2048]:
    for ordinal in sorted({0,max(0,len(text)-1),len(text),MAX}):add(f'string_{len(text)}_{width}_{ordinal}_{owned}','String',data=text,owned=owned,width=width,ordinal=ordinal)
  add('encoding_tail_end_'+str(owned),'String',data=b'AB\xff',owned=owned,ordinal=MAX)
  add('nul_end_'+str(owned),'String',data=b'A\0',owned=owned,ordinal=MAX)
  for kind in ['String','Buffer']:
   add(f'owner_{kind}_{owned}',kind,data=b'AB',owned=True,ordinal=MAX,error='InvalidState',extra='store.space.objects[0].value=Value::'+kind+' {'+('string_storage:StringStorage::Owned {string_owner:1}'if kind=='String'else'buffer_storage:BufferStorage::Owned {buffer_owner:1}')+'};')
   add(f'length_max_{kind}_{owned}',kind,data=b'AB',owned=True,ordinal=MAX,error='Capacity',extra=f'store.bytes.blocks[0].length={MAX};')
 for width in [0,2049,1<<63,MAX]:add('width_'+str(width),width=width,ordinal=MAX,object_count=MAX)
 for label,kwargs in [('empty',dict(object_count=0)),('count65',dict(object_count=65)),('count_high',dict(object_count=1<<63)),('count_max',dict(object_count=MAX)),('id_high',dict(object=1<<63)),('id_max',dict(object=MAX))]:add('identity_'+label,ordinal=MAX,**kwargs)
 for kind in ['Uninitialized','Package','Method','OperationRegion','NameReference','BufferField','Device','Reference']:add('unsupported_'+kind,kind,ordinal=MAX)
 for kind in ['String','Buffer']:
  add('unit_'+kind,kind,data=b'AB',unit=8,ordinal=MAX)
  add('source_max_'+kind,kind,data=b'AB',source_length=MAX,ordinal=MAX)
  add('uninitialized_'+kind,kind,data=b'AB',owned=True,ordinal=MAX,error='InvalidState',extra='store.bytes.blocks[0].initialized=false;')
  add('poison_unused_'+kind,kind,data=b'AB',owned=True,source_length=MAX,extra='store.bytes.blocks[0].bytes[2]=255;store.bytes.blocks[0].bytes[255]=254;')
 add('padding',data=b'AB',declared=4,width=9,ordinal=3)
 add('initializer_larger',data=b'ABCD',declared=1,width=9,ordinal=3)
 add('declared_max',data=b'AB',declared=MAX,ordinal=MAX)
 add('integer_unused_source','Integer',number=MAX,source_length=MAX)
 assert len(rows)==len({r['name']for r in rows});return rows
