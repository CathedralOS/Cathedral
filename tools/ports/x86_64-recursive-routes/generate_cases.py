#!/usr/bin/env python3
"""Original recursive route and translation scenarios; results come only from Rust mirrors."""
from pathlib import Path
import json,sys
HERE=Path(__file__).resolve().parent;MASK=0x000ffffffffff000;MAX=2**64-1;PAGE=0xffff920340000000
routes=[]
def add(size,op,words=None,frames=None,flags=3,parent=3,level=4,label=''):
 leaf={2**30:1,2**21:2,4096:3}[size]
 if words is None:
  words=[4097,8193,12289,0];words[leaf]=0 if op=='map' else 0x80000001|(128 if size!=4096 else 0)
 routes.append(dict(size=size,op=op,words=words.copy(),frames=frames or [None,None,None],flags=flags,parent=parent,level=level,page=PAGE,frame=0xc0000000,label=label))
for size in [2**30,2**21,4096]:
 leaf={2**30:1,2**21:2,4096:3}[size]
 for op in ['map','unmap','update','translate']:
  add(size,op,label='ordinary')
  for depth in range(leaf):
   for invalid in [0,128,4096*(depth+1),4096*(depth+1)|129]:
    words=[4097,8193,12289,0];words[leaf]=0 if op=='map' else 0x80000081;words[depth]=invalid
    add(size,op,words,label=f'parent-{depth}-{invalid}')
  for word in [0,512,0x80001001,0x80001081]:
   words=[4097,8193,12289,0];words[leaf]=word;add(size,op,words,label=f'leaf-{word}')
 for available in range(leaf+1):
  frames=[4096*(i+1) if i<available else None for i in range(3)]
  add(size,'map',[0,8193,12289,512],frames,label=f'new-tables-{available}')
 for parent in [0,2,128,129,7,MAX,4096]:
  add(size,'map',[0,8193,12289,512],[4096,8192,12288],parent=parent,label=f'new-parent-flags-{parent}')
 for depth in range(1,leaf):
  for available in range(leaf-depth+1):
   words=[4097,8193,12289,512];words[depth]=0
   frames=[4096*(depth+i+1) if i<available else None for i in range(3)]
   add(size,'map',words,frames,label=f'new-suffix-{depth}-{available}')
 add(size,'map',flags=128,label='leaf-huge-flag');add(size,'update',flags=MAX,label='all-leaf-flags')
 for level in [2,3,4]:
  add(size,'parent',level=level,flags=0,label=f'parent-clear-{level}')
  words=[4097,8193,12289,0];words[4-level]=0;add(size,'parent',words,level=level,label=f'parent-unused-{level}')
  for depth in range(max(0,4-level)):
   words=[4097,8193,12289,0];words[depth]=4096*(depth+1)|128
   add(size,'parent',words,level=level,flags=MAX,label=f'parent-nonpresent-huge-{level}-{depth}')
 # Existing non-present link cannot be silently upgraded with mandatory new-link flags.
 add(size,'map',[4096,8192,12288,0],parent=0,label='existing-no-present-parent-flags-zero')
 add(size,'map',[4096|128,8193,12289,0],parent=2,label='retain-write-before-huge-failure')
translations=[]
for va in [0,PAGE+0x12345,0xffffffffffffffff]:
 base=[4097,8193,12289,0x80000001]
 translations.append({'va':va,'words':base})
 for depth in range(4):
  for word in [0,512,4096*(depth+1),4096*(depth+1)|128,4096*(depth+1)|129,MAX]:
   words=base.copy();words[depth]=word;translations.append({'va':va,'words':words})
 for words in [[4096,8192,12288,0x80000000],[512,512,512,512],[0,128,128,128],[4097,0,128,128],[4097,8193,0,128]]:
  translations.append({'va':va,'words':words})
for name,value in [('cases.json',routes),('translation-cases.json',translations)]:
 p=HERE/name;text=json.dumps(value,indent=2)+'\n'
 if '--check' in sys.argv:
  if not p.exists() or p.read_text()!=text:raise SystemExit('case drift '+name)
 else:p.write_text(text)
print(len(routes),'route cases;',len(translations),'translation cases')
