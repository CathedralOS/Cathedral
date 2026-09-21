#!/usr/bin/env python3
"""Original bounded numeric-string scenarios; Python provides data, not Omega execution."""
import argparse,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
IMPORTS='''use helpers::integers::IntegerSize;
use helpers::integers::IntegerResult;
use helpers::string_numbers::ParseProfile;
use helpers::string_numbers::NumberFormat;
use helpers::string_numbers::string_to_integer;
use helpers::string_numbers::integer_to_string;
use helpers::string_numbers::buffer_to_numeric_string;
'''
HELPERS='''machine strings_test_fill(output:&mut [u8;256],index:u64,count:u64)
terminates by (index,count)->Nat::BoundedDistance;
->u8 {
 _=strings_test_fill_one(output,index,count);
 transition index<count {true -> strings_test_fill(output,index+1,count) _ -> (0)}
}
machine strings_test_fill_one(output:&mut [u8;256],index:u64,count:u64)->u8 {
 transition index<count && index<256 {true -> write(output,index) _ -> (0)}
 state write(output:&mut [u8;256],index:u64)->u8 {output[index]=90;0}
}
machine strings_test_equal(a:&[u8;256],b:&[u8;256],index:u64,count:u64,valid:bool)
terminates by (index,count)->Nat::BoundedDistance;
->bool {
 let next:bool=strings_test_equal_one(a,b,index,count,valid);
 transition index<count {true -> strings_test_equal(a,b,index+1,count,next) _ -> (valid)}
}
machine strings_test_equal_one(a:&[u8;256],b:&[u8;256],index:u64,count:u64,valid:bool)->bool {
 transition index<count && index<256 {true -> read(a,b,index,valid) _ -> (valid)}
 state read(a:&[u8;256],b:&[u8;256],index:u64,valid:bool)->bool {valid && a[index]==b[index]}
}
'''
def cases():
 rows=[]
 parsed=[('zero',b'0',0),('decimal',b'42',42),('leading_zero_decimal',b'077',77),('hex',b'0xAf',175),('hex_upper',b'0XfF',255),('max32',b'4294967295',4294967295),('above32',b'4294967296',4294967296),('max64',b'18446744073709551615',(1<<64)-1),('hex_max64',b'0xffffFFFFFFFFffff',(1<<64)-1),('overflow_decimal',b'18446744073709551616',None),('overflow_hex',b'0x10000000000000000',None),('empty',b'',0),('prefix_only',b'0x',0),('plus',b'+42',0),('minus',b'-42',0),('space',b' 42 ',42),('ascii_space',b'\t\n\v\f\r42\t',42),('only_space',b'\t ',0),('decimal_tail',b'42z',42),('hex_tail',b'0x12G4',18),('leading_letter',b'abc',0),('binary_prefix',b'0b10',0),('underscore',b'1_2',1),('decimal_point',b'1.5',1),('trailing_nul',b'42\0',42),('embedded_nul',b'4\0' + b'2',4),('nonascii',bytes([255]),0),('unicode_space','\u00a042'.encode(),42),('zeros256',b'0'*256,0),('leading_zero_max',b'0'*236+b'18446744073709551615',(1<<64)-1)]
 for name,data,pinned in parsed:
  for width in [32,64]:
   for profile in ['StrictDecimalHex','PinnedAscii']:
    error=0;value=pinned or 0
    if any(b==0 or b>=128 for b in data):error=7
    elif profile=='StrictDecimalHex':
     if not data:error=6
     else:
      text=data.decode();digits=text[2:]if text[:2].lower()=='0x'else text;base=16 if text[:2].lower()=='0x'else 10
      if not digits or any(c not in ('0123456789abcdefABCDEF'if base==16 else'0123456789')for c in digits):error=7
      else:
       value=int(digits,base)
       if value>(1<<width)-1:error=3
    elif pinned is None:error=3
    rows.append(dict(name=f'parse_{name}_{width}_{profile}',kind='parse',data=list(data),length=len(data),width=width,profile=profile,error=error,value=0 if error else value,pin_error=3 if pinned is None else 0,pin_value=pinned or 0))
 for length in [257,(1<<64)-1]:rows.append(dict(name=f'parse_capacity_{length}',kind='parse',data=[],length=length,width=64,profile='PinnedAscii',error=4,value=0))
 for width in [32,64]:
  for value in [0,1,9,10,15,16,255,256,4294967295,4294967296,(1<<64)-1]:
   for fmt in ['Decimal','Hexadecimal']:
    number=value&((1<<width)-1);expected=str(number)if fmt=='Decimal'else f'0x{number:X}'
    rows.append(dict(name=f'integer_{width}_{value}_{fmt}',kind='integer',width=width,value=value,format=fmt,capacity=256,error=0,text=expected))
 for name,data in [('empty',[]),('zero',[0]),('boundaries',[0,1,9,10,15,16,99,100,255]),('zero_tail',[42,0]),('exact_decimal',[100]*63+[10,1]),('hex_fit',[255]*51),('hex_excess',[255]*52),('too_long',[255]*256),('single255',[255])]:
  for fmt in ['Decimal','Hexadecimal']:
   expected=','.join(str(v)if fmt=='Decimal'else f'0x{v:02X}'for v in data)
   rows.append(dict(name=f'buffer_{name}_{fmt}',kind='buffer',data=data,length=len(data),format=fmt,capacity=256,error=0 if len(expected)<=256 else 4,text=expected if len(expected)<=256 else''))
 for kind in ['integer','buffer']:
  for capacity in [0,1,3,4,5,257,(1<<64)-1]:
   rows.append(dict(name=f'{kind}_capacity_{capacity}',kind=kind,width=64,value=255,data=[255],length=1,format='Hexadecimal',capacity=capacity,error=0 if capacity>=4 and capacity<=256 else 4,text='0xFF'if capacity>=4 and capacity<=256 else''))
 for length in [257,(1<<64)-1]:rows.append(dict(name=f'buffer_input_capacity_{length}',kind='buffer',data=[],length=length,format='Decimal',capacity=256,error=4,text=''))
 return rows

