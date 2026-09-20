#!/usr/bin/env python3
"""Render bounded real Omega body checks from separately measured pinned fragments."""
from pathlib import Path
import json,sys
HERE=Path(__file__).resolve().parent
cases=json.loads((HERE/'cases.json').read_text());observations=json.loads((HERE/'observations.json').read_text());assert len(cases)==len(observations)
expressions=[]
for c,v in zip(cases,observations):
 op=c['op'];a=list(c['args'])
 if op=='merge':args=['RegisterKind::'+['Cr0','Cr4','Efer','Rflags','Xcr0'][a[0]],*a[1:],v[1]]
 elif op=='cr3_observed':args=[*a,*v[:4]]
 elif op in ('cr3_raw','cr3_pcid'):args=[*a[:2],str(bool(a[2])).lower(),*v[:2]]
 elif op=='star':args=[*a,*v]
 elif op in ('cet_observed','apic_observed'):args=[*a,*v[:3]]
 else:args=[*a,*v[:2]]
 expressions.append('checks::'+op+'('+', '.join(str(x) for x in args)+')')
outputs={};manifest=[]
for group,start in enumerate(range(0,len(expressions),96)):
 part=expressions[start:start+96];name=f'batch_{group:02}.omg'
 source='use checks;\nuse x86_values::register_operands::RegisterKind;\nmachine test_result() -> i32 {\n    transition '+ ' &&\n        '.join(part)+' { true -> (0) _ -> (1) }\n}\nconst TEST_RESULT: i32 = test_result();\nmachine require_success(value: i32) requires value == 0; {}\ndata Main {}\nmachine Main::main(&mut self) { require_success(TEST_RESULT); }\n'
 outputs[HERE/name]=source;manifest.append({'path':name,'start':start,'count':len(part)})
controls=[]
for op,args in [('merge',[0,(1<<64)-1,1]),('xcr0',[(1<<64)-1,5]),('cr3_raw',[0,4096,0]),('cr8_observed',[257]),('star',[(65520<<48)|0xffffffff]),('cet_observed',[1<<47]),('apic_preserving',[4096,8192,0])]:
 indices=[j for j,c in enumerate(cases) if c=={'op':op,'args':args}];assert len(indices)==1,(op,args,indices)
 j=indices[0];old=expressions[j];head,tail=old.rsplit(', ',1)
 if op in ('star','cet_observed'):
  # Mutate the expected failure tag rather than a discarded payload.
  v=list(observations[j]);v[0]=0
  new='checks::'+op+'('+', '.join(str(x) for x in [*args,*(v if op=='star' else v[:3])])+')'
 elif op=='xcr0':new=old.replace(', 2, 0)',', 1, 0)')
 else:new=head+', '+str(int(tail[:-1])^1)+')'
 assert new!=old;controls.append({'path':f'batch_{j//96:02}.omg','old':old,'new':new,'family':op})
outputs[HERE/'fixtures.json']=json.dumps({'cases':len(cases),'batches':manifest,'controls':controls},indent=2)+'\n'
for p,s in outputs.items():
 if '--check' in sys.argv:
  if not p.exists() or p.read_text()!=s:raise SystemExit('fixture drift: '+str(p))
 else:p.write_text(s)
print('Generated',len(cases),'checks in',len(manifest),'bounded batches and',len(controls),'body controls')
