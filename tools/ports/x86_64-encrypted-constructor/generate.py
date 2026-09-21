#!/usr/bin/env python3
"""Render pinned observation outcomes, preserving explicit constructor errors."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
PREFIX='''use x86::encrypted_recursive_constructor;
use x86::mapper_topology::RecursiveObservation;
use x86::mapper_topology::InactiveReason;
use x86::memory_encryption;
use x86::memory_encryption::State;
use x86::memory_encryption::Configuration;
machine matches_constructor(value:RecursiveObservation,kind:u8,want_index:u16,want_frame:u64)->bool {
 transition value {
  RecursiveObservation::InvalidAddress -> (kind==5)
  RecursiveObservation::InvalidObservedFrame -> (kind==6)
  RecursiveObservation::NotRecursive -> (kind==1)
  RecursiveObservation::NotActive {reason} -> inactive(reason,kind)
  RecursiveObservation::Observed {recursive_index,physical_frame} -> (kind==0 && recursive_index==want_index && physical_frame==want_frame)
 }
 state inactive(reason:InactiveReason,kind:u8)->bool {
  transition reason {InactiveReason::EntryNotPresent -> (kind==2) InactiveReason::EntryHugeFrame -> (kind==3) _ -> (kind==4)}
 }
}
'''
TAIL='''const RESULT:i32=scenario();
machine require_success(value:i32) requires value==0; {}
data Main{}
machine Main::main(&mut self){require_success(RESULT);}
'''
rows=[json.loads(l) for l in (HERE/'reference.jsonl').read_text().splitlines()];assert len(rows)==610
folder=HERE/'cases';folder.mkdir(exist_ok=True)
for start in range(0,610,61):
 group=rows[start:start+61];profile=group[0]['profile'];assert all(r['profile']==profile for r in group)
 body=PREFIX
 for n,r in enumerate(group):
  body+=f'machine row{n}()->bool {{\nlet profile:State=memory_encryption::initial();\n'
  if profile!='disabled':
   for i,part in enumerate(profile.split('_')):
    case='EncryptedBit' if part[0]=='e' else 'SharedBit';body+=f'let configured{i}:bool=memory_encryption::configure(&mut profile,Configuration::{case} {{position:{part[1:]}}});\n'
  body+=f'let observed:RecursiveObservation=encrypted_recursive_constructor::observe_recursive(profile,{r["address"]},{r["observed"]},{r["word"]});\nmatches_constructor(observed,{r["kind"]},{r["index"]},{r["frame"]}) }}\n'
 body+='machine scenario()->i32 {\n'
 for n in range(61):body+=f'let pass{n}:bool=row{n}();\n'
 body+='transition '+' && '.join(f'pass{n}' for n in range(61))+' {true -> (0) _ -> (1)} }\n'+TAIL
 path=folder/f'batch-{start//61:02}.omg'
 if '--check' in sys.argv:assert path.read_text()==body,path
 else:path.write_text(body)
print('610 constructor observations;10 fixtures')
