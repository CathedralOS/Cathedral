#!/usr/bin/env python3
"""Compact initialized rows comparing actual pinned public observations."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
HEAD='''// SPDX-License-Identifier: MIT OR Apache-2.0
use x86::page_iterators;
use x86::encrypted_frames::IteratorResult;
use x86::addresses::NumberResult;
data Row [copy] {start:u64;end:u64;size:u64;inclusive:bool;index:u64;reverse:bool;ok:bool;value:u64;failed:bool;out_start:u64;out_end:u64;}
data Rows [copy] {values:[Row;32];}
machine compare(row:Row)->bool {
 let actual:IteratorResult=page_iterators::step(row.start,row.end,row.size,row.inclusive,row.index,row.reverse);
 let cursor:bool=actual.start==row.out_start && actual.end==row.out_end && actual.failed==row.failed;
 transition actual.selection {
  NumberResult::Value{value} -> (cursor && row.ok && value==row.value)
  _ -> (cursor && !row.ok)
 }
}
machine run(rows:&Rows,index:u64,count:u64,good:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {
 let next:bool=at(rows,index,count,good);
 transition index<count {true -> run(rows,index+1,count,next) _ -> (good)}
}
machine at(rows:&Rows,index:u64,count:u64,good:bool)->bool {
 transition index<count && index<32 {true -> item(rows.values[index],good) _ -> (good)}
 state item(row:Row,good:bool)->bool {let result:bool=compare(row);good && result}
}
'''
FOOT='\nconst TEST_RESULT:i32=test_result();\nmachine require_success(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_success(TEST_RESULT);}\n'
def rows():
 inputs=json.loads((HERE/'inputs.json').read_text());observations=[json.loads(s)for s in (HERE/'reference.jsonl').read_text().splitlines()]
 assert len(inputs)==len(observations)
 result=[]
 for n,(i,o)in enumerate(zip(inputs,observations)):
  assert o['id']==n
  result.append(dict(i,**o))
 for size in [0,3,8192,2**64-1]:
  result.append(dict(id=len(result),size=size,start=0,end=0,inclusive=True,index=0,reverse=False,direct=False,family='invalid-geometry',ok=False,value=0,failed=True,out_start=0,out_end=0))
 return result
def groups():
 items=rows();return [dict(name=f'group_{n//32:03}',rows=items[n:n+32])for n in range(0,len(items),32)]
def render(group,control=False):
 name=group['name']+('_control'if control else'_positive')
 text=f'machine Suite::{name}(&mut self)->i32 {{\n let values:Rows=Rows {{}};\n'
 for n,row in enumerate(group['rows']):
  r={k:v for k,v in row.items()if k not in ['id','direct','family']}
  if control and n==0:r['failed']=not r['failed']
  text+=f' values.values[{n}]=Row {{'+','.join(f'{k}:{str(v).lower()}'for k,v in r.items())+'};\n'
 text+=f' let good:bool=run(&values,0,{len(group["rows"])},true);transition good {{true -> (0) _ -> (1)}}\n}}\n'
 return text
def const_source(group,control=False):
 body=render(group,control);name=group['name']+('_control'if control else'_positive')
 return HEAD+body.replace('machine Suite::'+name+'(&mut self)','machine test_result()')+FOOT
def write(path,text):
 if '--check' in sys.argv:assert path.read_text()==text,('stale',path)
 else:path.write_text(text)
if __name__=='__main__':
 g=groups();summary=[dict(name=row['name'],first=row['rows'][0]['id'],count=len(row['rows']))for row in g]
 write(HERE/'groups.json',json.dumps(summary,indent=2)+'\n')
 print(sum(len(x['rows'])for x in g),'rows in',len(g),'compact groups and body controls')
