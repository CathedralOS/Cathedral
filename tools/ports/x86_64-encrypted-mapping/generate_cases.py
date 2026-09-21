#!/usr/bin/env python3
from pathlib import Path
import json,sys
HERE=Path(__file__).resolve().parent;MASK=0x000ffffffffff000;PAGE=0xffff920340000000;IDS=[65536,131072,196608];SIZES=[1<<30,1<<21,4096]
profiles=[{'name':'disabled','configs':[]}]+[{'name':f'{kind}{bit}','configs':[[bit,kind=='shared']]} for kind,bit in [('encrypted',0),('encrypted',7),('encrypted',12),('encrypted',21),('encrypted',30),('encrypted',47),('shared',47),('encrypted',51),('shared',63)]]+[{'name':'repeated47-48','configs':[[47,False],[48,True]]}]
rows=[]
for profile,p in enumerate(profiles):
 mask=MASK;marker=0;current=0
 for bit,_ in p['configs']:marker|=1<<bit;current=1<<bit;mask&=~current
 p.update(address_mask=mask,bit_mask=current,marker=marker)
 def add(mode,size,op,label,**kw):
  leaf=SIZES.index(size)+1;words=[(x|1|(marker&MASK)) for x in IDS]+[0]
  words[leaf]=(0xc0000000|marker|1|(128 if size!=4096 else 0)) if op!='map' else 0
  r=dict(profile=profile,recursive=mode,size=size,op=op,words=words,frames=[None]*3,flags=3,parent=3|marker,level=4,page=PAGE,frame=0xc0000000&~current,label=label)
  r.update(kw);rows.append(r)
 for mode in [False,True]:
  for size in SIZES:
   add(mode,size,'update','profile-leaf-update')
   add(mode,size,'translate','profile-leaf-translate')
  add(mode,4096,'unmap','profile-leaf-unmap')
  add(mode,4096,'map','new-full-route',words=[0,512,512,512],frames=IDS)
  words=[x|3|(marker&MASK) for x in IDS]+[0]
  add(mode,4096,'map','existing-flag-already-present',words=words,parent=3|(marker&MASK))
  if p['name'] in ['encrypted12','encrypted21','encrypted47','repeated47-48']:
   for size in [1<<30,1<<21]:add(mode,size,'unmap','huge-frame-mask-alignment')
  if p['name'] in ['encrypted47','repeated47-48']:
   for size in SIZES:
    for level in [4,3,2]:add(mode,size,'parent','profile-parent-update',level=level,flags=3)
   for depth in range(3):
    for bad in [0,IDS[depth]|marker,IDS[depth]|marker|128]:
     words=[x|1|(marker&MASK) for x in IDS]+[0xc0000001|marker];words[depth]=bad
     add(mode,4096,'update','ancestor-order',words=words)
   for count in [0,1,2]:add(mode,4096,'map','allocation-exhaustion',words=[0,512,512,512],frames=IDS[:count]+[None]*(3-count))
   add(mode,4096,'map','new-zero-parent-flags',words=[0,512,512,512],frames=IDS,parent=0)
   add(mode,4096,'map','new-huge-parent-flags',words=[0,512,512,512],frames=IDS,parent=128|marker)
   words=[IDS[0]|128|marker,IDS[1]|1|marker,IDS[2]|1|marker,0]
   add(mode,4096,'map','retained-write-before-huge',words=words,parent=2|marker)
   add(mode,4096,'update','raw-flag-address-contamination',flags=0xffffffffffffffff)
   if p['name']=='repeated47-48':
    add(mode,4096,'map','old-bit-allocated-frame',words=[0,512,512,512],frames=[x|(1<<47) for x in IDS],parent=3)
    add(mode,4096,'map','old-bit-leaf-frame',frame=(1<<47)|0xc0000000,parent=3)
outputs={HERE/'profiles.json':profiles,HERE/'cases.json':rows}
for path,obj in outputs.items():
 s=json.dumps(obj,indent=2)+'\n'
 if '--check' in sys.argv:
  if not path.exists() or path.read_text()!=s:raise SystemExit('case drift '+str(path))
 else:path.write_text(s)
print('PASS',len(profiles),'profiles;',len(rows),'route scenarios')
