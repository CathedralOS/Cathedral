#!/usr/bin/env python3
from pathlib import Path
import json,sys
HERE=Path(__file__).resolve().parent;MASK=0x000ffffffffff000
rows=json.loads((HERE/'cases.json').read_text());observed=json.loads((HERE/'observations.json').read_text())
expressions=[]
for r,o in zip(rows,observed['routes']):
 leaf={1073741824:1,2097152:2,4096:3}[r['size']];target=4-r['level'] if r['op']=='parent' else leaf
 loc=2 if o['visited']==target+1 else 1 if r['op']=='map' else 0
 payload=o['payload']
 if o['code']==0 and r['op'] in ('update','parent'):payload=r['words'][target]&MASK
 ids=[16384]+[w&MASK for w in r['words'][:3]];final_ids=ids.copy()
 for j in range(1,4):
  if o['zero']&(1<<j):final_ids[j]=o['words'][j-1]&MASK
 result_word=o['words'][o['visited']-1] if loc else 0;result_write=bool(o['write']&(1<<(o['visited']-1))) if loc else False
 req={'map':f'Request::Map {{ frame:{r["frame"]}, flags:{r["flags"]}, parent_flags:{r["parent"]} }}','unmap':'Request::Unmap','translate':'Request::Translate','update':f'Request::Update {{ new_flags:{r["flags"]} }}','parent':f'Request::SetParent {{ level:{r["level"]}, table_flags:{r["flags"]} }}'}[r['op']]
 frames=', '.join('FrameSupply::Unavailable' if v is None else f'FrameSupply::Supplied {{ address:{v} }}' for v in r['frames'])
 expected=f'Expected {{ words:{o["words"]}, ids:{final_ids}, indices:{[(r["page"]>>shift)&511 for shift in (39,30,21,12)]}, writes:{o["write"]}, zeros:{o["zero"]}, calls:{o["calls"]}, visited:{o["visited"]}, code:{o["code"]}, location:{loc}, payload:{payload}, result_word:{result_word}, result_write:{str(result_write).lower()} }}'
 expressions.append(f'checks::route(CapturedPath {{ table_ids:{ids}, words:{r["words"]} }}, AllocationInputs {{ frames:[{frames}] }}, {r["page"]}, {r["size"]}, {req}, {expected})')
outputs={};batches=[]
for group,start in enumerate(range(0,len(expressions),6)):
 name=f'route_{group:02}.omg';part=expressions[start:start+6]
 text='''use checks;
use checks::Expected;
use x86::mapping_routes::CapturedPath;
use x86::mapping_routes::AllocationInputs;
use x86::mapping_routes::Request;
use x86::mapping_plans::FrameSupply;
machine test_result() -> i32 {
    transition '''+' &&\n        '.join(part)+''' { true -> (0) _ -> (1) }
}
const TEST_RESULT: i32 = test_result();
machine require_success(value: i32) requires value == 0; {}
data Main {}
machine Main::main(&mut self) { require_success(TEST_RESULT); }
'''
 outputs[HERE/name]=text;batches.append({'path':name,'start':start,'count':len(part)})
translations=json.loads((HERE/'translation-cases.json').read_text());translation_expressions=[];translation_batches=[]
for r,v in zip(translations,observed['translations']):
 translation_expressions.append('translation_checks::check('+', '.join(map(str,[r['va'],*r['words'],*v]))+')')
for group,start in enumerate(range(0,len(translation_expressions),30)):
 name=f'translate_{group:02}.omg';part=translation_expressions[start:start+30]
 outputs[HERE/name]='use translation_checks;\nmachine test_result() -> i32 {\n transition '+' &&\n'.join(part)+' { true -> (0) _ -> (1) }\n}\nconst TEST_RESULT: i32 = test_result();\nmachine require_success(value: i32) requires value == 0; {}\ndata Main {}\nmachine Main::main(&mut self) { require_success(TEST_RESULT); }\n'
 translation_batches.append({'path':name,'start':start,'count':len(part)})
controls=[]
for label,size,oldfield,newfield in [
 ('new-parent-flags-0',1073741824,'words:[4099,','words:[4097,'),
 ('retain-write-before-huge-failure',1073741824,'writes:1','writes:0'),
 ('existing-no-present-parent-flags-zero',4096,'code:0','code:1'),
 ('parent-nonpresent-huge-2-0',4096,'code:0','code:2'),
 ('new-tables-0',4096,'calls:1','calls:0')]:
 n=next(j for j,r in enumerate(rows) if r['label']==label and r['size']==size)
 old=expressions[n];new=old.replace(oldfield,newfield);assert new!=old,(label,oldfield,old)
 controls.append({'path':f'route_{n//6:02}.omg','old':old,'new':new,'family':label})
for n,family in [(25,'nonpresent-translation'),(5,'root-huge-order'),(23,'leaf-huge-outcome')]:
 old=translation_expressions[n];r=translations[n];v=list(observed['translations'][n]);v[0]=1 if v[0]!=1 else 0
 new='translation_checks::check('+', '.join(map(str,[r['va'],*r['words'],*v]))+')';assert old!=new
 controls.append({'path':f'translate_{n//30:02}.omg','old':old,'new':new,'family':family})
outputs[HERE/'fixtures.json']=json.dumps({'routes':batches,'translations':translation_batches,'case_count':len(rows),'translation_count':len(translations),'controls':controls},indent=2)+'\n'
for p,text in outputs.items():
 if '--check' in sys.argv:
  if not p.exists() or p.read_text()!=text:raise SystemExit('fixture drift '+str(p))
 else:p.write_text(text)
print('Generated',len(rows),'route checks in',len(batches),'batches')
