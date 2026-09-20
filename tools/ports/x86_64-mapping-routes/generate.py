#!/usr/bin/env python3
"""Original complete-route scenarios, checked against pinned Rust execution."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
MASK=0x000ffffffffff000;MAX=2**64-1;PAGE=0xffff920340000000
rows=[]
def add(size,op,words=None,frames=None,flags=3,parent=3,level=4,label=''):
 leaf={2**30:1,2**21:2,4096:3}[size]
 if words is None:
  words=[4097,8193,12289,0]
  words[leaf]=0 if op=='map' else 0x80000001|(128 if size!=4096 else 0)
 rows.append(dict(size=size,op=op,words=words.copy(),frames=frames or [0,0,0],flags=flags,parent=parent,level=level,page=PAGE,frame=0xc0000000,label=label))
for size in [2**30,2**21,4096]:
 leaf={2**30:1,2**21:2,4096:3}[size]
 for op in ['map','unmap','update','translate']:
  add(size,op,label='ordinary')
  for depth in range(leaf):
   for invalid in [0,128,4096*(depth+1),4096*(depth+1)|129]:
    words=[4097,8193,12289,0];words[leaf]=0 if op=='map' else 0x80000081
    words[depth]=invalid
    add(size,op,words,label=f'parent-{depth}-{invalid}')
  for word in [0,512,0x80001001,0x80001081]:
   words=[4097,8193,12289,0];words[leaf]=word
   add(size,op,words,label=f'leaf-{word}')
 for available in range(leaf+1):
  frames=[4096*(i+1) if i<available else 0 for i in range(3)]
  add(size,'map',[0,8193,12289,512],frames,label=f'new-tables-{available}')
 for parent in [0,128,129,7]:
  add(size,'map',[0,8193,12289,512],[4096,8192,12288],parent=parent,label=f'new-parent-flags-{parent}')
 for depth in range(1,leaf):
  for available in range(leaf-depth+1):
   words=[4097,8193,12289,512];words[depth]=0
   frames=[4096*(depth+i+1) if i<available else 0 for i in range(3)]
   add(size,'map',words,frames,label=f'new-suffix-{depth}-{available}')
 add(size,'map',flags=128,label='leaf-huge-flag')
 add(size,'update',flags=MAX,label='all-leaf-flags')
 for level in [2,3,4]:
  add(size,'parent',level=level,flags=0,label=f'parent-clear-{level}')
  words=[4097,8193,12289,0];words[4-level]=0
  add(size,'parent',words,level=level,label=f'parent-unused-{level}')

def model(r):
 words=r['words'].copy();write=zero=calls=visited=0;kind='Ready';where='leaf';payload=0
 leaf={2**30:1,2**21:2,4096:3}[r['size']];target=4-r['level'] if r['op']=='parent' else leaf
 if r['op']=='parent' and target>=leaf:
  return dict(words=words,write=write,zero=zero,calls=calls,visited=visited,kind='ParentHugePage',where='parent',payload=0)
 for index in range(target):
  visited=index+1;word=words[index]
  if r['op']=='map':
   where='child';created=False
   if word==0:
    supplied=r['frames'][calls];calls+=1
    if not supplied:kind='AllocationFailed';break
    if r['parent']&128:kind='InvalidHugeInput';break
    word=supplied|r['parent'];words[index]=word;write|=1<<index;created=True
   elif r['parent'] and (word&~MASK)&r['parent']!=r['parent']:
    word|=r['parent'];words[index]=word;write|=1<<index
   if not word&1:kind='MissingPresent';break
   if word&128:kind='HugeParent';break
   if created:words[index+1]=0;zero|=1<<(index+1)
  else:
   where='parent'
   if not word&1:kind='ParentNotMapped';break
   if word&128:kind='ParentHugePage';break
 else:
  where='leaf';visited=target+1;word=words[target];payload=word&MASK
  if r['op']=='map':
   payload=r['frame']
   if word:kind='AlreadyMapped'
   elif r['size']==4096 and r['flags']&128:kind='InvalidHugeInput'
   else:words[target]=r['frame']|r['flags']|(128 if r['size']!=4096 else 0);write|=1<<target
  elif r['op']=='unmap':
   if not word&1:kind='PageNotMapped'
   elif bool(word&128)!=(r['size']!=4096):kind='ParentHugePage'
   elif payload%r['size']:kind='InvalidFrameAddress'
   else:words[target]=0;write|=1<<target
  elif r['op']=='translate':
   if not word:kind='PageNotMapped'
   elif payload%r['size']:kind='InvalidFrameAddress'
  else:
   if not word:kind='PageNotMapped'
   else:words[target]=payload|r['flags']|(128 if r['size']!=4096 and r['op']=='update' else 0);write|=1<<target
 return dict(words=words,write=write,zero=zero,calls=calls,visited=visited,kind=kind,where=where,payload=payload)

def write(path,text):
 if '--check' in sys.argv:assert path.read_text()==text,('stale',path)
 else:path.write_text(text)
def arr(values):return '['+','.join(map(str,values))+']'
prefix='// SPDX-License-Identifier: MIT OR Apache-2.0\nuse x86::mapping_routes;\nuse x86::mapping_routes::CapturedPath;\nuse x86::mapping_routes::AllocationInputs;\nuse x86::mapping_routes::Request;\nuse x86::mapping_routes::RoutePlan;\nuse x86::mapping_routes::Outcome;\nuse x86::mapping_plans::FrameSupply;\nuse x86::mapping_plans::ChildPlan;\nuse x86::mapping_plans::LeafPlan;\n'
blocks=[];rust=[]
codes={'Ready':0,'ParentNotMapped':1,'PageNotMapped':1,'ParentHugePage':2,'HugeParent':2,'InvalidFrameAddress':3,'AlreadyMapped':4,'InvalidHugeInput':5,'AllocationFailed':6,'MissingPresent':7}
for i,r in enumerate(rows):
 e=model(r);r['expected']=e
 req={'map':f'Request::Map {{ frame:{r["frame"]},flags:{r["flags"]},parent_flags:{r["parent"]} }}','unmap':'Request::Unmap','translate':'Request::Translate','update':f'Request::Update {{ new_flags:{r["flags"]} }}','parent':f'Request::SetParent {{ level:{r["level"]},table_flags:{r["flags"]} }}'}[r['op']]
 supplies=['FrameSupply::Unavailable' if n==0 else f'FrameSupply::Supplied {{ address:{n} }}' for n in r['frames']]
 body=f'let path:CapturedPath=CapturedPath {{ table_ids:[16384,4096,8192,12288],words:{arr(r["words"])} }}; let allocations:AllocationInputs=AllocationInputs {{ frames:[{",".join(supplies)}] }}; let value:RoutePlan=mapping_routes::plan(&path,&allocations,{r["page"]},{r["size"]},{req});\n'
 comparisons=[f'value.edits.words[{j}] == {n}' for j,n in enumerate(e['words'])]
 comparisons += [f'value.edits.{name} == {e[key]}' for name,key in [('write_mask','write'),('zero_mask','zero'),('allocation_calls','calls'),('visited','visited')]]
 comparisons += [f'value.edits.indices[{j}] == {(r["page"]>>shift)&511}' for j,shift in enumerate([39,30,21,12])]
 comparisons += [f'value.edits.table_ids[{j}] == {n}' for j,n in enumerate([16384,4096,8192,12288])]
 body+='let common:bool='+' && '.join(comparisons)+';\n'
 if e['where']=='parent':body+=f'transition common && value.outcome in Outcome::{e["kind"]} {{ true -> (0) _ -> (1) }}'
 else:
  typ='ChildPlan' if e['where']=='child' else 'LeafPlan';field='child' if e['where']=='child' else 'leaf';case='ChildFailure' if e['where']=='child' else 'Leaf'
  body+=f'transition value.outcome {{ Outcome::{case} {{ {field} }} -> inspect(common,{field}) _ -> (1) }} state inspect(common:bool,result:{typ})->i32 {{ '
  if e['kind'] in ['Ready','InvalidFrameAddress','AlreadyMapped']:
   payload={'Ready':'frame','InvalidFrameAddress':'address','AlreadyMapped':'requested_frame'}[e['kind']]
   body+=f'transition result {{ {typ}::{e["kind"]} {{ {payload} }} -> payload(common,{payload}) _ -> (1) }} }} state payload(common:bool,number:u64)->i32 {{ transition common && number == {e["payload"]} {{ true -> (0) _ -> (1) }} }}'
  else:body+=f'transition common && result in {typ}::{e["kind"]} {{ true -> (0) _ -> (1) }} }}'
 blocks.append(f'machine route_{i}()->i32 {{ {body} }}')
 typ={4096:'Size4KiB',2**21:'Size2MiB',2**30:'Size1GiB'}[r['size']]
 payload=e['payload'] if e['where']=='leaf' and e['kind'] in ['Ready','InvalidFrameAddress','AlreadyMapped'] else 0
 rust.append(f'assert_eq!(observe::<{typ}>("{r["op"]}",{r["level"]},{r["page"]},{arr(r["words"])},{arr(r["frames"])},{r["frame"]},{r["flags"]},{r["parent"]}),Observation{{kind:{codes[e["kind"]]},payload:{payload},words:{arr(e["words"])},calls:{e["calls"]},zero:{e["zero"]}}},"case {i}: {r["label"]}");')
write(HERE/'cases.json',json.dumps(rows,indent=2)+'\n')
write(HERE/'main.omg',prefix+'\n'.join(blocks)+'\ndata Main{}\nmachine Main::main(&mut self){}\n')
write(HERE/'src/main.rs',(HERE/'probe.rs.in').read_text().replace('// GENERATED_CHECKS','\n'.join(rust)))
print(len(rows),'complete-route cases')
if __name__=='__main__':pass
