"""Original field geometry vectors and independent interval expectations."""
MAX=(1<<64)-1

def plan(offset,length,flags,region,size=8):
 code=flags&15;update=(flags>>5)&3
 if flags>=128 or code>5 or update>2:return dict(error='InvalidFlags')
 if code==5:return dict(error='UnsupportedBuffer')
 if length==0:return dict(error='UnsupportedZeroWidth')
 if length>2048:return dict(error='Capacity')
 if offset+length>MAX:return dict(error='Overflow')
 width={0:1,1:1,2:2,3:4,4:8}[code];bits=width*8
 first=(offset//bits)*width;end=((offset+length+bits-1)//bits)*width
 if end>region:return dict(error='OutsideRegion')
 chunks=[]
 for byte in range(first,end,width):
  native_start=byte*8;left=max(offset,native_start);right=min(offset+length,native_start+bits);count=right-left;native=left-native_start
  chunks.append(dict(offset=byte,width=width,field_bit=left-offset,native_bit=native,bit_count=count,mask=((1<<count)-1)<<native,partial=count!=bits))
 return dict(start=first,end=end,width=width,count=len(chunks),preserve_reads=sum(c['partial']for c in chunks)if update==0 else 0,locked=bool(flags&16),shape='Integer'if length<=size*8 else'Buffer',bytes=(length+7)//8,chunks=chunks)

def cases():
 rows=[]
 def add(name,offset,length,flags,region=128,size=8,patch=''):
  rows.append(dict(name=name,offset=offset,length=length,flags=flags,region=region,size=size,patch=patch,expected=plan(offset,length,flags,region,size)))
 for code,width in [(0,1),(1,1),(2,2),(3,4),(4,8)]:
  for offset in sorted({0,1,width*8-1,width*8,width*8+1}):
   for length in sorted({1,width*8-1,width*8,width*8+1,33,65}):add(f'geometry_{code}_{offset}_{length}',offset,length,code)
 for code in [1,3,4]:
  for length in [32,33,64,65]:add(f'integer32_{code}_{length}',1,length,code,size=4)
 for update in [1,2]:
  for offset,length in [(3,11),(0,16),(3,17)]:add(f'update_{update}_{offset}_{length}',offset,length,2|(update<<5))
 for code in range(5):add('locked_'+str(code),1,65,code|16)
 for code,offset in [(0,1),(1,1),(2,15),(4,63)]:add('capacity_'+str(code),offset,2048,code,300)
 for name,offset,length,flags,region in [('odd_word',16,1,2,3),('short_qword',8,1,4,7),('empty_region',0,1,1,0),('field_past_region',64,1,1,8),('region_max',1,65,4,MAX),('offset_max',MAX,1,1,MAX),('end_overflow',MAX-6,7,1,MAX),('last_byte',MAX-7,7,1,MAX),('length_max',0,MAX,1,MAX),('length_high',0,1<<63,1,MAX),('offset_high',1<<63,64,4,MAX)]:add(name,offset,length,flags,region)
 for flags in [5,6,15,0x61,0x81,0xff]:add('flags_'+str(flags),3,11,flags)
 for offset in [0,1,7]:add('zero_'+str(offset),offset,0,1)
 add('capacity_overflow',0,2049,1,300)
 for name,patch,error in [
 ('kind','kind=DeclarationKind::Bank;','UnsupportedKind'),('index','kind=DeclarationKind::Index;','UnsupportedKind'),
 ('connection','field.connection=Connection::Name {connection_name:Path {absolute:true}};','UnsupportedConnection'),
 ('attribute','field.access.attribute=1;','UnsupportedMetadata'),('mode','field.access.attribute_mode=1;','UnsupportedMetadata'),('extended','field.access.extended=true;','UnsupportedMetadata'),('access_length','field.access.access_length=1;','UnsupportedMetadata'),
 ('forged_kind','field.access.kind=AccessType::QWord;','InconsistentAccess'),('forged_update','field.access.update=UpdateRule::WriteAsOnes;','InconsistentAccess'),('forged_lock','field.access.locked=true;','InconsistentAccess')]:
  add(name,3,11,2,patch=patch);rows[-1]['expected']=dict(error=error)
 return rows
