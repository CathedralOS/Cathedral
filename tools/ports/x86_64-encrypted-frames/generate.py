#!/usr/bin/env python3
"""Build actual Omega comparisons from pinned public numeric/cursor observations."""
import argparse,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
IMPORTS='''use x86::encrypted_frames;
use x86::encrypted_frames::IteratorResult;
use x86::memory_encryption::State;
use x86::memory_encryption::Configuration;
use x86::memory_encryption::initial;
use x86::memory_encryption::configure;
use x86::addresses::NumberResult;
use x86::addresses::OverflowingStep;
'''
HELPERS='''machine ef_fixture_number(result:NumberResult,accepted:bool,expected:u64)->bool {
 transition result {NumberResult::Value {value} -> compare(value,accepted,expected) _ -> (!accepted)}
 state compare(value:u64,accepted:bool,expected:u64)->bool {accepted && value==expected}
}
machine ef_fixture_iterator(result:IteratorResult,accepted:bool,expected:u64,failed:bool,start:u64,end:u64)->bool {
 let selected:bool=ef_fixture_number(result.selection,accepted,expected);
 selected && result.failed==failed && result.start==start && result.end==end
}
'''
def boolean(v):return str(v).lower()
def cases():
 rows=[json.loads(line)for line in (HERE/'reference.jsonl').read_text().splitlines()]
 for i,row in enumerate(rows):row['case_id']=i;row['origin']='actual pinned public Rust'
 for size in [0,1,8192,(1<<64)-1]:
  for op in ['from_start','containing','from_pfn','pfn','arithmetic','difference','range_count','range_bytes','range_at','iterator']:
   rows.append(dict(case_id=len(rows),origin='added closed-size admission policy',profile='e47_s48_e47',op=op,size=size,start=0,end=0,index=0,inclusive=True,reverse=False,ok=False,value=0,failed=True,out_start=0,out_end=0))
 return rows

def groups():
 rows=cases();out=[]
 for profile in dict.fromkeys(row['profile']for row in rows):
  selected=[row for row in rows if row['profile']==profile]
  for at in range(0,len(selected),48):out.append(dict(name=f'{profile}_batch_{at//48}',profile=profile,cases=selected[at:at+48]))
 return out

def render(group,control=False,machine=None):
 name=machine or'test_result';head='machine '+name+('(&mut self)'if'::'in name else'()')+'->i32 {\nlet mut profile:State=initial();\n';checks=[]
 if group['profile']!='disabled':
  for i,part in enumerate(group['profile'].split('_')):
   kind='EncryptedBit'if part[0]=='e'else'SharedBit';head+=f'let configured{i}:bool=configure(&mut profile,Configuration::{kind} {{position:{part[1:]}}});\n';checks.append(f'configured{i}')
 for i,row in enumerate(group['cases']):
  op=row['op'];a=row['start'];b=row['end'];size=row['size'];n=row['index'];inc=boolean(row['inclusive']);rev=boolean(row['reverse']);ok=row['ok']
  if op in ['from_start','containing','from_pfn','pfn']:call=f'{op}(profile,{a},{size})'
  elif op=='arithmetic':call=f'arithmetic(profile,{a},{size},{n},{rev})'
  elif op=='difference':call=f'difference(profile,{b},{a},{size})'
  elif op in ['range_count','range_bytes']:call=f'{op}(profile,{a},{b},{size},{inc})'
  else:call=f'{"iterator_step"if op=="iterator"else"range_at"}(profile,{a},{b},{size},{inc},{n},{rev})'
  if op=='iterator':
   fail=row['failed']^(control and i==0);head+=f'let result{i}:IteratorResult=encrypted_frames::{call};\nlet check{i}:bool=ef_fixture_iterator(result{i},{boolean(ok)},{row["value"]},{boolean(fail)},{row["out_start"]},{row["out_end"]});\n'
  else:
   expected_ok=ok^(control and i==0);head+=f'let result{i}:NumberResult=encrypted_frames::{call};\nlet check{i}:bool=ef_fixture_number(result{i},{boolean(expected_ok)},{row["value"]});\n'
   if op=='arithmetic':
    head+=f'let overflow{i}:OverflowingStep=encrypted_frames::arithmetic_overflowing(profile,{a},{size},{n},{rev});\nlet check_overflow{i}:bool=overflow{i}.overflow=={boolean(not ok)} && overflow{i}.value=={row["value"]if ok else a};\n';checks.append(f'check_overflow{i}')
  checks.append(f'check{i}')
 return head+'transition '+' && '.join(checks)+' {true -> (0) _ -> (1)}\n}\n'

def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();rows=cases();batches=groups()
 text=IMPORTS+HELPERS+render(batches[0])+'''const TEST_RESULT:i32=test_result();
machine require_ok(value:i32) requires value==0;{}
data Main{}
machine Main::main(&mut self){require_ok(TEST_RESULT);}
'''
 for path,content in [(HERE/'vectors.json',json.dumps(rows,indent=2)+'\n'),(HERE/'main.omg',text)]:
  if a.check:assert path.read_text()==content,path
  else:path.write_text(content)
 print(len(rows),'numeric/cursor cases in',len(batches),'groups', 'verified'if a.check else'generated')
if __name__=='__main__':main()
