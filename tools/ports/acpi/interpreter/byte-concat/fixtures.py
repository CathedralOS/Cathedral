#!/usr/bin/env python3
"""Original same-type concatenate cases, with all 256 output bytes checked."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5';MAX=2**64-1
IMPORTS='''use helpers::integers::IntegerSize;
use helpers::integers::IntegerResult;
use helpers::byte_concat::concat_integers;
use helpers::byte_concat::concat_buffers;
use helpers::byte_concat::concat_strings;
'''
HELPERS='''machine fixture_fill(output:&mut [u8;256],seed:u64,index:u64,limit:u64)
terminates by (index,limit)->Nat::BoundedDistance;
->u8 { _ = fixture_fill_byte(output,seed,index,limit);transition index<limit {true -> fixture_fill(output,seed,index+1,limit) _ -> (0)} }
machine fixture_fill_byte(output:&mut [u8;256],seed:u64,index:u64,limit:u64)->u8 {
 transition index<limit && index<256 && seed<=255 {true -> store(output,seed,index) _ -> (0)}
 state store(output:&mut [u8;256],seed:u64,index:u64) {output[index]=((index*17+seed)&255)as u8;0}
}
machine fixture_equal(actual:&[u8;256],expected:&[u8;256],index:u64,limit:u64,prior:bool)
terminates by (index,limit)->Nat::BoundedDistance;
->bool {let good:bool=fixture_equal_byte(actual,expected,index,limit);transition index<limit {true -> fixture_equal(actual,expected,index+1,limit,prior && good) _ -> (prior)}}
machine fixture_equal_byte(actual:&[u8;256],expected:&[u8;256],index:u64,limit:u64)->bool {
 transition index<limit && index<256 {true -> (actual[index]==expected[index]) _ -> (true)}
}
'''
def cases():
 rows=[]
 def add(name,kind,left,right,capacity=256,a=None,b=None,width=8):
  a=len(left)if a is None and kind!='integer'else a;b=len(right)if b is None and kind!='integer'else b
  row=dict(name=name,kind=kind,left=left,right=right,capacity=capacity,a=a,b=b,width=width)
  error=0;data=[]
  if kind=='integer':
   if capacity>256 or capacity<2*width:error=4
   else:data=list((left&((1<<(width*8))-1)).to_bytes(width,'little')+(right&((1<<(width*8))-1)).to_bytes(width,'little'))
  elif a>256 or b>256 or capacity>256:error=4
  elif kind=='string'and any(x==0 or x>127 for x in left[:a]+right[:b]):error=7
  elif a+b>capacity:error=4
  else:data=left[:a]+right[:b]
  row.update(error=error,value=len(data),bytes=data);rows.append(row)
 for width in [4,8]:
  for n,(left,right)in enumerate([(0,0),(MAX,MAX),(0xfedcba9876543210,0x0123456789abcdef),(0x8000000000000001,0xffffffff00000000)]):add(f'integer_{width}_{n}','integer',left,right,2*width if n%2 else 256,width=width)
  for n,capacity in enumerate([0,2*width-1,257,MAX]):add(f'integer_capacity_{width}_{n}','integer',MAX,1,capacity,width=width)
 pattern=[(i*29+7)&255 for i in range(256)];ascii_bytes=[33+i%90 for i in range(256)]
 for name,left,right,capacity in [('empty',[],[],0),('left_empty',[],[0,255],2),('right_empty',[128,0],[],2),('small',[1,2],[3,4],4),('tail',[1,2],[3],256),('too_small',[1,2],[3,4],3),('zero_capacity',[1],[],0),('left_full',pattern,[],256),('right_full',[],pattern,256),('halves',pattern[:128],pattern[128:],256),('last_byte',pattern[:255],[255],256),('first_byte',[255],pattern[:255],256),('overflow',pattern,[0],256),('overflow_halves',pattern[:128],pattern[:129],256)]:add('buffer_'+name,'buffer',left,right,capacity)
 for kind in ['buffer','string']:
  for n,(a,b,capacity)in enumerate([(257,0,256),(MAX,0,256),(0,257,256),(0,MAX,256),(0,0,257),(0,0,MAX),(MAX,MAX,MAX)]):add(f'{kind}_domain_{n}',kind,[0],[255],capacity,a,b)
 for name,left,right,capacity in [('empty',[],[],0),('left_empty',[],[65],1),('right_empty',[65],[],1),('small',[65,66],[67,68],4),('tail',[65],[66],256),('halves',ascii_bytes[:128],ascii_bytes[128:],256),('left_full',ascii_bytes,[],256),('right_full',[],ascii_bytes,256),('too_small',[65,66],[67,68],3),('overflow',ascii_bytes,[65],256),('left_nul',[0],[65],256),('right_nul',[65],[0],256),('left_late_nul',[65]*255+[0],[],256),('right_late_nonascii',[],[65]*255+[255],256),('left_late_nonascii',[65]*255+[128],[],256),('invalid_beats_capacity',[65],[66,0],0),('both_invalid',[255],[65]*255+[0],256),('domain_beats_encoding',[0],[128],257),('nul_outside_logical',[65,0],[66,255],2)]:
  add('string_'+name,'string',left,right,capacity,1 if name=='nul_outside_logical'else None,1 if name=='nul_outside_logical'else None)
 return rows

def render(row,control=False,machine='test_result'):
 signature=machine+'(&mut self)'if'::'in machine else machine+'()'
 source=f'machine {signature}->i32 {{\nlet mut output:[u8;256];let mut expected:[u8;256];\n_ = fixture_fill(&mut output,173,0,256);_ = fixture_fill(&mut expected,173,0,256);\n'
 for index,value in enumerate(row['bytes']):source+=f'expected[{index}]={value};'
 if control:source+='expected[255]=expected[255]^1;' # Mutate a real full-output assertion, including untouched-tail checks.
 if row['kind']=='integer':call=f'concat_integers(IntegerSize::{"FourBytes"if row["width"]==4 else"EightBytes"},{row["left"]},{row["right"]},&mut output,{row["capacity"]})'
 else:
  source+='let mut left:[u8;256];let mut right:[u8;256];_ = fixture_fill(&mut left,11,0,256);_ = fixture_fill(&mut right,27,0,256);\n'
  for var in ['left','right']:
   for index,value in enumerate(row[var]):source+=f'{var}[{index}]={value};'
  call=f'concat_{"strings"if row["kind"]=="string"else"buffers"}(&left,{row["a"]},&right,{row["b"]},&mut output,{row["capacity"]})'
 source+=f'let observed:IntegerResult={call};let all_bytes:bool=fixture_equal(&output,&expected,0,256,true);transition observed.error=={row["error"]} && observed.value=={row["value"]} && observed.remainder==0 && all_bytes {{true -> (0) _ -> (1)}}}}\n'
 return source
if __name__=='__main__':
 paths={HERE/'cases.json':json.dumps(cases(),indent=2)+'\n',HERE/'main.omg':IMPORTS+HELPERS+render(cases()[0])+'''const TEST_RESULT:i32=test_result();
machine require_ok(value:i32) requires value==0;{}
data Main{}
machine Main::main(&mut self){require_ok(TEST_RESULT);}
'''}
 for path,text in paths.items():
  if '--check'in sys.argv:assert path.read_text()==text,path
  else:path.write_text(text)
 print(len(cases()),'concatenate scenarios verified')