def render(row,control=False,machine=None):
 name=machine or 'test_result';head=f'machine {name}'+('(&mut self)'if'::'in name else'()')+'->i32 {\n';body=''
 def array(data,name):return f'let mut {name}:[u8;256];\n'+''.join(f'{name}[{i}]={v};\n'for i,v in enumerate(data)if v)
 size='IntegerSize::FourBytes'if row.get('width')==32 else'IntegerSize::EightBytes'
 if row['kind']=='parse':
  body+=array(row['data'],'input')+f"let result:IntegerResult=string_to_integer({size},&input,{row['length']},ParseProfile::{row['profile']});\n"
  condition=f"result.error=={row['error']} && result.value=={row['value']}"
  if control:condition=condition.replace(f"result.error=={row['error']}",f"result.error=={(row['error']+1)%8}")
 else:
  body+='let mut output:[u8;256];let mut expected:[u8;256];\n_=strings_test_fill(&mut output,0,256);_=strings_test_fill(&mut expected,0,256);\n'
  if row['kind']=='integer':body+=f"let result:IntegerResult=integer_to_string({size},{row['value']},NumberFormat::{row['format']},{row['capacity']},&mut output);\n"
  else:body+=array(row['data'],'input')+f"let result:IntegerResult=buffer_to_numeric_string(&input,{row['length']},NumberFormat::{row['format']},{row['capacity']},&mut output);\n"
  if not row['error']:body+=''.join(f'expected[{i}]={v};\n'for i,v in enumerate(row['text'].encode()))
  body+='let equal:bool=strings_test_equal(&output,&expected,0,256,true);\n'
  condition=f"equal && result.error=={row['error']} && result.value=={len(row['text']) if not row['error'] else 0}"
  if control:condition=condition.replace('equal &&','!equal &&',1)
 return head+body+'transition '+condition+' {true -> (0) _ -> (1)}\n}\n'

def main():
 parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');args=parser.parse_args();rows=cases()
 source=IMPORTS+HELPERS+render(rows[1])+'''const TEST_RESULT:i32=test_result();
machine require_ok(value:i32) requires value==0;{}
data Main{}
machine Main::main(&mut self){require_ok(TEST_RESULT);}
'''
 for path,text in [(HERE/'cases.json',json.dumps(rows,indent=2)+'\n'),(HERE/'main.omg',source)]:
  if args.check:assert path.read_text()==text,path
  else:path.write_text(text)
 print(len(rows),'string/number scenarios', 'verified'if args.check else'generated')
if __name__=='__main__':main()
