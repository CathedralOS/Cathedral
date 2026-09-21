#!/usr/bin/env python3
from pathlib import Path
import json,sys
HERE=Path(__file__).resolve().parent;rows=json.loads((HERE/'cases.json').read_text());observed=json.loads((HERE/'observations.json').read_text());profiles=json.loads((HERE/'profiles.json').read_text())
expressions=[]
for r,o in zip(rows,observed):
 mask=profiles[r['profile']]['address_mask'];leaf={1073741824:1,2097152:2,4096:3}[r['size']];target=4-r['level'] if r['op']=='parent' else leaf
 loc=2 if o['visited']==target+1 else 1 if r['op']=='map' else 0;payload=o['payload']
 if o['code']==0 and r['op'] in ('update','parent'):payload=r['words'][target]&mask
 ids=[262144]+[w&mask for w in r['words'][:3]];final_ids=ids.copy()
 for j in range(1,4):
  if o['zero']&(1<<j):final_ids[j]=o['words'][j-1]&mask
 result_word=o['words'][o['visited']-1] if loc else 0;result_write=bool(o['write']&(1<<(o['visited']-1))) if loc else False
 req={'map':f'Request::Map {{ frame:{r["frame"]}, flags:{r["flags"]}, parent_flags:{r["parent"]} }}','unmap':'Request::Unmap','translate':'Request::Translate','update':f'Request::Update {{ new_flags:{r["flags"]} }}','parent':f'Request::SetParent {{ level:{r["level"]}, table_flags:{r["flags"]} }}'}[r['op']]
 frames=', '.join('FrameSupply::Unavailable' if v is None else f'FrameSupply::Supplied {{ address:{v} }}' for v in r['frames'])
 expected=f'Expected {{ words:{o["words"]}, ids:{final_ids}, indices:{[(r["page"]>>shift)&511 for shift in (39,30,21,12)]}, writes:{o["write"]}, zeros:{o["zero"]}, calls:{o["calls"]}, visited:{o["visited"]}, code:{o["code"]}, location:{loc}, payload:{payload}, result_word:{result_word}, result_write:{str(result_write).lower()} }}'
 expressions.append(f'checks::{"recursive" if r["recursive"] else "mapped"}_route(profile, CapturedPath {{ table_ids:{ids}, words:{r["words"]} }}, AllocationInputs {{ frames:[{frames}] }}, {r["page"]}, {r["size"]}, {req}, {expected})')
outputs={};batches=[];case_file={}
for pn,p in enumerate(profiles):
 selected=[j for j,r in enumerate(rows) if r['profile']==pn]
 for group,start in enumerate(range(0,len(selected),6)):
  name=f'profile_{pn:02}_{group:02}.omg';indices=selected[start:start+6];part=[expressions[j] for j in indices]
  init='    let profile: State = memory_encryption::initial();\n'
  for n,(bit,reverse) in enumerate(p['configs']):init+=f'    let configured{n}: bool = memory_encryption::configure(&mut profile, Configuration::{"SharedBit" if reverse else "EncryptedBit"} {{ position:{bit} }});\n'
  s='''use checks;
use checks::Expected;
use x86::mapping_routes::CapturedPath;
use x86::mapping_routes::AllocationInputs;
use x86::mapping_routes::Request;
use x86::mapping_plans::FrameSupply;
use x86::memory_encryption;
use x86::memory_encryption::State;
use x86::memory_encryption::Configuration;
'''
  for n,expr in enumerate(part):s+=f'machine row{n}() -> bool {{\n'+init+f'    let result: bool = {expr};\n    result\n}}\n'
  s+='machine test_result() -> i32 {\n'+''.join(f'    let check{n}: bool = row{n}();\n' for n in range(len(part)))+'    transition '+' && '.join(f'check{n}' for n in range(len(part)))+''' { true -> (0) _ -> (1) }
}
const TEST_RESULT: i32 = test_result();
machine require_success(value: i32) requires value == 0; {}
data Main {}
machine Main::main(&mut self) { require_success(TEST_RESULT); }
'''
  outputs[HERE/name]=s;batches.append({'path':name,'profile':pn,'indices':indices})
  for j in indices:case_file[j]=name
controls=[]
choices=[('encrypted47','profile-leaf-update',False,'result_word','profile-update-mask'),('encrypted47','existing-flag-already-present',False,'writes','existing-encryption-flag-no-write'),('encrypted47','new-zero-parent-flags',True,'code','recursive-forced-flags'),('encrypted47','retained-write-before-huge',False,'writes','retained-write-before-failure'),('repeated47-48','old-bit-allocated-frame',False,'ids','accumulated-mask-allocation'),('repeated47-48','old-bit-leaf-frame',True,'payload','latest-bit-admission'),('shared47','profile-leaf-translate',True,'payload','shared-profile-frame'),('encrypted12','huge-frame-mask-alignment',False,'code','masked-huge-alignment')]
for profile,label,recursive,field,family in choices:
 n=next(j for j,r in enumerate(rows) if profiles[r['profile']]['name']==profile and r['label']==label and r['recursive']==recursive)
 old=expressions[n];o=observed[n]
 if field=='ids':
  # Change the first newly cleared child identity inside Expected, not the input.
  before,expected=old.split('Expected {');expected=expected.replace('ids:[262144, 65536,','ids:[262144, 65537,');new=before+'Expected {'+expected
 else:
  import re
  before,expected=old.split('Expected {');m=re.search(r'\b'+field+r':(\d+)',expected);assert m;value=int(m[1]);expected=expected[:m.start(1)]+str(value^1)+expected[m.end(1):];new=before+'Expected {'+expected
 assert old!=new,(family,old)
 controls.append({'path':case_file[n],'old':old,'new':new,'family':family})
outputs[HERE/'fixtures.json']=json.dumps({'case_count':len(rows),'batches':batches,'controls':controls},indent=2)+'\n'
for p,s in outputs.items():
 if '--check' in sys.argv:
  if not p.exists() or p.read_text()!=s:raise SystemExit('fixture drift '+str(p))
 else:p.write_text(s)
print('PASS generated',len(batches),'Omega batches and',len(controls),'mutations')
