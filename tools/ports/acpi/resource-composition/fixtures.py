#!/usr/bin/env python3
"""Original bounded resource concatenation vectors; full output atomicity checks."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[3];PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
IMPORTS='''use composition::concatenate;
use composition::concatenate::Composition;
use resources::resource_model::Outcome;
'''
HELPERS='''machine fill(bytes:&mut [u8;4096],index:u64,count:u64,value:u8)
terminates by (index,count)->Nat::BoundedDistance;
->u8 {_=fill_at(bytes,index,count,value);transition index<count {true -> fill(bytes,index+1,count,value) _ -> (0)}}
machine fill_at(bytes:&mut [u8;4096],index:u64,count:u64,value:u8)->u8 {transition index<count && index<4096 {true -> write(bytes,index,value) _ -> (0)} state write(bytes:&mut [u8;4096],index:u64,value:u8)->u8 {bytes[index]=value;0}}
machine equal(a:&[u8;4096],b:&[u8;4096],index:u64,count:u64,valid:bool)
terminates by (index,count)->Nat::BoundedDistance;
->bool {let next:bool=equal_at(a,b,index,count,valid);transition index<count {true -> equal(a,b,index+1,count,next) _ -> (valid)}}
machine equal_at(a:&[u8;4096],b:&[u8;4096],index:u64,count:u64,valid:bool)->bool {transition valid && index<count && index<4096 {true -> read(a[index],b[index]) _ -> (valid)} state read(a:u8,b:u8)->bool {a==b}}
'''
def cases():
 end=[121,0];vendor=[113,7];irq=[34,1,0];large=[132,251,15]+[0]*4091
 rows=[]
 def add(name,left,right,result='Composed',output=None,unknown=0,capacity=4096,a=None,b=None,reason=None):
  rows.append(dict(name=name,left=left,right=right,a=len(left)if a is None else a,b=len(right)if b is None else b,result=result,output=output or [],unsupported=unknown,capacity=capacity,reason=reason))
 add('empty',[],[],output=end)
 add('empty_left',[],irq+end,output=irq+end)
 add('empty_right',irq+end,[],output=irq+end)
 add('two_endtags',end,end,output=end)
 add('checked_input',[121,135],irq+end,output=irq+end)
 add('two_resources',irq+end,vendor+end,output=irq+vendor+end,unknown=1)
 add('two_unsupported',vendor+end,vendor+end,output=vendor+vendor+end,unknown=2)
 add('exact_capacity',irq+end,vendor+end,output=irq+vendor+end,unknown=1,capacity=7)
 add('short_capacity',irq+end,vendor+end,result='Capacity',capacity=6)
 add('zero_capacity',[],[],result='Capacity',capacity=0)
 add('capacity_over',[],[],result='Capacity',capacity=4097)
 add('capacity_max',[],[],result='Capacity',capacity=2**64-1)
 add('left_length_max',[],[],result='Capacity',a=2**64-1)
 add('right_length_max',[],[],result='Capacity',b=2**64-1)
 add('left_one',[121],[],result='InvalidLeft',reason='BadEncoding')
 add('right_one',[],[121],result='InvalidRight',reason='BadEncoding')
 add('left_missing',irq,[],result='InvalidLeft',reason='MissingEndTag')
 add('right_missing',end,irq,result='InvalidRight',reason='MissingEndTag')
 add('bad_checksum',[121,1],[],result='InvalidLeft',reason='BadChecksum')
 add('trailing',end+[0],[],result='InvalidLeft',reason='TrailingData')
 add('short_header',[132,0],[],result='InvalidLeft',reason='Truncated')
 add('left_before_right',[121],[121],result='InvalidLeft',reason='BadEncoding')
 add('capacity_before_encoding',[121],[121],result='Capacity',capacity=4097)
 add('full',large+end,[],output=large+end,unknown=1)
 add('full_overflow',large+end,vendor+end,result='Capacity')
 return rows

def render(r,control=False,machine='test_result'):
 signature=machine+'(&mut self)'if'::'in machine else machine+'()'
 s=f'machine {signature}->i32 {{\nlet mut left:[u8;4096];let mut right:[u8;4096];let mut output:[u8;4096];let mut expected:[u8;4096];\n_=fill(&mut output,0,4096,90);_=fill(&mut expected,0,4096,90);\n'
 for name in ['left','right']:
  for i,v in enumerate(r[name]):
   if v:s+=f'{name}[{i}]={v};\n'
 if r['result']=='Composed':
  s+=f'_=fill(&mut expected,0,{len(r["output"])},0);\n'
  for i,v in enumerate(r['output']):
   if v:s+=f'expected[{i}]={v};\n'
 s+=f'let result:Composition=concatenate::compose(&left,{r["a"]},&right,{r["b"]},{r["capacity"]},&mut output);\nlet bytes_equal:bool=equal(&output,&expected,0,4096,true);\n'
 check='!bytes_equal'if control else'bytes_equal'
 if r['result']=='Composed':pattern='Composition::Composed{length,unsupported}';check+=f' && length=={len(r["output"])} && unsupported=={r["unsupported"]}'
 elif r['result']=='Capacity':pattern='Composition::Capacity'
 else:pattern=f'Composition::{r["result"]}{{reason}}';check+=f' && reason==Outcome::{r["reason"]}'
 s+=f'transition result {{{pattern} -> finish({check}) _ -> (1)}} state finish(valid:bool)->i32 {{transition valid {{true -> (0) _ -> (1)}}}} }}\n'
 return s
if __name__=='__main__':
 text=json.dumps(cases(),indent=2)+'\n';p=HERE/'cases.json'
 if '--check'in sys.argv:assert p.read_text()==text
 else:p.write_text(text)
 print(len(cases()),'resource composition scenarios verified')
