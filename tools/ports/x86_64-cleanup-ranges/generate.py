#!/usr/bin/env python3
"""Reference whole-tree outcomes plus every step of bounded cursor scenarios."""
import copy,json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
MASK=0x000ffffffffff000;DENSE=2**48-1;MAXPAGE=2**64-4096
def canonical(x):return x|0xffff000000000000 if x&(1<<47)else x
def indices(x):return [(x>>shift)&511 for shift in [39,30,21,12]]
def chain(page=0,two=False):
 i=indices(page);r=[(16384,i[0],4097),(4096,i[1],8193),(8192,i[2],12289)]
 if two:r.append((8192,i[2]+1,20481))
 return r
scenarios=[
 ('empty-root',0,MAXPAGE,[],4),
 ('empty-chain-all',0,MAXPAGE,chain(),4),
 ('two-empty-leaves',0,2**21,chain(two=True),4),
 ('partial-first-leaf',4096,8192,chain(two=True),4),
 ('later-leaf-only',2**21,2**21,chain(two=True),4),
 ('neighbor-nonpresent',0,0,chain()+[(12288,511,1<<63)],4),
 ('leaf-nonpresent',0,8192,chain()+[(12288,0,512)],4),
 ('huge-parent',0,2**21,[(16384,0,4097),(4096,0,8193),(8192,0,129)],4),
 ('nonpresent-parent',0,2**30,[(16384,0,4097),(4096,0,8192)],4),
 ('high-half',0xffff800000000000,0xffff800000200000,chain(0xffff800000000000,True),4),
 ('canonical-gap',0x7ffffffff000,0xffff800000000000,[(16384,256,4097),(4096,0,8193),(8192,0,12289)],4),
 ('last-page',MAXPAGE,MAXPAGE,chain(MAXPAGE),4),
 ('reversed',4096,0,chain(),4),
 ('budget-after-prefix',0,2**21,chain(two=True),1),
 ('budget-before-work',0,2**21,chain(two=True),0),
]
rows=[];checks=[]
def arr(xs):return '['+','.join(map(str,xs))+']'
for name,first,last,entries,budget in scenarios:
 tree={id:{}for id in [4096,8192,12288,16384,20480]}
 for id,index,word in entries:tree[id][index]=word
 page=first;retired=[];status='Done'if first>last else 'Exhausted'if budget==0 else 'Active';history=[]
 while status=='Active':
  slots=indices(page);ids=[16384,0,0,0];words=[0]*4;others=[False]*4;depth=0
  while True:
   table=tree[ids[depth]];words[depth]=table.get(slots[depth],0);others[depth]=any(word!=0 and index!=slots[depth]for index,word in table.items())
   if depth==3 or words[depth]&129!=1:break
   ids[depth+1]=words[depth]&MASK;depth+=1
  out=words.copy();freed=[];mask=0;d=depth
  while d>0 and out[d]==0 and not others[d]:
   freed.append(ids[d]);out[d-1]=0;mask|=1<<(d-1);tree[ids[d-1]][slots[d-1]]=0;d-=1
  retired+=freed
  span=2**48 if out[0]==0 and not others[0]else 2**39 if out[0]&129!=1 else 2**30 if out[1]&129!=1 else 2**21
  covered_end=((page&DENSE)|(span-1))&~4095;remaining=budget-1
  done=(last&DENSE)<=covered_end;nextpage=page if done else canonical(covered_end+4096)
  nextstatus='Done'if done else'Exhausted'if remaining==0 else'Active'
  row=dict(scenario=name,page=page,last=last,budget=budget,ids=ids,words=words,others=others,out=out,retired=freed,write=mask,visited=depth+1,next=nextpage,remaining=remaining,status=nextstatus)
  rows.append(row);history.append(len(rows)-1);page=nextpage;budget=remaining;status=nextstatus
 if status=='Done':
  final=sorted((id,i,w)for id,table in tree.items()for i,w in table.items()if w)
  rustentries='&['+','.join(f'({a},{b},{c})'for a,b,c in entries)+']'
  rustfinal='vec!['+','.join(f'({a},{b},{c})'for a,b,c in final)+']'
  checks.append(f'assert_eq!(observe({first},{last},{rustentries}),({rustfinal},vec!{arr(retired)}),"{name}");')
def write(path,text):
 if '--check'in sys.argv:assert path.read_text()==text,('stale',path)
 else:path.write_text(text)
prefix='// SPDX-License-Identifier: MIT OR Apache-2.0\nuse x86::cleanup_ranges;\nuse x86::cleanup_ranges::Cursor;\nuse x86::cleanup_ranges::RangeStep;\nuse x86::cleanup_ranges::RangeStatus;\nuse x86::cleanup_branch::CleanupOutcome;\n'
blocks=[]
for i,r in enumerate(rows):
 booleans=[str(x).lower()for x in r['others']]
 body=f'let words:[u64;4]={arr(r["words"])};let ids:[u64;4]={arr(r["ids"])};let others:[bool;4]={arr(booleans)};let cursor:Cursor=cleanup_ranges::begin({r["page"]},{r["last"]},{r["budget"]});let value:RangeStep=cleanup_ranges::step(cursor,&words,&ids,&others);'
 c=['value.has_plan','value.branch.outcome in CleanupOutcome::Ready',f'value.cursor.status in RangeStatus::{r["status"]}',f'value.cursor.next_page=={r["next"]}',f'value.cursor.last_page=={r["last"]}',f'value.cursor.budget=={r["remaining"]}',f'value.branch.retire_count=={len(r["retired"])}',f'value.branch.write_mask=={r["write"]}',f'value.branch.visited=={r["visited"]}']
 c += [f'value.branch.words[{j}]=={w}'for j,w in enumerate(r['out'])]
 c += [f'value.branch.frames_to_retire[{j}]=={f}'for j,f in enumerate(r['retired']+[0]*(3-len(r['retired'])))]
 body+='transition '+' && '.join(c)+' { true -> (0) _ -> (1) }'
 blocks.append(f'machine range_{i}()->i32 {{ {body} }}')
write(HERE/'cases.json',json.dumps(dict(scenarios=scenarios,steps=rows),indent=2)+'\n')
write(HERE/'main.omg',prefix+'\n'.join(blocks)+'\ndata Main{}\nmachine Main::main(&mut self){}\n')
write(HERE/'src/main.rs',(HERE/'probe.rs.in').read_text().replace('// GENERATED_CHECKS','\n'.join(checks)))
print(len(scenarios),'scenarios;',len(rows),'actual step fixtures;',len(checks),'whole-range Rust witnesses')
