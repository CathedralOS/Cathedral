#!/usr/bin/env python3
"""Generate Omega behavior fixtures from actual pinned numeric Rust witnesses."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
COMMON='''// SPDX-License-Identifier: MIT OR Apache-2.0
// Expected numeric results observed from pinned Rust public APIs/private mirrors.
use values::addresses::NumberResult;
use values::mapper_topology::offset_frame_address;
use values::mapper_topology::recursive_p3;
use values::mapper_topology::recursive_p2;
use values::mapper_topology::recursive_p1;
use values::mapper_topology::observe_recursive;
use values::mapper_topology::RecursiveObservation;
use values::mapper_topology::InactiveReason;
machine number_is(result:NumberResult,valid:bool,expected:u64)->bool {
 transition result {NumberResult::Value {value} -> compare(value,valid,expected) _ -> (!valid)}
 state compare(value:u64,valid:bool,expected:u64)->bool {valid && value==expected}
}
machine observed_is(result:RecursiveObservation,code:u64,index:u16,frame:u64)->bool {
 transition result {
  RecursiveObservation::InvalidAddress -> (code==5)
  RecursiveObservation::InvalidObservedFrame -> (code==6)
  RecursiveObservation::NotRecursive -> (code==1)
  RecursiveObservation::NotActive {reason} -> inactive(reason,code)
  RecursiveObservation::Observed {recursive_index,physical_frame} -> active(recursive_index,physical_frame,code,index,frame)
 }
 state inactive(reason:InactiveReason,code:u64)->bool {transition reason {InactiveReason::EntryNotPresent -> (code==2) InactiveReason::EntryHugeFrame -> (code==3) InactiveReason::DifferentFrame -> (code==4)}}
 state active(actual_index:u16,actual_frame:u64,code:u64,index:u16,frame:u64)->bool {code==0 && actual_index==index && actual_frame==frame}
}
'''
TAIL='''
const RESULT:u64=test();
machine require_ok(value:u64) requires value==0; {}
data Main{}
machine Main::main(&mut self){require_ok(RESULT);}
'''
COORDS='''
data Vector [copy]{recursive_index:u16;page4:u64;page2:u64;page1:u64;third4:u64;third2:u64;third1:u64;second4:u64;second2:u64;first4:u64;}
machine check_vector(row:Vector)->bool {
 let r:u16=row.recursive_index;let p4:u64=row.page4;let p2:u64=row.page2;let p1:u64=row.page1;
 let a:NumberResult=recursive_p3(p4,4096,r);let b:NumberResult=recursive_p3(p2,2097152,r);let c:NumberResult=recursive_p3(p1,1073741824,r);
 let d:NumberResult=recursive_p2(p4,4096,r);let e:NumberResult=recursive_p2(p2,2097152,r);let f:NumberResult=recursive_p1(p4,r);
 let aa:bool=number_is(a,true,row.third4);let bb:bool=number_is(b,true,row.third2);let cc:bool=number_is(c,true,row.third1);
 let dd:bool=number_is(d,true,row.second4);let ee:bool=number_is(e,true,row.second2);let ff:bool=number_is(f,true,row.first4);
 aa && bb && cc && dd && ee && ff
}
machine topology_scan(rows:&[Vector;32],index:u64,limit:u64,ok:bool)
terminates by (index,limit) -> Nat::BoundedDistance;
-> bool {
 let next:bool=topology_step(rows,index,ok);
 transition index<limit {true -> topology_scan(rows,index+1,limit,next) _ -> (ok)}
}
machine topology_step(rows:&[Vector;32],index:u64,ok:bool)->bool {
 transition index<32 {true -> compare(rows,index,ok) _ -> (ok)}
 state compare(rows:&[Vector;32],index:u64,ok:bool)->bool {let passed:bool=check_vector(rows[index]);ok && passed}
}
'''

def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');args=p.parse_args()
 subprocess.run([sys.executable,str(HERE/'generate_reference.py'),'--check'],cwd=ROOT,check=True)
 result=subprocess.run(['cargo','run','--quiet','--locked','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT,capture_output=True,text=True,check=True)
 rows=[line.split('|') for line in result.stdout.splitlines()];coordinates=[[int(x) for x in row[1:]] for row in rows if row[0]=='C'];offsets=[[int(x) for x in row[1:]] for row in rows if row[0]=='O'];constructors=[[int(x) for x in row[1:]] for row in rows if row[0]=='N']
 assert len(coordinates)==512 and {r[0] for r in coordinates}==set(range(512))
 files={};cases={}
 def case(name,source,old,new,count):
  assert source.count(old)==1,(name,old)
  files['cases/'+name+'.omg']=source;cases[name]={'mutation':[old,new],'numeric_checks':count}
 names=['recursive_index','page4','page2','page1','third4','third2','third1','second4','second2','first4']
 for block in range(16):
  group=coordinates[block*32:(block+1)*32];body='let mut rows:[Vector;32];\n'
  for i,row in enumerate(group):body+=f'rows[{i}]=Vector {{'+','.join(f'{name}:{value}' for name,value in zip(names,row))+'};\n'
  body+='let good:bool=topology_scan(&rows,0,32,true);transition good {true -> (0) _ -> (1)}'
  case(f'coordinates-{block:02}',COMMON+COORDS+'\nmachine test()->u64{\n'+body+'\n}\n'+TAIL,f'first4:{group[0][-1]}',f'first4:{group[0][-1]^4096}',192)
 body='';checks=[]
 for i,(offset,frame,valid,value) in enumerate(offsets):
  body+=f'let r{i}:NumberResult=offset_frame_address({offset},{frame});let c{i}:bool=number_is(r{i},{str(bool(valid)).lower()},{value});\n';checks.append(f'c{i}')
 source=COMMON+'\nmachine test()->u64{\n'+body+'transition '+' && '.join(checks)+' {true -> (0) _ -> (1)}\n}\n'+TAIL
 case('offsets',source,'number_is(r0,true,0)','number_is(r0,true,1)',len(offsets))
 for block in range(4):
  group=constructors[block*12:(block+1)*12];body='';checks=[]
  for i,(address,observed,word,code,r) in enumerate(group):
   body+=f'let r{i}:RecursiveObservation=observe_recursive({address},{observed},{word});let c{i}:bool=observed_is(r{i},{code},{r},{observed});\n';checks.append(f'c{i}')
  source=COMMON+'\nmachine test()->u64{\n'+body+'transition '+' && '.join(checks)+' {true -> (0) _ -> (1)}\n}\n'+TAIL
  code=group[0][3];old=f'observed_is(r0,{code},';new=f'observed_is(r0,{(code+1)%5},';case(f'constructor-{block}',source,old,new,len(group))
 body='''let p3:NumberResult=recursive_p3(0,4096,512);let bad_size:NumberResult=recursive_p3(0,8192,1);let giant_p2:NumberResult=recursive_p2(0,1073741824,1);let unaligned:NumberResult=recursive_p1(1,1);let hole:NumberResult=recursive_p1(0x800000000000,1);
let a:bool=number_is(p3,false,0);let b:bool=number_is(bad_size,false,0);let c:bool=number_is(giant_p2,false,0);let d:bool=number_is(unaligned,false,0);let e:bool=number_is(hole,false,0);
let bad_address:RecursiveObservation=observe_recursive(0x800000000000,0,0);let bad_observed:RecursiveObservation=observe_recursive(0,1,0);let nonrecursive:RecursiveObservation=observe_recursive(4096,1,0);
let f:bool=observed_is(bad_address,5,0,0);let g:bool=observed_is(bad_observed,6,0,0);let h:bool=observed_is(nonrecursive,1,0,0);
transition a && b && c && d && e && f && g && h {true -> (0) _ -> (1)}'''
 case('checked-inputs',COMMON+'\nmachine test()->u64{\n'+body+'\n}\n'+TAIL,'observed_is(nonrecursive,1,0,0)','observed_is(nonrecursive,6,0,0)',8)
 files['cases.json']=json.dumps(cases,indent=2,sort_keys=True)+'\n'
 files['reference.json']=json.dumps({'coordinates':coordinates,'offsets':offsets,'constructors':constructors,'private_body_policy':'Exact extracted p3_page/p2_page/p1_page bodies; explicit constructor observation substitutions and offset numeric expression. No private API or live constructor invoked.','numeric_reference_checks':len(coordinates)*6+len(offsets)+len(constructors)},indent=2)+'\n'
 for name,text in files.items():
  path=HERE/name
  if args.check:assert path.read_text()==text,name+' differs from actual pinned reference'
  else:path.write_text(text)
 print(f'{len(cases)} Omega fixtures bind {len(coordinates)*6+len(offsets)+len(constructors)} Rust numeric witnesses plus eight checked-input cases.')
if __name__=='__main__':main()
