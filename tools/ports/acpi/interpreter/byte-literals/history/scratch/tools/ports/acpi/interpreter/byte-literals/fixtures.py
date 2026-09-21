#!/usr/bin/env python3
import json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
MAX=(1<<64)-1

def integer(n,width=None):
 if width is None:
  if n==0:return b'\0'
  if n==1:return b'\1'
  width=next(w for w in [1,2,4,8]if n<(1<<(8*w)))
 return bytes([{1:10,2:11,4:12,8:14}[width]])+n.to_bytes(width,'little')
def package(body,width=None):
 if width is None:width=next(w for w in range(1,5)if len(body)+w<(64 if w==1 else 1<<(4+8*(w-1))))
 n=len(body)+width
 length=bytes([n])if width==1 else bytes([((width-1)<<6)|(n&15)])+(n>>4).to_bytes(width-1,'little')
 return b'\x11'+length+body

def cases():
 rows=[]
 def add(name,data,outcome='Success',kind=None,expected=b'',declared=0,start=None,end=None,**kw):
  at=kw.pop('at',0);data=bytes(data);row=dict(name=name,data=list(data),outcome=outcome,kind=kind,expected=list(expected),declared=declared,source_length=kw.pop('source_length',len(data)),frame_end=kw.pop('frame_end',len(data)),source_unit=kw.pop('source_unit',7),frame_unit=kw.pop('frame_unit',7),at=at,bits=kw.pop('bits',64),offset=kw.pop('offset',at),next=kw.pop('next',len(data)),start=start,end=end);assert not kw,kw;rows.append(row)
 def string(name,payload,prefix=b'',suffix=b'',**kw):
  data=prefix+b'\x0d'+bytes(payload)+b'\0'+suffix;at=len(prefix);add(name,data,kind='String',expected=payload,start=at+1,end=at+1+len(payload),at=at,next=at+len(payload)+2,**kw)
 def buffer(name,size,payload,width=None,pkg_width=None,bits=64,prefix=b'',suffix=b'',**kw):
  sizebytes=integer(size,width);literal=package(sizebytes+bytes(payload),pkg_width);at=len(prefix);data=prefix+literal+suffix;logical=max(size&((1<<bits)-1),len(payload));start=at+len(literal)-len(payload)
  add(name,data,kind='Buffer',expected=(bytes(payload)+bytes(max(0,logical-len(payload))))if logical<=256 else b'',declared=size&((1<<bits)-1),start=start,end=at+len(literal),at=at,next=at+len(literal),bits=bits,**kw)
 for n in [0,1,2,63,255,256]:string('string_'+str(n),bytes([65+(i%26)for i in range(n)]))
 string('string_ascii_extremes',[1,127,32]);string('string_opcode_data',[17,14,92,112]);string('string_nul_stops',[],suffix=b'\xff\xff');string('string_offset',[65,66],prefix=b'\xff'*11,suffix=b'\xff'*5)
 string('string_capacity',b'A'*257,outcome='Capacity')
 add('string_missing_nul',b'\x0dAB','Truncated',offset=3)
 add('string_nonascii',b'\x0dA\x80\0','BadEncoding',offset=3)
 add('string_nonascii_late',b'\x0d'+b'A'*255+b'\xff\0','BadEncoding',offset=257)
 add('string_frame_nul_outside',b'\x0dAB\0','Truncated',frame_end=3,offset=3)
 add('string_source_nul_outside',b'\x0dAB\0','Truncated',source_length=3,frame_end=3,offset=3)
 string('string_exact_frame',[65,66],suffix=b'\xff',frame_end=4)
 for n in [0,1,2,63,255,256]:buffer('buffer_exact_'+str(n),n,bytes(i%256 for i in range(n)))
 for n in [0,1,255,256]:buffer('buffer_zero_pad_'+str(n),n,[])
 buffer('buffer_partial_pad',256,[1,2,3]);buffer('buffer_initializer_larger',1,[1,2,3]);buffer('buffer_zero_declared',0,[1,2,3])
 buffer('buffer_opcode_bytes',4,[0x11,0x0d,0xa4,0xff]);buffer('buffer_offset',3,[1,2,3],prefix=b'\xff'*9,suffix=b'\xff'*5)
 for width in [1,2,3,4]:buffer('buffer_pkg_width_'+str(width),2,[1,2],pkg_width=width)
 for width in [1,2,4,8]:buffer('buffer_size_width_'+str(width),3,[1],width=width)
 buffer('buffer_width32_truncate',0x100000003,[1],width=8,bits=32)
 buffer('buffer_width32_zero',0x100000000,[],width=8,bits=32)
 buffer('buffer_width64_large',0x100000003,[1],width=8,outcome='Capacity')
 buffer('buffer_capacity_size',257,[],outcome='Capacity');buffer('buffer_capacity_initializer',1,b'Z'*257,outcome='Capacity')
 buffer('buffer_size_max64',MAX,[],outcome='Capacity');buffer('buffer_size_max32',MAX,[],bits=32,outcome='Capacity')
 for bits in [32,64]:add('buffer_ones_'+str(bits),package(b'\xff'),'Capacity',bits=bits)
 add('buffer_revision',package(b'\x5b\x30'),kind='Buffer',expected=b'\0\0',declared=2,start=4,end=4)
 for label,body in [('arg',b'\x68'),('local',b'\x60'),('name',b'SIZE'),('add',b'\x72\1\1\0')]:add('buffer_dynamic_'+label,package(body+b'\x01\x02'),'UnsupportedSyntax',offset=2)
 for label,data,error,offset in [('missing_package',b'\x11','Truncated',1),('truncated_package',b'\x11\x40','Truncated',2),('reserved_package',b'\x11\x50\0','BadEncoding',1),('package_too_short',b'\x11\x40\0','BadEncoding',1),('missing_size',b'\x11\1','Truncated',2),('truncated_size',b'\x11\3\x0b\1','Truncated',3),('package_past_frame',b'\x11\x3f\0','BadEncoding',1)]:add(label,data,error,offset=offset)
 add('buffer_active_frame',package(integer(2)+b'AB'),'BadEncoding',frame_end=4,offset=1)
 add('unsupported_integer',b'\x0a\x01','UnsupportedSyntax');add('unsupported_package',b'\x12\1','UnsupportedSyntax');add('unsupported_prefix',b'\x5b','UnsupportedSyntax')
 add('empty_input',b'','Truncated');add('at_end',b'\x0d\0','Truncated',at=2)
 for label,kw in [('unit',dict(frame_unit=8)),('source_length',dict(source_length=1025)),('source_max',dict(source_length=MAX)),('frame_past_source',dict(frame_end=3)),('frame_max',dict(frame_end=MAX)),('offset_past_frame',dict(at=3)),('offset_max',dict(at=MAX))]:add('invalid_'+label,b'\x0d\0','InvalidState',**kw)
 string('maximum_unit',[65],source_unit=MAX,frame_unit=MAX)
 return rows

