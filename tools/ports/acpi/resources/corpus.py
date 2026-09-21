"""Original synthetic wire cases; independent bounded profile oracle."""
def le(x,w):return list(x.to_bytes(w,'little'))
def large(tag,data):return [128|tag]+le(len(data),2)+data
def address(w,kind=0,flags=12,values=None,source=[]):return large({2:8,4:7,8:10}[w],[kind,flags,0]+sum([le(x,w)for x in (values or [0,0x100,0x1ff,0,0x100])],[])+source)
def extended(flags=31,irqs=[17],source=[]):return large(9,[flags,len(irqs)]+sum([le(x,4)for x in irqs],[])+source)
CASES=[]
def add(name,data,**kw):CASES.append(dict(name=name,data=data,**kw))
add('irq-default',[0x22,0x40,0]);add('irq-empty',[0x22,0,0]);add('irq-all',[0x22,255,255])
for flags in [1,8,17,24,33,40,49,56]:add('irq-flags-'+str(flags),[0x23,0x81,0x80,flags])
for speed,width in [(0,0),(1,1),(2,2),(3,0)]:add(f'dma-{speed}-{width}',[0x2a,0x84,(speed<<5)|width|4])
add('io',[0x47,1,0xf8,3,0xf8,3,8,8]);add('io-zero',[0x47,0,0,0,0xff,0xff,0,0])
add('fixed-memory',large(6,[1]+le(0x12345000,4)+le(0x1000,4)));add('fixed-memory-raw-max',large(6,[254]+le(0xffffffff,4)+le(0xffffffff,4)))
for w in [2,4,8]:
 add('address-'+str(w),address(w,kind={2:2,4:1,8:0}[w]))
 add('address-max-'+str(w),address(w,flags=2,values=[2**(w*8)-1]*5))
add('address-source',address(2,source=[7,92,95,83,66,46,76,78,75,65,0]));add('address-empty-source',address(4,source=[0,0]))
add('extended-one',extended());add('extended-many',extended(0,[0,255,0xffffffff]));add('extended-source',extended(2,[16,17],[3,92,76,78,75,65,0]))
for tag in [6,7,9,10,14]:add('unsupported-small-'+str(tag),[tag<<3])
for tag in [1,2,4,5,11,13,15,16,17,18,19]:add('unsupported-large-'+str(tag),large(tag,[]))
for name,data in [('reserved-small',[0]),('reserved-small11',[88]),('reserved-large0',[128,0,0]),('reserved-large3',[131,0,0]),('reserved-large20',[148,0,0]),('irq-short',[0x21,0]),('irq-long',[0x24,0,0,1,0]),('irq-invalid-polarity',[0x23,1,0,0]),('irq-reserved',[0x23,1,0,65]),('dma-reserved',[0x2a,1,3]),('dma-flag-reserved',[0x2a,1,128]),('dma-short',[0x29,1]),('io-reserved',[0x47,128,0,0,0,0,0,1]),('fixed-short',large(6,[1]*8)),('fixed-long',large(6,[1]*10)),('address-short',large(8,[0]*12)),('address-reserved-kind',address(2,kind=3)),('address-reserved-flags',address(2,flags=128)),('address-unterminated',address(2,source=[0,65])),('address-source-index-only',address(2,source=[0])),('address-source-inner-nul',address(2,source=[0,65,0,65,0])),('address-source-nonascii',address(2,source=[0,128,0])),('extended-zero',large(9,[0,0,0,0,0,0])),('extended-truncated-table',large(9,[0,2]+le(17,4))),('extended-reserved',extended(128)),('extended-source-bad',extended(0,[17],[0,65])),('endtag-short',[0x78]),('endtag-long',[0x7a,0,0])]:add(name,data)
add('address-pcc',address(8,kind=10));add('address-vendor',address(8,kind=192))
for name,data in [('empty',[]),('large-header-one',[0x86]),('large-header-two',[0x86,9]),('large-payload-truncated',[0x86,9,0,1]),('small-payload-truncated',[0x22,1]),('length-max',[0x86,255,255])]:add(name,data)
add('offset-high',[0x79,0],at=2**64-1);add('length-over-capacity',[0x79,0],length=4097)
add('offset-nonzero',[9,9,0x22,0x40,0],at=2)
add('endtag',[0x79,0]);add('endtag-nonzero',[0x79,0x87])
for name,data in [('template-empty',[]),('template-only-end',[121,0]),('template-checksum',[121,135]),('template-bad-checksum',[121,1]),('template-trailing',[121,0,0]),('template-missing',[0x22,64,0]),('template-mixture',[0x71,0]+[0x22,64,0]+[0x2a,4,0]+[121,0]),('template-later-bad',[0x22,64,0,0]),('template-later-short',[0x22,64,0,0x86]),('template-source',[*address(2,source=[0,65,0]),121,0])]:add(name,data,template=True)
# A nonzero checksum includes unsupported descriptor bytes and all headers.
d=[0x71,9,0x22,64,0,121];add('template-checked-mixture',d+[(-sum(d))&255],template=True)

