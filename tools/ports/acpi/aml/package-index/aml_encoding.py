"""Original authored AML byte constructors, reused from Cathedral PCI fixtures."""
def integer(value):
 assert 0<=value<1<<64
 if value in [0,1]:return bytes([value])
 if value==(1<<64)-1:return b'\xff'
 width=next(n for n in [1,2,4,8]if value<1<<(8*n))
 return bytes([{1:10,2:11,4:12,8:14}[width]])+value.to_bytes(width,'little')
def pkg(op,payload):
 for width in range(1,5):
  length=len(payload)+width
  if length<1<<(6 if width==1 else 4+8*(width-1)):
   encoded=bytes([length])if width==1 else bytes([((width-1)<<6)|(length&15)])+(length>>4).to_bytes(width-1,'little')
   return (bytes([op])if op<256 else op.to_bytes(2,'big'))+encoded+payload
 raise ValueError('package too large')
def name(text):
 prefix=b''
 if text.startswith('\\'):prefix=b'\\';text=text[1:]
 while text.startswith('^'):prefix+=b'^';text=text[1:]
 if not text:return prefix+b'\0'
 segments=[s.encode('ascii').ljust(4,b'_')for s in text.split('.')]
 assert all(len(s)==4 for s in segments)
 return prefix+(b''if len(segments)==1 else b'.'if len(segments)==2 else b'/'+bytes([len(segments)]))+b''.join(segments)
def value(v):
 if isinstance(v,int):return integer(v)
 if 'text'in v:return b'\x0d'+v['text'].encode('ascii')+b'\0'
 if 'string_hex'in v:return b'\x0d'+bytes.fromhex(v['string_hex'])+b'\0'
 if 'name'in v:return name(v['name'])
 if 'buffer'in v:
  data=bytes.fromhex(v['buffer']);return pkg(0x11,integer(len(data))+data)
 if 'package'in v:return pkg(0x12,bytes([len(v['package'])])+b''.join(map(value,v['package'])))
 raise ValueError(v)
def named(n,v):return b'\x08'+name(n)+value(v)
def device(n,body):return pkg(0x5b82,name(n)+body)
def method(n,body):return pkg(0x14,name(n)+b'\0'+body)
