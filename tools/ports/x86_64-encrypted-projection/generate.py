#!/usr/bin/env python3
"""Render real Translate default outcomes as profile projection fixtures."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
PREFIX='''use x86::encrypted_address_projection;
use x86::translation::Translation;
use x86::addresses::NumberResult;
use x86::memory_encryption;
use x86::memory_encryption::State;
use x86::memory_encryption::Configuration;
machine projection_matches(result:NumberResult,valid:bool,expected:u64)->bool {
 transition result { NumberResult::Value {value} -> (valid && value==expected) _ -> (!valid) }
}
'''
TAIL='''const RESULT:i32=scenario();
machine require_success(value:i32) requires value==0; {}
data Main{}
machine Main::main(&mut self){require_success(RESULT);}
'''
def main():
 rows=[json.loads(line) for line in (HERE/'reference.jsonl').read_text().splitlines()];assert len(rows)==240
 folder=HERE/'cases';folder.mkdir(exist_ok=True)
 for start in range(0,240,30):
  group=rows[start:start+30];profile=group[0]['profile'];assert all(r['profile']==profile for r in group)
  body=PREFIX+'machine scenario()->i32 {\nlet profile:State=memory_encryption::initial();\n'
  if profile!='disabled':
   for n,part in enumerate(profile.split('_')):
    case='EncryptedBit' if part[0]=='e' else 'SharedBit';body+=f'let configured{n}:bool=memory_encryption::configure(&mut profile,Configuration::{case} {{position:{part[1:]}}});\n'
  for n,row in enumerate(group):
   value=f'Translation::Mapped {{frame:{row["frame"]},size:{row["size"]},offset:{row["offset"]},flags:9223372036854775808}}' if row['input_kind']==0 else ('Translation::NotMapped' if row['input_kind']==1 else 'Translation::InvalidVirtual')
   body+=f'let observation{n}:Translation={value};\nlet projected{n}:NumberResult=encrypted_address_projection::translated_address_pinned(&profile,observation{n});\nlet pass{n}:bool=projection_matches(projected{n},{str(row["outcome"]==0).lower()},{row["value"]});\n'
  body+='transition '+' && '.join(f'pass{n}' for n in range(30))+' { true -> (0) _ -> (1) } }\n'+TAIL
  path=folder/f'batch-{start//30:02}.omg'
  if '--check' in sys.argv:assert path.read_text()==body,path
  else:path.write_text(body)
 print('240 actual Translate-default calls across8 profiles;8 Omega projection fixtures')
if __name__=='__main__':main()