add('logical-length-max',[121,0],length=2**64-1)
add('template-length-max',[121,0],length=2**64-1,template=True)
add('template-length-capacity',[121,0],length=4097,template=True)
add('last-byte-truncated',[0]*4095+[121],at=4095)
add('last-two-bytes-end',[0]*4094+[121,0],at=4094)
add('extended-255',extended(7,list(range(255))))
add('address-distinct64',address(8,flags=0,values=[0x7fffffffffffffff,0x8000000000000000,0xffffffffffffffff,0xfedcba9876543210,0x123456789abcdef0]))

# ACPI 6.6 connection wire fixtures: all offsets are descriptor-relative.
def gpio(kind=0,flags=1,config=0,pins=[4],source=[92,71,80,73,79,48,0],vendor=[],general=1):
 source_at=23+2*len(pins);vendor_at=source_at+len(source)
 return large(12,[1,kind]+le(general,2)+le(flags,2)+[config]+le(125,2)+le(250,2)+le(23,2)+[0]+le(source_at,2)+le(vendor_at,2)+le(len(vendor),2)+sum([le(x,2)for x in pins],[])+source+vendor)
def i2c(mode=0,address=0x52,flags=2,vendor=[],source=[92,73,50,67,48,0],index=0,lvr=0,speed=400000):
 return large(14,[2,index,1,flags,mode,lvr,1]+le(6+len(vendor),2)+le(speed,4)+le(address,2)+vendor+source)
def changed(data,at,value,width=1):
 result=list(data);result[at:at+width]=le(value,width);return result
for flags in [0,1,2,3,5,8,17,27,29]:add('gpio-interrupt-'+str(flags),gpio(flags=flags))
for restriction in range(4):add('gpio-io-'+str(restriction),gpio(kind=1,flags=8|restriction,config=restriction,general=0))
add('gpio-pins-vendor',gpio(pins=[0,65535,513],vendor=[0,255,128]))
add('gpio-empty-source',gpio(source=[0]));add('gpio-offset-prefix',[1,2,3]+gpio(pins=[65535]),at=3)
for flags in range(8):add('i2c-flags-'+str(flags),i2c(flags=flags))
add('i2c-ten-bit',i2c(mode=1,address=1023,vendor=[255,0,128],index=255,lvr=255,speed=0xffffffff))
add('i2c-seven-max',i2c(address=127,speed=0));add('i2c-empty-source',i2c(source=[0]));add('i2c-offset-prefix',[9]+i2c(),at=1)
for name,at,value,width in [('revision',3,0,1),('kind',4,2,1),('general-low',5,2,1),('general-high',6,1,1),('flags-reserved',7,32,1),('flags-high',8,1,1),('both-level',7,4,1),('polarity-reserved',7,7,1),('config-reserved',9,4,1),('config-vendor',9,128,1),('pin-in-header',14,22,2),('pin-empty',14,25,2),('pin-reversed',14,27,2),('pin-offset-max',14,65535,2),('pin-odd',17,26,2),('source-offset-max',17,65535,2),('index',16,1,1),('vendor-in-source',19,25,2),('vendor-past-end',19,65535,2),('vendor-length',21,1,2),('vendor-length-max',21,65535,2)]:add('gpio-bad-'+name,changed(gpio(),at,value,width))
add('gpio-io-reserved',gpio(kind=1,flags=4));add('gpio-io-wake-reserved',gpio(kind=1,flags=16))
for name,source in [('unterminated',[65]),('inner-nul',[65,0,66,0]),('nonascii',[128,0]),('absent',[])]:add('gpio-source-'+name,gpio(source=source));add('i2c-source-'+name,i2c(source=source))
for name,at,value,width in [('revision-zero',3,0,1),('revision-old',3,1,1),('revision-high',3,3,1),('flags',6,8,1),('mode',7,2,1),('type-revision-zero',9,0,1),('type-revision-high',9,2,1),('type-length-short',10,5,2),('type-length-long',10,65535,2),('type-length-no-source',10,12,2),('address-seven',16,128,2),('address-reserved',16,65535,2),('bus-reserved',5,0,1),('bus-reserved-high',5,5,1),('bus-spi',5,2,1),('bus-uart',5,3,1),('bus-csi2',5,4,1),('bus-vendor',5,192,1)]:add('i2c-'+name,changed(i2c(),at,value,width))
add('i2c-ten-overflow',i2c(mode=1,address=1024))
add('gpio-short',large(12,[]));add('gpio-header-only',large(12,gpio()[3:23]));add('serial-short',large(14,[]));add('i2c-short',large(14,i2c()[3:17]))
gap=gpio();gap[23:23]=[99,88];gap[1:3]=le(len(gap)-3,2)
for offset in [14,17,19]:gap[offset:offset+2]=le(int.from_bytes(bytes(gap[offset:offset+2]),'little')+2,2)
add('gpio-padding-before-pins',gap)
for name,wire in [('gpio',gpio()),('i2c',i2c())]:add(name+'-capacity-end',[0]*(4096-len(wire))+wire,at=4096-len(wire))
add('template-connections',gpio()+i2c()+[121,0],template=True)
add('template-unsupported-bus',changed(i2c(),5,2)+[121,0],template=True)

