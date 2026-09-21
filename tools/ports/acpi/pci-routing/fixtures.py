#!/usr/bin/env python3
"""Original authored AML/resource fixtures; no firmware or external suite input."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent

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
def irq(n,flags=0x18):return bytes([0x23])+((1<<n).to_bytes(2,'little'))+bytes([flags])
def extended(numbers,flags=13):
 payload=bytes([flags,len(numbers)])+b''.join(n.to_bytes(4,'little')for n in numbers)
 return b'\x89'+len(payload).to_bytes(2,'little')+payload
END=b'\x79\0';IRQ=irq(10)+END

def cases():
 rows=[]
 def add(label,entries=None,strict='accept',reason='Four typed fields, DWORD values, wildcard function and direct GSI.',queries=None,root=b'',local=b'',outer=None,method_prt=False,body=None,strict_routes=None):
  if entries is None:entries=[[0x3ffff,0,0,40]]
  obj={'package':[{'package':r}if isinstance(r,list)else r for r in entries]}if outer is None else outer
  declaration=method('_PRT',b'\xa4'+value(obj))if method_prt else named('_PRT',obj)
  aml=root+device('PCI0',local+declaration)if body is None else body
  rows.append(dict(name=label,aml_hex=aml.hex(),prt_path='\\PCI0._PRT',entries=entries,queries=queries or [[3,0,0]],strict_expectation=dict(complete_request=strict,reason=reason,route_expectations=strict_routes or []),origin='Original synthetic AML and resource bytes; ACPI6.6 §6.2.14 and chapters19/20; no firmware fixtures'))
 def linked(label,source=None,index=0,resource=IRQ,strict='accept',reason='Named link; source-index selects a descriptor.',elements=None,root=None,local=b'',**kw):
  source={'name':'LNKA'}if source is None else source
  defs=device('LNKA',named('_CRS',{'buffer':resource.hex()}))if root is None else root
  add(label,[elements if elements is not None else [0x3ffff,0,source,index]],strict,reason,root=defs,local=local,**kw)
 add('direct_all_pins',[[0x3ffff,p,0,40+p]for p in range(4)],queries=[[3,f,p]for f in [0,7]for p in range(4)]+[[4,0,0]])
 add('direct_empty',[],queries=[[3,0,0]])
 for gsi in [0,1,0xffffffff,0x100000000,(1<<64)-1]:add('gsi_'+str(gsi),[[0x3ffff,0,0,gsi]],'accept'if gsi<=0xffffffff else'reject','Source-index/GSI must fit DWORD; pin casts to u32.')
 for address in [0xffff,0x3ffff,0xffff_ffff,0x1_0003ffff,(1<<64)-1]:add('address_'+str(address),[[address,0,0,41]],'accept'if address<=0xffffffff else'reject','Address must fit DWORD; pin discards bits32..63.',queries=[[(address>>16)&65535,0,0]])
 for function in [0,1,7,8,65534,65535]:add('function_'+str(function),[[(3<<16)|function,0,0,42]],'accept'if function==65535 else'reject','ACPI6.6 requires wildcard function FFFF; pin also matches exact functions.',queries=[[3,f,0]for f in [0,1,7,8,65534,65535]])
 for pin in [4,255,256,(1<<64)-1]:add('pin_'+str(pin),[[0x3ffff,pin,0,40]],'reject','Only pin integers0..3 are admitted.')
 add('first_duplicate_wildcard',[[0x3ffff,0,0,41],[0x3ffff,0,0,42]],reason='Duplicate valid records retained in source order; first match selected.')
 add('wildcard_before_specific',[[0x3ffff,0,0,41],[0x30001,0,0,42]],'reject','Specific function is a pin extension; first-match precedence is observable.',queries=[[3,1,0],[3,7,0]])
 add('specific_before_wildcard',[[0x30001,0,0,42],[0x3ffff,0,0,41]],'reject','Specific function is a pin extension; first-match precedence is observable.',queries=[[3,1,0],[3,7,0]])
 add('outer_integer',outer=7,strict='reject',reason='Outer result must be Package.')
 add('inner_integer',[7],strict='reject',reason='Every entry must be Package.')
 for n in range(7):
  fields=[0x3ffff,0,0,40,99,100][:n]
  add('direct_length_'+str(n),[fields],'accept'if n==4 else'reject','Each inner package has exactly four elements; pin indexes or ignores surplus.')
 for n in range(7):
  fields=[0x3ffff,0,{'name':'LNKA'},0,99,100][:n]
  linked('link_length_'+str(n),elements=fields,strict='accept'if n==4 else'reject',reason='Four-element shape required; pin never reads link source-index.')
 for position in [0,1,3]:add('field_'+str(position)+'_string',[[{'text':'BAD'}if j==position else v for j,v in enumerate([0x3ffff,0,0,40])]],'reject','Address/pin/direct-GSI field has wrong type.')
 for label,source in [('one',1),('maximum',(1<<64)-1),('buffer',{'buffer':'00'}),('package',{'package':[]})]:add('source_'+label,[[0x3ffff,0,source,40]],'reject','Source must be NamePath or integer zero; no implicit integer conversion.')
 linked('name_simple')
 linked('name_absolute',{'name':'\\LNKA'})
 linked('name_parent',{'name':'^LNKA'})
 linked('name_missing_simple',{'name':'MISS'},strict='reject',reason='Named source level absent.')
 linked('name_missing_absolute',{'name':'\\MISS'},strict='reject',reason='Strict resolves/validates source level; pin defers absolute missing name to route.')
 nested=device('BRDG',device('LNKA',named('_CRS',{'buffer':IRQ.hex()})))
 linked('name_relative_multisegment',{'name':'BRDG.LNKA'},root=b'',local=nested,reason='Relative path must resolve against source scope; pin returns it unresolved.')
 linked('name_absolute_multisegment',{'name':'\\PCI0.BRDG.LNKA'},root=b'',local=nested)
 linked('name_parent_multisegment',{'name':'^BRDG.LNKA'},root=nested,reason='Parent-relative path must resolve; pin leaves non-search name unresolved.')
 for label,text in [('simple','LNKA'),('absolute','\\LNKA'),('parent','^LNKA'),('relative_multi','BRDG.LNKA'),('missing','\\MISS'),('empty',''),('invalid_lead','1'),('long_segment','AB.CDEFG'),('empty_segment','LNKA.'),('root','\\'),('two_parents','^^LNKA')]:
  linked('string_'+label,{'text':text},strict='reject',reason='String source is an explicit pin/legacy extension, not ACPI6.6 NamePath syntax.')
 linked('string_non_ascii',{'string_hex':'ff'},strict='reject',reason='No malformed/non-ASCII source string admission.')
 local=device('LNKA',named('_CRS',{'buffer':(irq(11)+END).hex()}))
 linked('name_nearest_level',local=local,reason='Single NameSeg selects nearest existing level.')
 linked('name_object_shadow',local=named('LNKA',0),reason='An ordinary nearer object does not shadow a farther level in level lookup.')
 linked('string_nearest_level',{'text':'LNKA'},local=local,strict='reject',reason='String extension searches from _PRT path then its ancestors.')
 for source,label in [({'name':'LNKA'},'name'),({'text':'LNKA'},'string')]:
  package={'package':[{'package':[0x3ffff,0,source,0]}]}
  base=device('BASE',device('LNKA',named('_CRS',{'buffer':(irq(11)+END).hex()}))+named('PKG0',package))
  body=device('LNKA',named('_CRS',{'buffer':IRQ.hex()}))+base+device('PCI0',method('_PRT',b'\xa4'+name('\\BASE.PKG0')))
  add('captured_source_scope_'+label,[[0x3ffff,0,source,0]],'accept'if label=='name'else'reject','NamePath retains declaration scope BASE; string extension searches from PCI0._PRT.',body=body)
 for label,index in [('zero',0),('one',1),('two',2),('maximum',0xffffffff),('overflow',0x100000000),('string',{'text':'BAD'})]:linked('link_index_'+label,index=index,strict='accept'if index==0 else'reject',reason='Source-index must be DWORD and identify the selected descriptor; pin ignores type/value.')
 resources=[
  ('two_irqs_index0',irq(9)+irq(10)+END,0,'accept','First physical descriptor is selected; pin rejects two retained resources.'),
  ('two_irqs_index1',irq(9)+irq(10)+END,1,'accept','Second physical descriptor is selected; pin rejects two retained resources.'),
  ('vendor_then_irq_index0',b'\x71\xaa'+irq(10)+END,0,'reject','Physical descriptor0 is vendor data, not IRQ; pin filters it and ignores index.'),
  ('vendor_then_irq_index1',b'\x71\xaa'+irq(10)+END,1,'accept','Physical descriptor1 is IRQ; pin filters vendor descriptor.'),
  ('dma_then_irq_index1',b'\x2a\x01\x00'+irq(10)+END,1,'accept','Second descriptor is IRQ; pin retains DMA and rejects aggregate length.'),
  ('only_dma',b'\x2a\x01\x00'+END,0,'reject','Selected resource is not IRQ.'),
  ('empty_crs',b'',0,'reject','Strict template needs terminal EndTag and selected descriptor.'),
  ('only_end',END,0,'reject','EndTag is not an IRQ descriptor.'),
  ('missing_end',irq(10),0,'reject','Existing strict resource profile requires EndTag; pin accepts exhaustion.'),
  ('valid_checksum',irq(10)+b'\x79'+bytes([(-sum(irq(10)+b'\x79'))&255]),0,'accept','Nonzero EndTag checksum verifies the complete template.'),
  ('bad_checksum',irq(10)+b'\x79\x01',0,'reject','Nonzero EndTag checksum must verify; pin ignores it.'),
  ('trailing_after_end',irq(10)+END+b'\xff',0,'reject','No trailing bytes after strict terminal EndTag; pin ignores remainder.'),
  ('truncated_irq',b'\x23\x01',0,'reject','Descriptor extent is incomplete; pin may panic.'),
  ('extended_gsi',extended([0x12345678])+END,0,'accept','Extended IRQ carries a full DWORD interrupt.'),
  ('extended_multiple',extended([9,10])+END,0,'reject','Current-resource extended IRQ count must be one; pin returns both.'),
 ]
 for label,data,index,status,reason in resources:linked(label,index=index,resource=data,strict=status,reason=reason)
 linked('crs_wrong_type',root=device('LNKA',named('_CRS',7)),strict='reject',reason='Detached _CRS result must be Buffer.')
 linked('crs_missing',root=device('LNKA',b''),strict='reject',reason='Selected link lacks _CRS.')
 linked('crs_method',root=device('LNKA',method('_CRS',b'\xa4'+value({'buffer':IRQ.hex()}))))
 add('prt_method',method_prt=True)
 add('error_late_row',[[0x3ffff,0,0,40],[0x4ffff,4,0,41]],'reject','Decode validates all rows before publishing even when first row would match.')
 add('capacity_32',[[0x3ffff,0,0,40]]*32,reason='Maximum32 output records in the proposed bounded profile; all rows are well formed.')
 add('capacity_33',[[0x3ffff,0,0,40]]*33,'reject','Explicit bounded output-capacity32 policy; ACPI does not impose this limit.')
 assert len({r['name']for r in rows})==len(rows)
 return rows
if __name__=='__main__':
 text=json.dumps(dict(format='cathedral-pci-routing-fixtures-v1',cases=cases()),indent=2,sort_keys=True)+'\n';path=HERE/'fixtures.json'
 if '--check'in sys.argv:assert path.read_text()==text,'fixture drift'
 else:path.write_text(text)
 print(len(cases()),'original _PRT fixtures')