IMPORTS='''use execution::byte_literals::preflight_byte_literal;
use aml::model::Outcome;
use aml::model::Value;
use aml::model::ValueRead;
use aml::model::Span;
use aml::model::ObjectStore;
use aml::model::StringStorage;
use aml::model::BufferStorage;
use aml::byte_storage::ByteRead;
use aml::byte_storage::ByteOutcome;
use aml::byte_storage::read_bytes;
use integer_helpers::integers::IntegerSize;
'''
HELPERS='''machine literal_test_bytes(a:&[u8;256],b:&[u8;256],index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {
 let equal:bool=literal_test_byte(a,b,index);
 transition index<count {true -> literal_test_bytes(a,b,index+1,count,prior && equal) _ -> (prior)}
}
machine literal_test_byte(a:&[u8;256],b:&[u8;256],index:u64)->bool {transition index<256 {true -> (a[index]==b[index]) _ -> (true)}}
machine literal_test_source(value:Value,string:bool,start:u64,end:u64,unit:u64,declared:u64)->bool {
 transition value {Value::String {string_storage} -> check_string(string_storage,string,start,end,unit)
 Value::Buffer {buffer_storage} -> check_buffer(buffer_storage,string,start,end,unit,declared) _ -> (false)}
 state check_string(storage:StringStorage,string:bool,start:u64,end:u64,unit:u64)->bool {transition storage {StringStorage::Source {string_source} -> (string && string_source.start==start && string_source.end==end && string_source.unit==unit) _ -> (false)}}
 state check_buffer(storage:BufferStorage,string:bool,start:u64,end:u64,unit:u64,declared:u64)->bool {transition storage {BufferStorage::Source {declared_size,buffer_initializer} -> (!string && declared_size==declared && buffer_initializer.start==start && buffer_initializer.end==end && buffer_initializer.unit==unit) _ -> (false)}}
}
machine literal_test_empty(value:Value)->bool {transition value {Value::Uninitialized -> (true) _ -> (false)}}
'''
def render(row,control=False,machine='test_result()'):
 body='let mut input:[u8;1024];\n'+''.join(f'input[{i}]={v};'for i,v in enumerate(row['data']))+'\n'
 body+=f'let result:ValueRead=preflight_byte_literal(&input,{row["source_length"]},{row["source_unit"]},{row["frame_unit"]},{row["frame_end"]},{row["at"]},IntegerSize::{"FourBytes"if row["bits"]==32 else"EightBytes"});\n'
 if row['outcome']=='Success':
  body+='let mut store:ObjectStore=ObjectStore {};store.space.object_count=1;store.space.objects[0].value=result.value;\n'
  body+=f'let bytes:ByteRead=read_bytes(&input,{row["source_length"]},{row["source_unit"]},&store,0);\nlet mut expected:[u8;256];\n'+''.join(f'expected[{i}]={v};'for i,v in enumerate(row['expected']))+'\n'
  body+=f'let shape:bool=literal_test_source(result.value,{str(row["kind"]=="String").lower()},{row["start"]},{row["end"]},{row["source_unit"]},{row["declared"]});let equal:bool=literal_test_bytes(&bytes.bytes,&expected,0,256,true);\n'
  test=f'result.outcome==Outcome::Success && result.next=={row["next"]+int(control)} && result.offset==0 && shape && bytes.outcome==ByteOutcome::Success && bytes.length=={len(row["expected"])} && equal'
 else:
  body+='let empty:bool=literal_test_empty(result.value);\n'
  test=f'result.outcome==Outcome::{"Success"if control else row["outcome"]} && result.next==0 && result.offset=={row["offset"]} && empty'
 return f'machine {machine}->i32 {{\n'+body+'transition '+test+' {true -> (0) _ -> (1)}\n}\n'

def main():
 import argparse
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();path=HERE/'cases.json';text=json.dumps(cases(),indent=2)+'\n'
 if a.check:assert path.read_text()==text
 else:path.write_text(text)
 print(len(cases()),'byte literal preflight cases')
if __name__=='__main__':main()
