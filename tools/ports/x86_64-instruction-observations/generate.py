#!/usr/bin/env python3
"""Extract exact two CPUID result predicates; no CPUID or provider constructor runs."""
import importlib.util,json,re,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];checkout=ROOT/'reference_code/rust-osdev/x86_64'
spec=importlib.util.spec_from_file_location('i',ROOT/'tools/ports/inventory.py');i=importlib.util.module_from_spec(spec);spec.loader.exec_module(i)
files=['src/instructions/random.rs','src/instructions/smap.rs'];m=i.snapshot(checkout,'cc35c876d3badb57df54a66e22f7768a52be95f2',files,'https://github.com/rust-osdev/x86_64')
conditions=[]
for f in files:
 s=(checkout/f).read_text();condition=re.search(r'if (cpuid\.(?:ecx|ebx).*?) \{',s).group(1);conditions.append(condition)
assert conditions==['cpuid.ecx & (1 << 30) != 0','cpuid.ebx.get_bit(20)']
rust='// SPDX-License-Identifier: MIT OR Apache-2.0\nuse bit_field::BitField;\nstruct Observed {ecx:u32,ebx:u32}\n'
for name,condition in zip(('rdrand','smap'),conditions):rust+='fn '+name+'(cpuid:Observed)->bool { '+condition+' }\n'
rust+='fn main(){for bit in 0..32 {let x=1u32<<bit;assert_eq!(rdrand(Observed{ecx:x,ebx:0}),bit==30);assert_eq!(smap(Observed{ecx:0,ebx:x}),bit==20);}for x in [0,u32::MAX]{assert_eq!(rdrand(Observed{ecx:x,ebx:x}),x!=0);assert_eq!(smap(Observed{ecx:x,ebx:x}),x!=0);}println!("PASS 68 exact pinned predicate observations; no CPUID execution");}\n'
omega=['use x86_values::instruction_observations;','machine test_result() -> i32 {']
checks=[]
for method,bit in [('rdrand_bit',30),('smap_bit',20)]:
 for n in range(32):checks.append(f'instruction_observations::{method}({1<<n}) == {str(n==bit).lower()}')
 checks.extend([f'!instruction_observations::{method}(0)',f'instruction_observations::{method}(4294967295)'])
omega+=['transition '+' &&\n'.join(checks)+' { true -> (0) _ -> (1) }','}','const RESULT:i32=test_result();','machine require_success(value:i32) requires value==0; {}','data Main{}','machine Main::main(&mut self){require_success(RESULT);}']
helper='source/libraries/x86_64/instruction_observations.omg'
for path,file in m['files'].items():
 file.update(disposition='translated',reason='Only the detached CPUID register predicate is translated; live feature/probe/provider operations remain explicit boundaries.',targets=[{'path':helper,'anchor':'module instruction_observations;'}])
 for key,row in file['symbols'].items():
  if key in ('10:new','36:new'):
   anchor='pub machine rdrand_bit' if path.endswith('random.rs') else 'pub machine smap_bit'
   row.update(disposition='translated',reason='Exact pure predicate extracted; CPUID execution and feature-authorized marker construction deliberately excluded.',targets=[{'path':helper,'anchor':anchor}])
  else:row.update(disposition='omitted',reason='RNG output, SMAP state/lifecycle, provider marker or Rust scaffolding outside detached CPUID predicate slice. No compiler blocker claimed.')
outputs={HERE/'src/main.rs':rust,HERE/'main.omg':'\n'.join(omega)+'\n',ROOT/'source/libraries/x86_64/instruction-observations-inventory.json':json.dumps(m,indent=2)+'\n'}
for p,text in outputs.items():
 if '--check' in sys.argv:
  if not p.exists() or p.read_text()!=text:raise SystemExit('generated predicate artifact drift: '+str(p))
 else:p.write_text(text)
print(i.check(m,checkout))
