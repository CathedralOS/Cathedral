#!/usr/bin/env python3
"""Original bounded AML bytecode to observe public generic object semantics."""
from check import fixture as f

integer=f.integer; name=f.name; op=f.op; ret=f.ret; pkg=f.pkg
def buffer(data,declared=None):return pkg(0x11,integer(len(data) if declared is None else declared)+bytes(data))
def string(text):return b'\x0d'+text.encode('ascii')+b'\0'
def package(*items):return pkg(0x12,bytes([len(items)])+b''.join(items))
def index(value,number,target=b'\0'):return op(0x88,value,integer(number),target)
def deref(value):return op(0x83,value)
def reference(value):return op(0x71,value)
def cases():
 rows=[]
 def add(label,body,globals=None,methods=None,after=(),revision=2,note=''):
  table=b''.join(b'\x08'+name(k)+v for k,v in (globals or {}).items())
  for key,(flags,code) in {'MAIN':(0,body),**(methods or {})}.items():table+=pkg(0x14,name(key)+bytes([flags])+code)
  rows.append(dict(name=label,table_hex=table.hex(),revision=revision,after=list(after),review_note=note))
 for label,value in [('buffer',buffer([1,2,3])),('string',string('ABC')),('package',package(integer(1),string('A'),buffer([2,3]))),('empty_buffer',buffer([])),('empty_string',string('')),('padded_buffer',buffer([1],4)),('oversized_initializer',buffer([1,2,3],1))]:
  add('literal_'+label,ret(value),note='Buffer actual size follows ACPI max(declared, initializer); pinned oversized initializer may panic.')
 for operation,label in [(0x70,'store'),(0x9d,'copy')]:
  add(label+'_local_named_reference',op(0x70,reference(name('X')),b'\x60')+op(operation,integer(22),b'\x60')+ret(b'\x60'),{'X':integer(11)},after=['X'])
  add(label+'_arg_named_reference',ret(name('CAL')+reference(name('X'))),{'X':integer(11)},methods={'CAL':(1,op(operation,integer(22),b'\x68')+ret(b'\x68'))},after=['X'])
  add(label+'_arg_plain_named',ret(name('CAL')+name('X')),{'X':integer(11)},methods={'CAL':(1,op(operation,integer(22),b'\x68')+ret(b'\x68'))},after=['X'])
  add(label+'_arg_plain_local',op(0x70,integer(11),b'\x60')+name('CAL')+b'\x60'+ret(b'\x60'),methods={'CAL':(1,op(operation,integer(22),b'\x68')+ret(b'\x68'))})
  add(label+'_local_plain_named',op(0x70,name('X'),b'\x60')+op(operation,integer(22),b'\x60')+ret(b'\x60'),{'X':integer(11)},after=['X'])
  add(label+'_bytes_self',op(operation,name('BUF'),name('BUF'))+ret(name('BUF')),{'BUF':buffer([1,2,3])},after=['BUF'])
  add(label+'_bytes_distinct',op(operation,name('SRC'),name('DST'))+ret(name('DST')),{'SRC':buffer([1,2,3]),'DST':buffer([9])},after=['SRC','DST'])
  add(label+'_named_type',op(operation,string('12'),name('X'))+ret(name('X')),{'X':integer(11)},after=['X'])
  add(label+'_package_index',op(operation,integer(22),index(name('PKG'),0))+ret(name('PKG')),{'PKG':package(integer(11))},after=['PKG'])
 add('reference_local_identity',op(0x70,integer(11),b'\x60')+op(0x70,reference(b'\x60'),b'\x61')+op(0x70,integer(22),b'\x60')+ret(deref(b'\x61')))
 add('reference_named_return',ret(reference(name('X'))),{'X':integer(11)})
 add('reference_local_escape',ret(name('CAL')),methods={'CAL':(0,op(0x70,integer(11),b'\x60')+ret(reference(b'\x60')))})
 add('late_named_operand_read',ret(op(0x72,name('X'),name('CAL'),b'\0')),{'X':integer(11)},methods={'CAL':(0,op(0x70,integer(22),name('X'))+ret(integer(1)))},after=['X'])
 add('package_copy_shared_element',op(0x9d,name('PKG'),name('CPY'))+index(name('CPY'),0,b'\x60')+op(0x70,integer(22),b'\x60')+ret(name('PKG')),{'PKG':package(integer(11)),'CPY':integer(0)},after=['PKG','CPY'])
 add('buffer_copy_independent',op(0x9d,name('BUF'),name('CPY'))+index(name('CPY'),0,b'\x60')+op(0x70,integer(22),b'\x60')+ret(name('BUF')),{'BUF':buffer([11]),'CPY':integer(0)},after=['BUF','CPY'])
 for label,value in [('buffer',buffer([11,22])),('string',string('AB')),('package',package(integer(11),integer(22)))]:
  for at in [0,1,2]:add('index_'+label+'_'+str(at),ret(deref(index(name('OBJ'),at))),{'OBJ':value})
  add('index_'+label+'_write',index(name('OBJ'),0,b'\x60')+op(0x70,integer(67),b'\x60')+ret(name('OBJ')),{'OBJ':value},after=['OBJ'])
  add('index_'+label+'_reference',ret(index(name('OBJ'),0)),{'OBJ':value})
 for revision in [1,2]:
  for label,value in [('integer',integer(0x1234)),('buffer',buffer([0x34,0x12])),('string',string('12')),('empty_string',string(''))]:
   for code,operation in [(0x96,'to_buffer'),(0x99,'to_integer'),(0x97,'to_decimal'),(0x98,'to_hex')]:
    add(operation+'_'+label+'_'+str(revision),ret(op(code,value,b'\0')),revision=revision)
 for length in [0,1,9]:add('to_string_nul_'+str(length),ret(op(0x9c,buffer([65,0,66]),integer(length),b'\0')))
 for label,left,right in [('integers',integer(0x12),integer(0x34)),('buffers',buffer([1]),buffer([2,3])),('strings',string('A'),string('B')),('string_integer',string('A'),integer(12)),('buffer_string',buffer([1]),string('A'))]:
  add('concat_'+label,ret(op(0x73,left,right,b'\0')))
 for label,value in [('buffer',buffer([1,2,3])),('string',string('ABC'))]:
  for start,length in [(0,2),(1,1),(1,9),(3,1)]:add('mid_'+label+'_'+str(start)+'_'+str(length),ret(op(0x9e,value,integer(start),integer(length),b'\0')))
 assert len({r['name'] for r in rows})==len(rows)
 return rows
