#!/usr/bin/env python3
"""Render pinned recursive-body observations into Omega semantic fixtures."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
PREFIX='''use x86::encrypted_recursive_translation;
use x86::recursive_translation::RecursiveTranslation;
use x86::memory_encryption;
use x86::memory_encryption::State;
use x86::memory_encryption::Configuration;
use x86::translation::Translation;
machine matches_encrypted_result(value:Translation, kind:u8, want_frame:u64, want_size:u64, want_offset:u64, want_flags:u64)->bool {
 transition value {
  Translation::NotMapped -> (kind==0)
  Translation::InvalidRootHugePage -> (kind==2)
  Translation::Mapped { frame, size, offset, flags } ->
   (kind==1 && frame==want_frame && size==want_size && offset==want_offset && flags==want_flags)
  _ -> (false)
 }
}
machine matches_recursive_profile(observation:RecursiveTranslation, kind:u8, frame:u64, size:u64, offset:u64, flags:u64)->bool {
 transition observation {
  RecursiveTranslation::InvalidLeafHugePage -> (kind==3)
  RecursiveTranslation::Result {value} -> compare(value,kind,frame,size,offset,flags)
 }
 state compare(inner:Translation,kind:u8,frame:u64,size:u64,offset:u64,flags:u64) {
  let valid:bool=matches_encrypted_result(inner,kind,frame,size,offset,flags);valid
 }
}
'''
TAIL='''const RESULT:i32=scenario();
machine require_success(value:i32) requires value==0; {}
data Main{}
machine Main::main(&mut self){require_success(RESULT);}
'''
def profile(name):
 text='let profile:State=memory_encryption::initial();\n'
 if name!='disabled':
  for n,part in enumerate(name.split('_')):
   variant='EncryptedBit' if part[0]=='e' else 'SharedBit'
   text+=f'let configured{n}:bool=memory_encryption::configure(&mut profile,Configuration::{variant} {{ position:{part[1:]} }});\n'
 return text
def main():
 rows=[json.loads(line) for line in (HERE/'reference.jsonl').read_text().splitlines()];assert len(rows)==330
 folder=HERE/'cases';folder.mkdir(exist_ok=True);expected=set()
 for start in range(0,len(rows),6):
  group=rows[start:start+6];body=PREFIX
  for n,row in enumerate(group):
   p4,p3,p2,p1=row['words'];args=','.join(str(row[x]) for x in ['kind','frame','size','offset','flags'])
   body+=f'machine row{n}()->bool {{\n'+profile(row['profile'])
   body+=f'let observation:RecursiveTranslation=encrypted_recursive_translation::translate_words(&profile,{row["va"]},{p4},{p3},{p2},{p1});\nlet good:bool=matches_recursive_profile(observation,{args});\ngood\n}}\n'
  body+='machine scenario()->i32 {\n'+''.join(f'let check{n}:bool=row{n}();\n' for n in range(len(group)))
  body+='transition '+' && '.join(f'check{n}' for n in range(len(group)))+' { true -> (0) _ -> (1) } }\n'+TAIL
  path=folder/f'batch-{start//6:03}.omg';expected.add(path.name)
  if '--check' in sys.argv:assert path.read_text()==body,path
  else:path.write_text(body)
 assert {p.name for p in folder.glob('*.omg')}==expected
 print(f'{len(rows)} adapted recursive Rust observations; {len(expected)} Omega fixtures with actual recursive profile calls')
if __name__=='__main__':main()