def parse(data,length=None,at=0):
 n=len(data)if length is None else length
 def fail(s):return dict(outcome=s)
 if n>4096 or at>n:return fail('InvalidState')
 if at==n:return fail('Truncated')
 lead=data[at];islarge=bool(lead&128);tag=lead&127 if islarge else(lead>>3)&15;header=3 if islarge else 1
 if n-at<header:return fail('Truncated')
 size=int.from_bytes(bytes(data[at+1:at+3]),'little')if islarge else lead&7;body=at+header;end=body+size
 if end>n:return fail('Truncated')
 b=data[body:end];result=dict(outcome='Success',at=at,end=end,large=islarge,tag=tag)
 def num(offset,width):return int.from_bytes(bytes(b[offset:offset+width]),'little')
 def source(core):
  tail=b[core:]
  if not tail:return dict(present=False,index=0,start=0,end=0)
  if len(tail)<2 or tail[-1]!=0 or any(x==0 or x>127 for x in tail[1:-1]):return None
  return dict(present=True,index=tail[0],start=body+core+1,end=end-1)
 if (not islarge and tag in [6,7,9,10,14])or(islarge and tag in [1,2,4,5,11,13,15,16,17,18,19]):result.update(kind='Unsupported',norm=None);return result
 if not islarge:
  if tag==4:
   if size not in [2,3]:return fail('BadEncoding')
   f=b[2]if size==3 else 1
   if f&192 or f&9 not in [1,8]:return fail('BadEncoding')
   ints=[i for i in range(16)if num(0,2)&(1<<i)]
   result.update(kind='Irq',mask=num(0,2),flags=f,extended=False,consumer=False,edge=bool(f&1),low=bool(f&8),shared=bool(f&16),wake=bool(f&32),irqs=ints,source=dict(present=False,index=0,start=0,end=0))
  elif tag==5:
   if size!=2 or b[1]&128 or b[1]&3==3:return fail('BadEncoding')
   result.update(kind='Dma',flags=b[1],channels=b[0],speed=(b[1]>>5)&3,width=b[1]&3,master=bool(b[1]&4),norm=[2,b[0],(b[1]>>5)&3,b[1]&3,int(bool(b[1]&4))])
  elif tag==8:
   if size!=7 or b[0]&254:return fail('BadEncoding')
   result.update(kind='Io',decode=bool(b[0]&1),minimum=num(1,2),maximum=num(3,2),alignment=b[5],length=b[6],norm=[3,b[0]&1,num(1,2),num(3,2),b[5],b[6]])
  elif tag==15:
   if size!=1:return fail('BadEncoding')
   result.update(kind='EndTag',checksum=b[0],norm=None)
  else:return fail('BadEncoding')
 else:
  if tag==6:
   if size!=9:return fail('BadEncoding')
   result.update(kind='FixedMemory',flags=b[0],writable=bool(b[0]&1),base=num(1,4),length=num(5,4),norm=[4,b[0]&1,num(1,4),num(5,4)])
  elif tag in [7,8,10]:
   w={7:4,8:2,10:8}[tag];core=3+5*w
   if size<core:return fail('BadEncoding')
   if b[0]==10 or b[0]>=192:result.update(kind='Unsupported',norm=None);return result
   if b[0]>2 or b[1]&240:return fail('BadEncoding')
   src=source(core)
   if src is None:return fail('BadEncoding')
   values=[num(3+i*w,w)for i in range(5)]
   result.update(kind='Address',width=w,address_kind=b[0],flags=b[1],type_flags=b[2],values=values,source=src,norm=[5,b[0],int(bool(b[1]&8)),int(bool(b[1]&4)),int(bool(b[1]&2)),*values])
  elif tag==9:
   if size<6 or b[0]&224 or b[1]==0:return fail('BadEncoding')
   core=2+b[1]*4
   if size<core:return fail('BadEncoding')
   src=source(core)
   if src is None:return fail('BadEncoding')
   f=b[0];result.update(kind='Irq',flags=f,extended=True,consumer=bool(f&1),edge=bool(f&2),low=bool(f&4),shared=bool(f&8),wake=bool(f&16),irqs=[num(2+i*4,4)for i in range(b[1])],table_start=body+2,table_end=body+core,source=src)
  elif tag==12:
   if size<20:return fail('BadEncoding')
   kind=b[1];general=num(2,2);flags=num(4,2);config=b[6];pins=num(11,2);src=num(14,2);vendor=num(16,2);vendor_len=num(18,2);total=size+3
   if b[0]!=1 or b[13]!=0 or general&65534 or kind>1:return fail('BadEncoding')
   if kind==0 and(flags&65504 or (flags>>1)&3==3 or ((flags>>1)&3==2 and not flags&1)):return fail('BadEncoding')
   if kind==1 and flags&65524:return fail('BadEncoding')
   if not(23<=pins<src<vendor<=total)or(src-pins)%2 or vendor_len!=total-vendor:return fail('BadEncoding')
   name=b[src-3:vendor-3]
   if not name or name[-1]!=0 or any(x==0 or x>127 for x in name[:-1]):return fail('BadEncoding')
   if config>=128:result.update(kind='Unsupported',norm=None);return result
   if config>3:return fail('BadEncoding')
   pin_values=[int.from_bytes(bytes(b[i:i+2]),'little')for i in range(pins-3,src-3,2)];vendors=b[vendor-3:];conn=[0,int(bool(flags&1)),(flags>>1)&3,int(bool(flags&16))]if kind==0 else[1,flags&3]
   result.update(kind='Gpio',consumer=bool(general&1),shared=bool(flags&8),config=config,drive=num(7,2),debounce=num(9,2),pin_start=at+pins,pin_end=at+src,pins=pin_values,source=dict(present=True,index=0,start=at+src,end=at+vendor-1),vendor_start=at+vendor,vendor_end=end,vendors=vendors,general=general,flags=flags,connection=conn,norm=[6,general&1,int(bool(flags&8)),config,num(7,2),num(9,2),len(pin_values),*pin_values,len(name)-1,*name[:-1],len(vendors),*vendors,*conn])
  elif tag==14:
   if size<9:return fail('BadEncoding')
   bus=b[2]
   if bus in [2,3,4]or bus>=192:result.update(kind='Unsupported',norm=None);return result
   if bus!=1 or size<15:return fail('BadEncoding')
   flags=b[3];mode=b[4];address=num(13,2);typed=num(7,2);total=size+3
   if b[0]!=2 or flags&248 or mode&254 or b[6]!=1 or address>(1023 if mode==1 else 127):return fail('BadEncoding')
   if typed<6 or typed>=total-12:return fail('BadEncoding')
   name=b[9+typed:];vendors=b[15:9+typed]
   if not name or name[-1]!=0 or any(x==0 or x>127 for x in name[:-1]):return fail('BadEncoding')
   result.update(kind='I2c',consumer=bool(flags&2),shared=bool(flags&4),device=bool(flags&1),mode=mode,speed=num(9,4),address=address,lvr=b[5],flags=flags,source=dict(present=True,index=b[1],start=at+12+typed,end=end-1),vendor_start=at+18,vendor_end=at+12+typed,vendors=vendors,norm=[7,int(bool(flags&2)),int(bool(flags&4)),int(bool(flags&1)),mode,num(9,4),address,b[1],len(name)-1,*name[:-1],len(vendors),*vendors])
  else:return fail('BadEncoding')
 if result['kind']=='Irq':result['norm']=[1,int(result['consumer']),int(result['edge']),int(result['low']),int(result['shared']),int(result['wake']),len(result['irqs']),*result['irqs']]
 return result

def template(data,length=None):
 if length is not None and length>4096:return dict(outcome='InvalidState',next=0,count=0,unsupported=0,checksum=0)
 at=0;count=0;unsupported=0
 while at<len(data):
  p=parse(data,at=at)
  if p['outcome']!='Success':return dict(outcome=p['outcome'],next=at,count=count,unsupported=unsupported,checksum=0)
  at=p['end']
  if p['kind']=='EndTag':
   outcome='TrailingData'if at!=len(data)else'BadChecksum'if p['checksum']!=0 and sum(data)&255 else'Success'
   return dict(outcome=outcome,next=at,count=count,unsupported=unsupported,checksum=p['checksum'])
  count+=1;unsupported+=p['kind']=='Unsupported'
 return dict(outcome='MissingEndTag',next=at,count=count,unsupported=unsupported,checksum=0)
