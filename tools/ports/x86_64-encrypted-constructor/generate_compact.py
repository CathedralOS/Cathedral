from pathlib import Path
import json,subprocess,os
HERE=Path(__file__).resolve().parent;root=HERE.parents[2];src=HERE;work=HERE
rows=[json.loads(l)for l in (src/'reference.jsonl').read_text().splitlines()]
text='''use x86::encrypted_recursive_constructor;
use x86::mapper_topology::RecursiveObservation;
use x86::mapper_topology::InactiveReason;
use x86::memory_encryption;
use x86::memory_encryption::State;
use x86::memory_encryption::Configuration;
data Case [copy] {address:u64;observed:u64;word:u64;kind:u8;index:u16;frame:u64;}
data Suite {}
machine one(profile:State,row:Case,mutate:bool)->bool {
 let observed:RecursiveObservation=encrypted_recursive_constructor::observe_recursive(profile,row.address,row.observed,row.word);
 transition observed {
  RecursiveObservation::InvalidAddress -> (row.kind==5)
  RecursiveObservation::InvalidObservedFrame -> (row.kind==6)
  RecursiveObservation::NotRecursive -> (row.kind==1)
  RecursiveObservation::NotActive {reason} -> inactive(reason,row.kind)
  RecursiveObservation::Observed {recursive_index,physical_frame} -> (row.kind==0 && recursive_index==row.index && ((physical_frame==row.frame)!=mutate))
 }
 state inactive(reason:InactiveReason,kind:u8)->bool {transition reason {InactiveReason::EntryNotPresent -> (kind==2) InactiveReason::EntryHugeFrame -> (kind==3) _ -> (kind==4)}}
}
machine loop_rows(profile:State,rows:&[Case;61],index:u64,count:u64,mutate:bool,valid:bool)
terminates by (index,count)->Nat::BoundedDistance;
->bool {let next:bool=row_at(profile,rows,index,count,mutate,valid);transition index<count {true -> loop_rows(profile,rows,index+1,count,mutate,next) _ -> (valid)}}
machine row_at(profile:State,rows:&[Case;61],index:u64,count:u64,mutate:bool,valid:bool)->bool {
 transition index<count && index<61 {true -> compare(profile,rows[index],mutate,valid) _ -> (valid)}
 state compare(profile:State,row:Case,mutate:bool,valid:bool)->bool {let good:bool=one(profile,row,mutate);valid && good}
}
'''
selections=[]
for start in range(0,610,61):
 group=rows[start:start+61];profile=group[0]['profile']
 for mutation in [False,True]:
  name=f'p{start//61}_'+('control'if mutation else'positive');selections.append('Suite::'+name+'='+str(int(mutation)))
  text+=f'machine Suite::{name}(&mut self)->i32 {{let profile:State=memory_encryption::initial();\n'
  if profile!='disabled':
   for n,part in enumerate(profile.split('_')):
    case='EncryptedBit'if part[0]=='e'else'SharedBit';text+=f'let configured{n}:bool=memory_encryption::configure(&mut profile,Configuration::{case}{{position:{part[1:]}}});\n'
  text+='let mut rows:[Case;61];\n'
  for i,r in enumerate(group):text+=f'rows[{i}]=Case{{address:{r["address"]},observed:{r["observed"]},word:{r["word"]},kind:{r["kind"]},index:{r["index"]},frame:{r["frame"]}}};\n'
  text+=f'let valid:bool=loop_rows(profile,&rows,0,61,{str(mutation).lower()},true);transition valid {{true -> (0) _ -> (1)}} }}\n'
import sys
outputs={HERE/'compact_suite.omg':text,HERE/'compact_selections.json':json.dumps(selections,indent=2)+'\n'}
for path,content in outputs.items():
 if '--check' in sys.argv:assert path.read_text()==content,path
 else:path.write_text(content)
print('610 constructor rows in10 shared-loop groups with10 frame comparison controls')
