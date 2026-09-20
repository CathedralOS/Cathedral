#!/usr/bin/env python3
"""Independent finite cursor oracle, checked against pinned whole-tree cleanup."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
MASK=0x000ffffffffff000;DENSE=2**48-1;MAXPAGE=2**64-4096;ROOT=16384
IDS=[4096,8192,12288,ROOT,20480,24576,28672]
def canonical(x):return x|0xffff000000000000 if x&(1<<47)else x
def indices(x):return [(x>>s)&511 for s in [39,30,21,12]]
def chain(page=0,second=False):
 i=indices(page);a,b,c=([20480,24576,28672]if second else[4096,8192,12288]);return [(ROOT,i[0],a|1),(a,i[1],b|1),(b,i[2],c|1)]
G=2**39;HI=0xffff800000000000
scenarios=[
 ('skip-zero',0,0,0,[],3),
 ('skip-interior',4096,G-8192,0,[],1),
 ('skip-low-to-high',255*G,HI,255,chain(HI),1),
 ('skip-high',HI,HI+G,256,chain(HI+G),1),
 ('skip-final',MAXPAGE,MAXPAGE,511,[],1),
 ('skip-final-span',canonical(511*G),MAXPAGE,511,[],4),
 ('two-ordinary-around-self',0,2*G,1,chain()+chain(2*G,True),1),
 ('self-occupancy',0,2*G,1,chain()+chain(2*G,True),4),
 ('range-ends-in-self',0,G+4096,1,chain(),4),
 ('leaf-neighbor',0,0,511,chain()+[(12288,511,1<<63)],4),
 ('leaf-nonpresent',0,8192,511,chain()+[(12288,0,512)],4),
 ('huge-parent',0,2**21,511,[(ROOT,0,4097),(4096,0,8193),(8192,0,129)],4),
 ('nonpresent-parent',0,2**30,511,[(ROOT,0,4097),(4096,0,8192)],4),
 ('high-ordinary',HI,HI+2**21,0,chain(HI),4),
 ('gap-ordinary',0x7ffffffff000,HI,0,chain(0x7ffffffff000)+chain(HI,True),1),
 ('last-ordinary',MAXPAGE,MAXPAGE,0,chain(MAXPAGE),4),
 ('reversed',4096,0,511,chain(),4),
 ('budget-zero',0,0,511,chain(),0),
 ('budget-max-skip',0,0,0,[],2**64-1),
 ('budget-max-ordinary',0,0,1,chain(),2**64-1),
]
rows=[];checks=[];records=[]
def arr(xs):return '['+','.join(map(str,xs))+']'
for name,first,last,r,seed,budget in scenarios:
 entries=seed+[(ROOT,r,ROOT|1)];assert sum(id==ROOT and i==r for id,i,w in entries)==1
 tree={id:{}for id in IDS}
 for id,i,w in entries:tree[id][i]=w
 page=first;retired=[];history=[];resumes=0;status='Done'if first>last else'Exhausted'if budget==0 else'Active'
 while status!='Done':
  if status=='Exhausted':budget=2;resumes+=1;status='Active'
  assert len(history)<2048
  slots=indices(page);ids=[ROOT,0,0,0];words=[0]*4;others=[False]*4;depth=0
  skip=slots[0]==r
  if skip:
   # Deliberately inconsistent captures: excluded slot must not inspect them.
   words=[129,2**64-1,17,33];ids=[0,1,2,3];others=[False]*4
   out=[0]*4;freed=[];mask=0;visited=0;span=G
  else:
   while True:
    table=tree[ids[depth]];words[depth]=table.get(slots[depth],0);others[depth]=any(w and i!=slots[depth]for i,w in table.items())
    if depth==3 or words[depth]&129!=1:break
    ids[depth+1]=words[depth]&MASK;depth+=1
   out=words.copy();freed=[];mask=0;d=depth;visited=depth+1
   while d>0 and out[d]==0 and not others[d]:
    freed.append(ids[d]);out[d-1]=0;mask|=1<<(d-1);tree[ids[d-1]][slots[d-1]]=0;d-=1
   assert others[0], 'self-link must remain counted outside every ordinary root slot'
   span=G if out[0]&129!=1 else 2**30 if out[1]&129!=1 else 2**21
  retired+=freed;end=((page&DENSE)|(span-1))&~4095;remaining=budget-1
  done=(last&DENSE)<=end;nextpage=page if done else canonical(end+4096);nextstatus='Done'if done else'Exhausted'if remaining==0 else'Active'
  row=dict(scenario=name,page=page,last=last,index=r,budget=budget,ids=ids,words=words,others=others,has_plan=not skip,out=out,retired=freed,write=mask,visited=visited,next=nextpage,remaining=remaining,status=nextstatus)
  rows.append(row);history.append(len(rows)-1);page=nextpage;budget=remaining;status=nextstatus
 final=sorted((id,i,w)for id,t in tree.items()for i,w in t.items()if w)
 rustentries='&['+','.join(f'({a},{b},{c})'for a,b,c in entries)+']';rustfinal='vec!['+','.join(f'({a},{b},{c})'for a,b,c in final)+']'
 checks.append(f'{{let (tree,retired,empty,resolved)=observe({first},{last},{r},{rustentries});assert_eq!(tree,{rustfinal},"{name} final tree");assert_eq!(retired,vec!{arr(retired)},"{name} retirement order");assert!(!empty,"self-link retains root occupancy");assert!(!resolved.contains(&ROOT_ID));}}')
 records.append(dict(name=name,first=first,last=last,index=r,entries=entries,initial_budget=scenarios[len(records)][-1],steps=history,resumes=resumes,final=final,retired=retired))
def write(path,text):
 if '--check'in sys.argv:assert path.read_text()==text,('stale',path)
 else:path.parent.mkdir(parents=True,exist_ok=True);path.write_text(text)
prefix='''// SPDX-License-Identifier: MIT OR Apache-2.0
use values::recursive_cleanup::begin;
use values::recursive_cleanup::step;
use values::recursive_cleanup::RecursiveCursor;
use values::recursive_cleanup::RecursiveStep;
use values::cleanup_ranges::RangeStatus;
use values::cleanup_branch::CleanupOutcome;
'''
footer='\nconst RESULT:i32=test();\nmachine require_ok(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_ok(RESULT);}\n'
for i,row in enumerate(rows):
 q=row;body=f'let words:[u64;4]={arr(q["words"])};let ids:[u64;4]={arr(q["ids"])};let others:[bool;4]={arr([str(x).lower()for x in q["others"]])};\nlet cursor:RecursiveCursor=begin({q["page"]},{q["last"]},{q["budget"]},{q["index"]});let value:RecursiveStep=step(cursor,&words,&ids,&others);\n'
 c=[f'value.has_plan=={str(q["has_plan"]).lower()}',f'value.cursor.range.status in RangeStatus::{q["status"]}',f'next=={q["next"]}',f'last=={q["last"]}',f'budget=={q["remaining"]}',f'value.cursor.recursive_index=={q["index"]}',f'value.branch.retire_count=={len(q["retired"])}',f'value.branch.write_mask=={q["write"]}',f'value.branch.visited=={q["visited"]}']
 if q['has_plan']:c+=['value.branch.outcome in CleanupOutcome::Ready']
 c += [f'value.branch.words[{j}]=={w}'for j,w in enumerate(q['out'])]+[f'value.branch.frames_to_retire[{j}]=={v}'for j,v in enumerate(q['retired']+[0]*(3-len(q['retired'])))]
 body+='let next:u64=value.cursor.range.next_page;let last:u64=value.cursor.range.last_page;let budget:u64=value.cursor.range.budget;\ntransition '+' && '.join(c)+' {true -> (0) _ -> (1)}'
 write(HERE/'cases'/f'step-{i:03}.omg',prefix+f'// {q["scenario"]}\nmachine test()->i32 {{\n{body}\n}}'+footer)
write(HERE/'cases.json',json.dumps(dict(scenarios=records,steps=rows),indent=2)+'\n')
write(HERE/'src/main.rs',(HERE/'probe.rs.in').read_text().replace('// GENERATED_CHECKS','\n'.join(checks)))
print(len(records),'whole-tree witnesses;',len(rows),'Omega step fixtures')
