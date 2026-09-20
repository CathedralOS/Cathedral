#!/usr/bin/env python3
"""Render actual pinned mapper observations into Omega semantic fixtures."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
PREFIX='''use x86::encrypted_translation;
use x86::memory_encryption;
use x86::memory_encryption::State;
use x86::memory_encryption::Configuration;
use x86::translation::Translation;
use x86::translation::CapturedWalk;
use x86::mapping_routes::CapturedPath;
machine matches_encrypted_result(value:Translation, kind:u8, want_frame:u64, want_size:u64, want_offset:u64, want_flags:u64)->bool {
 transition value {
  Translation::NotMapped -> (kind==0)
  Translation::InvalidRootHugePage -> (kind==2)
  Translation::Mapped { frame, size, offset, flags } ->
   (kind==1 && frame==want_frame && size==want_size && offset==want_offset && flags==want_flags)
  _ -> (false)
 }
}
machine captured(observation:CapturedWalk, kind:u8, frame:u64, size:u64, offset:u64, flags:u64)->bool {
 transition observation {
  CapturedWalk::Result { value } -> check(value,kind,frame,size,offset,flags)
  _ -> (false)
 }
 state check(inner:Translation,kind:u8,frame:u64,size:u64,offset:u64,flags:u64) {
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
   body+=f'let value:Translation=encrypted_translation::translate_words(&profile,{row["va"]},{p4},{p3},{p2},{p1});\nlet good:bool=matches_encrypted_result(value,{args});\n'
   body+=f'let path:CapturedPath=CapturedPath {{ table_ids:[0,{",".join(map(str,row["ids"]))}],words:[{p4},{p3},{p2},{p1}] }};\n'
   body+=f'let walk:CapturedWalk=encrypted_translation::walk_path(&profile,{row["va"]},&path);\nlet same:bool=captured(walk,{args});\ngood && same\n}}\n'
  body+='machine scenario()->i32 {\n'+''.join(f'let check{n}:bool=row{n}();\n' for n in range(len(group)))
  body+='transition '+' && '.join(f'check{n}' for n in range(len(group)))+' { true -> (0) _ -> (1) } }\n'+TAIL
  path=folder/f'batch-{start//6:03}.omg';expected.add(path.name)
  if '--check' in sys.argv:assert path.read_text()==body,path
  else:path.write_text(body)
 assert {p.name for p in folder.glob('*.omg')}==expected
 print(f'{len(rows)} actual Rust observations; {len(expected)} Omega fixtures with both word and captured-ID calls')
if __name__=='__main__':main()
