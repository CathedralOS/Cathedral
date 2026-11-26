from pathlib import Path
import json,subprocess,os
HERE=Path(__file__).resolve().parent;root=HERE.parents[2];src=HERE;work=HERE
rows=[json.loads(l)for l in (src/'reference.jsonl').read_text().splitlines()]
text='''use {omega}/language/core/nat.omg;
use {x86}/encrypted_recursive_constructor.omg;
use {x86}/mapper_topology.omg;
use {x86}/memory_encryption.omg;
data Case [copy] {address:u64;observed:u64;word:u64;kind:u8;index:u16;frame:u64;}
data Suite {}
machine one(profile:|memory_encryption|::State,row:Case,mutate:bool)->bool {
 let observed:|mapper_topology|::RecursiveObservation=|encrypted_recursive_constructor|::observe_recursive(profile,row.address,row.observed,row.word);
 transition observed {
  |mapper_topology|::RecursiveObservation::InvalidAddress -> (row.kind==5)
  |mapper_topology|::RecursiveObservation::InvalidObservedFrame -> (row.kind==6)
  |mapper_topology|::RecursiveObservation::NotRecursive -> (row.kind==1)
  |mapper_topology|::RecursiveObservation::NotActive {reason} -> inactive(reason,row.kind)
  |mapper_topology|::RecursiveObservation::Observed {recursive_index,physical_frame} -> (row.kind==0 && recursive_index==row.index && ((physical_frame==row.frame)!=mutate))
 }
 state inactive(reason:|mapper_topology|::InactiveReason,kind:u8)->bool {transition reason {|mapper_topology|::InactiveReason::EntryNotPresent -> (kind==2) |mapper_topology|::InactiveReason::EntryHugeFrame -> (kind==3) _ -> (kind==4)}}
}
machine loop_rows(profile:|memory_encryption|::State,rows:&[Case;61],index:u64,count:u64,mutate:bool,valid:bool)
terminates by (index,count)->|nat|::Nat::BoundedDistance;
->bool {let next:bool=row_at(profile,rows,index,count,mutate,valid);transition index<count {true -> loop_rows(profile,rows,index+1,count,mutate,next) _ -> (valid)}}
machine row_at(profile:|memory_encryption|::State,rows:&[Case;61],index:u64,count:u64,mutate:bool,valid:bool)->bool {
 transition index<count && index<61 {true -> compare(profile,rows[index],mutate,valid) _ -> (valid)}
 state compare(profile:|memory_encryption|::State,row:Case,mutate:bool,valid:bool)->bool {let good:bool=one(profile,row,mutate);valid && good}
}
'''
selections=[]
for start in range(0,610,61):
 group=rows[start:start+61];profile=group[0]['profile']
 for mutation in [False,True]:
  name=f'p{start//61}_'+('control'if mutation else'positive');selections.append('Suite::'+name+'='+str(int(mutation)))
  text+=f'machine Suite::{name}(&mut self)->i32 {{let profile:|memory_encryption|::State=|memory_encryption|::initial();\n'
  if profile!='disabled':
   for n,part in enumerate(profile.split('_')):
    case='EncryptedBit'if part[0]=='e'else'SharedBit';text+=f'let configured{n}:bool=|memory_encryption|::configure(&mut profile,|memory_encryption|::Configuration::{case}{{position:{part[1:]}}});\n'
  text+='let mut rows:[Case;61];\n'
  for i,r in enumerate(group):text+=f'rows[{i}]=Case{{address:{r["address"]},observed:{r["observed"]},word:{r["word"]},kind:{r["kind"]},index:{r["index"]},frame:{r["frame"]}}};\n'
  text+=f'let valid:bool=loop_rows(profile,&rows,0,61,{str(mutation).lower()},true);transition valid {{true -> (0) _ -> (1)}} }}\n'
import sys
outputs={HERE/'compact_suite.omg':text,HERE/'compact_selections.json':json.dumps(selections,indent=2)+'\n'}
for path,content in outputs.items():
 if '--check' in sys.argv:assert path.read_text()==content,path
 else:path.write_text(content)
print('610 constructor rows in10 shared-loop groups with10 frame comparison controls')
