#!/usr/bin/env python3
"""Generate Omega witnesses from actual pinned public GDT observations."""
import argparse,json,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def observed():
 p=subprocess.run(['cargo','run','--quiet','--locked','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT,check=True,text=True,capture_output=True)
 print(p.stderr,end='')
 rows=[json.loads(line) for line in p.stdout.splitlines()]
 for row in rows:
  for obj,key in [(row,'initial'),(row,'words')]+[(action,'words') for action in row['actions']]:
   values=obj[key];obj[key]={'length':len(values),'nonzero':{str(i):v for i,v in enumerate(values) if v}}
 return rows
PREFIX='''// SPDX-License-Identifier: MIT OR Apache-2.0
use x86::gdt_storage;
use x86::gdt_storage::Table;
use x86::gdt_storage::AppendResult;
use x86::addresses::NumberResult;
use facts::x86_descriptors::Descriptor;
machine added(value: AppendResult, expected: u16) -> bool {
 transition value { AppendResult::Added { selector } -> (selector.raw == expected) _ -> (false) }
}
machine number(value: NumberResult, expected: u64) -> bool {
 transition value { NumberResult::Value { value } -> (value == expected) _ -> (false) }
}
'''
def fixture(row):
 lines=[PREFIX,'machine scenario() -> i32 {',' let mut table: Table;',' table.words[8191] = 16045690984833335023;']
 initial=row['initial'];cap=row['capacity']
 if initial['length']:
  lines+=[' let mut input: [u64; 8192];']
  lines += [f' input[{i}] = {v};' for i,v in initial['nonzero'].items()]
  lines+=[f" let initialized: bool = gdt_storage::import_words(&mut table, &input, {initial['length']}, {cap});"]
 else:lines+=[f' let initialized: bool = gdt_storage::initialize(&mut table, {cap});']
 checks=['initialized',f'table.capacity == {cap}']
 for n,action in enumerate(row['actions']):
  low,hi=action['low'],action['high'];desc=f'Descriptor::UserSegment {{ low: {low} }}' if hi is None else f'Descriptor::SystemSegment {{ low: {low}, high: {hi} }}'
  lines += [f' let result{n}: AppendResult = gdt_storage::append(&mut table, {desc});']
  if action['selector']<0:lines += [f' let pass{n}: bool = result{n} in AppendResult::Full;']
  else:lines += [f" let pass{n}: bool = added(result{n}, {action['selector']});"]
  words=action['words'];length=words['length'];indices=set(range(min(length,9)))|{0,8191,length-1}
  if length<8192:indices.add(length)
  indices |= {int(i) for i in words['nonzero']}
  match=' && '.join(f"table.words[{i}] == {words['nonzero'].get(str(i),0)}" for i in sorted(indices))
  lines += [f' let content{n}: bool = table.length == {length} && {match};',f' let limit{n}: NumberResult = gdt_storage::limit(&table);',f" let bound{n}: bool = number(limit{n}, {action['limit']});"]
  checks += [f'pass{n}',f'content{n}',f'bound{n}']
 lines += [f" let last: NumberResult = gdt_storage::entry(&table, {row['length']-1});",f" let projection: bool = number(last, {row['words']['nonzero'].get(str(row['length']-1),0)});",f" let outside: NumberResult = gdt_storage::entry(&table, {row['length']});"]
 checks+=['projection','outside in NumberResult::Rejected']
 lines+=[' transition '+' && '.join(checks)+' { true -> (0) _ -> (1) }','}','const RESULT: i32 = scenario();','machine require_success(value: i32) requires value == 0; {}','data Main {}','machine Main::main(&mut self) { require_success(RESULT); }']
 return '\n'.join(lines)+'\n'
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');args=p.parse_args();rows=observed();outputs={HERE/'cases.json':json.dumps(rows,indent=2)+'\n'}
 outputs.update({HERE/'cases'/f"{row['name']}.omg":fixture(row) for row in rows})
 for path,text in outputs.items():
  if args.check:assert path.read_text()==text,f'stale {path}'
  else:path.parent.mkdir(exist_ok=True);path.write_text(text)
 print('PASS actual Rust observations and generated GDT fixtures')
if __name__=='__main__':main()
