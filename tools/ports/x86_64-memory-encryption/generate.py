#!/usr/bin/env python3
"""Derive Omega comparisons from isolated actual upstream API observations."""
import argparse,json,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
PREFIX='''// SPDX-License-Identifier: MIT OR Apache-2.0
use {x86}/memory_encryption.omg;
use {x86}/addresses.omg;
use {x86}/page_entries.omg;
machine number(result: |addresses|::NumberResult, expected: u64) -> bool {
 transition result { |addresses|::NumberResult::Value { value } -> (value == expected) _ -> (false) }
}
machine frame(result: |page_entries|::FrameResult, kind: u8, expected: u64) -> bool {
 transition result {
  |page_entries|::FrameResult::NotPresent -> (kind == 0)
  |page_entries|::FrameResult::HugeFrame -> (kind == 1)
  |page_entries|::FrameResult::Value { address } -> (kind == 2 && address == expected)
 }
}
'''
def fixture(row):
 lines=[PREFIX,'machine scenario() -> i32 {',' let mut profile: |memory_encryption|::State = |memory_encryption|::initial();'];checks=[]
 if row['profile']!='disabled':
  for n,part in enumerate(row['profile'].split('_')):
   variant='EncryptedBit' if part[0]=='e' else 'SharedBit';position=int(part[1:])
   lines += [f' let configured{n}: bool = |memory_encryption|::configure(&mut profile, |memory_encryption|::Configuration::{variant} {{ position: {position} }});'];checks += [f'configured{n}']
 for n,obs in enumerate(row['observations']):
  word=obs['word']
  exprs={'encrypted':('bool',f'is_encrypted(&profile, {word})'),'clear':('|addresses|::NumberResult',f'set_encrypted(&profile, {word}, false)'),'set':('|addresses|::NumberResult',f'set_encrypted(&profile, {word}, true)'),'address':('u64',f'entry_address(&profile, {word})'),'flags':('u64',f'entry_flags(&profile, {word})'),'replaced':('u64',f'set_flags(&profile, {word}, 17293822569102704647)'),'physical':('|addresses|::NumberResult',f'physical_check(&profile, {word})'),'truncated':('|addresses|::NumberResult',f'physical_truncate(&profile, {word})'),'set_addr':('|addresses|::NumberResult',f'set_address(&profile, {word}, 9223372036854775811)'),'set_frame':('|addresses|::NumberResult',f'set_frame(&profile, {word}, 9223372036854775811)')}
  for key,(typ,call) in exprs.items():
   name=f'{key}{n}';lines += [f' let {name}: {typ} = |memory_encryption|::{call};'];expected=obs[key]
   if typ=='|addresses|::NumberResult':
    if expected is None:checks += [f'{name} in |addresses|::NumberResult::Rejected']
    else:lines += [f' let ok_{name}: bool = number({name}, {expected});'];checks += [f'ok_{name}']
   else:checks += [f'{name} == {str(expected).lower()}']
  lines += [f' let frame{n}: |page_entries|::FrameResult = |memory_encryption|::entry_frame(&profile, {word});',f" let ok_frame{n}: bool = frame(frame{n}, {obs['frame_kind']}, {obs['frame']});"];checks += [f'ok_frame{n}']
 lines += [' transition '+' && '.join(checks)+' { true -> (0) _ -> (1) }','}','const RESULT: i32 = scenario();','machine require_success(value: i32) requires value == 0; {}','data Main {}','machine Main::main(&mut self) { require_success(RESULT); }']
 return '\n'.join(lines)+'\n'
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');args=p.parse_args()
 r=subprocess.run(['cargo','run','--quiet','--locked','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT,capture_output=True,text=True,check=True);print(r.stderr,end='')
 rows=[json.loads(line) for line in r.stdout.splitlines()];assert len(rows)==133
 outputs={HERE/'cases.json':json.dumps(rows,indent=2)+'\n'}
 outputs.update({HERE/'cases'/f"{row['profile']}.omg":fixture(row) for row in rows})
 for path,text in outputs.items():
  if args.check:assert path.read_text()==text,f'stale {path}'
  else:path.parent.mkdir(exist_ok=True);path.write_text(text)
 print(f"PASS {len(rows)} profiles / {sum(len(row['observations']) for row in rows)} actual Rust observations and fixtures")
if __name__=='__main__':main()
