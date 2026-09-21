#!/usr/bin/env python3
"""Finite snapshot/cursor oracle, independently checked by pinned Rust cleanup."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
MASK=0x000ffffffffff000; DENSE=2**48-1; MAX=2**64-4096; ROOT=262144
IDS=[0,65536,131072,196608,ROOT,327680,393216,458752]
G=2**39; HI=0xffff800000000000
def canonical(x):return x|0xffff000000000000 if x&(1<<47) else x
def indices(x):return [(x>>s)&511 for s in [39,30,21,12]]
def arr(xs):return '['+','.join(str(x).lower() if isinstance(x,bool) else str(x) for x in xs)+']'
def write(path,text):
 if '--check' in sys.argv:assert path.read_text()==text,('stale',path)
 else:path.parent.mkdir(parents=True,exist_ok=True);path.write_text(text)
profiles=[]
for name,configs in [('disabled',[]),('encrypted0',[(0,False)]),('encrypted7',[(7,False)]),('encrypted12',[(12,False)]),('encrypted21',[(21,False)]),('encrypted30',[(30,False)]),('encrypted47',[(47,False)]),('shared47',[(47,True)]),('encrypted51',[(51,False)]),('shared63',[(63,True)]),('repeated47-48',[(47,False),(48,True)])]:
 mask=MASK;marker=0
 for bit,shared in configs:mask &= ~(1<<bit);marker |= 1<<bit
 profiles.append(dict(name=name,configs=configs,address_mask=mask,marker=marker))
def scenarios(p,recursive):
 marker=p['marker']; nonzero=marker or 512
 def chain(page=0,second=False,marked=True):
  a,b,c=[327680,393216,458752] if second else [65536,131072,196608]
  ix=indices(page);m=marker if marked else 0
  return [(ROOT,ix[0],a|1|m),(a,ix[1],b|1|m),(b,ix[2],c|1|m)]
 base=chain(); bare=chain(marked=False)
 result=[
  ('empty-root',0,0 if recursive else MAX,[],4,511),
  ('marked-chain',0,0,base,4,511),
  ('bare-chain',0,0,bare,4,511),
  ('leaf-marker-only',0,0,bare+[(196608,0,nonzero)],4,511),
  ('neighbor-marker-only',0,0,bare+[(196608,511,nonzero)],4,511),
  ('middle-neighbor',0,0,bare+[(131072,511,nonzero)],4,511),
  ('root-neighbor',0,0,bare+[(ROOT,510,nonzero)],4,511),
  ('p2-nonpresent',0,0,bare[:2]+[(131072,0,(196608|marker)&~1)],4,511),
  ('p3-nonpresent',0,0,bare[:1]+[(65536,0,(131072|marker)&~1)],4,511),
  ('p4-nonpresent',0,0,[(ROOT,0,(65536|marker)&~1)],4,511),
  ('p2-huge',0,0,bare[:2]+[(131072,0,196608|marker|129)],4,511),
  ('p4-huge',0,0,[(ROOT,0,65536|marker|129)],4,511),
  ('zero-address-child',0,0,bare[:2]+[(131072,0,marker|1)],4,511),
 ]
 if p['name'] in ['disabled','encrypted12','encrypted47','shared63','repeated47-48']:
  result += [
   ('two-leaves-resume',0,2**21,bare+[(131072,1,327680|1|marker)],1,511),
   ('initial-zero-budget',0,2**21,bare+[(131072,1,327680|1|marker)],0,511),
   ('two-roots-around-self',0,2*G,chain(marked=False)+chain(2*G,True,False),1,1),
   ('gap',0x7ffffffff000,HI,chain(0x7ffffffff000,marked=False)+chain(HI,True,False),1,0),
   ('last-page',MAX,MAX,chain(MAX,marked=False),2**64-1,0),
   ('reversed',4096,0,bare,4,511),
   ('high-budget',0,0,bare,2**64-1,511),
  ]
 if recursive:
  result += [('skip-zero',0,G-4096,[],2**64-1,0),('skip-final',MAX,MAX,[],1,511)]
 if p['name']=='repeated47-48':
  result += [('old-bit-only-leaf',0,0,bare+[(196608,0,1<<47)],4,511),('old-bit-only-neighbor',0,0,bare+[(196608,511,1<<47)],4,511),('old-bit-links',0,0,[(id,i,w|(1<<47))for id,i,w in bare],4,511)]
 return result

rows=[];worlds=[];rust=[]
for pi,p in enumerate(profiles):
 for recursive in [False,True]:
  for name,first,last,seed,budget,r in scenarios(p,recursive):
   entries=seed+([(ROOT,r,ROOT|1|p['marker'])] if recursive else [])
   assert len({(id,i)for id,i,w in entries})==len(entries)
   tree={id:{} for id in IDS}
   for id,i,w in entries:tree[id][i]=w
   page=first;retired=[];history=[];initial_budget=budget;resumes=0
   status='Done' if first>last else 'Exhausted' if budget==0 else 'Active'
   while status!='Done':
    if status=='Exhausted':budget=2;status='Active';resumes+=1
    assert len(history)<2048
    slots=indices(page);ids=[ROOT,0,0,0];words=[0]*4;others=[False]*4;depth=0
    skip=recursive and slots[0]==r
    if skip:
     words=[129,2**64-1,17,33];ids=[0,1,2,3];out=[0]*4;freed=[];write_mask=0;visited=0;span=G
    else:
     while True:
      table=tree[ids[depth]];words[depth]=table.get(slots[depth],0);others[depth]=any(w!=0 and i!=slots[depth] for i,w in table.items())
      if depth==3 or words[depth]&129!=1:break
      ids[depth+1]=words[depth]&p['address_mask'];depth+=1
     out=words.copy();freed=[];write_mask=0;d=depth;visited=depth+1
     while d>0 and out[d]==0 and not others[d]:
      freed.append(ids[d]);out[d-1]=0;write_mask|=1<<(d-1);tree[ids[d-1]][slots[d-1]]=0;d-=1
     if recursive:assert others[0], 'root self-link belongs to other-entry occupancy'
     span=2**48 if out[0]==0 and not others[0] else G if out[0]&129!=1 else 2**30 if out[1]&129!=1 else 2**21
    retired+=freed;end=((page&DENSE)|(span-1))&~4095;remaining=budget-1
    done=(last&DENSE)<=end;nextpage=page if done else canonical(end+4096)
    nextstatus='Done' if done else 'Exhausted' if remaining==0 else 'Active'
    rows.append(dict(profile=pi,recursive=recursive,scenario=name,page=page,last=last,budget=budget,index=r,ids=ids,words=words,others=others,has_plan=not skip,out=out,retired=freed,write=write_mask,visited=visited,next=nextpage,remaining=remaining,status=nextstatus))
    history.append(len(rows)-1);page=nextpage;budget=remaining;status=nextstatus
   final=sorted((id,i,w)for id,t in tree.items() for i,w in t.items() if w)
   world=dict(profile=pi,recursive=recursive,name=name,first=first,last=last,index=r,entries=entries,initial_budget=initial_budget,resumes=resumes,steps=history,final=final,retired=retired)
   worlds.append(world)
   tup=lambda values:'['+','.join('('+','.join(map(str,v))+')' for v in values)+']'
   empty=not any(id==ROOT for id,i,w in final)
   rust.append(f'if profile=={pi} {{let (tree,retired,empty,_)=observe({str(recursive).lower()},{first},{last},{r},&{tup(entries)});assert_eq!(tree,vec!{tup(final)},"{pi}/{recursive}/{name} final");assert_eq!(retired,vec!{arr(retired)},"{pi}/{recursive}/{name} retirement");assert_eq!(empty,{str(empty).lower()});}}')

prefix='''// SPDX-License-Identifier: MIT OR Apache-2.0
use x86::memory_encryption;
use x86::memory_encryption::State;
use x86::memory_encryption::Configuration;
use x86::encrypted_cleanup_ranges;
use x86::encrypted_cleanup_recursive;
use x86::cleanup_ranges::Cursor;
use x86::cleanup_ranges::RangeStep;
use x86::cleanup_ranges::RangeStatus;
use x86::cleanup_branch::CleanupOutcome;
use x86::recursive_cleanup::RecursiveCursor;
use x86::recursive_cleanup::RecursiveStep;
'''
footer='\nconst TEST_RESULT:i32=test_result();\nmachine require_success(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_success(TEST_RESULT);}\n'
fixtures=[];controls=[]
for n,q in enumerate(rows):
 p=profiles[q['profile']];code='    let profile: State = memory_encryption::initial();\n'
 for bit,shared in p['configs']:code+=f'    _ = memory_encryption::configure(&mut profile, Configuration::{"SharedBit" if shared else "EncryptedBit"} {{ position: {bit} }});\n'
 code+=f'    let words:[u64;4]={arr(q["words"])}; let ids:[u64;4]={arr(q["ids"])}; let others:[bool;4]={arr(q["others"])};\n'
 if q['recursive']:
  code+=f'    let cursor:RecursiveCursor=encrypted_cleanup_recursive::begin({q["page"]},{q["last"]},{q["budget"]},{q["index"]});\n    let value:RecursiveStep=encrypted_cleanup_recursive::step(profile,cursor,&words,&ids,&others);\n'
  cursor='value.cursor.range';extra=[f'value.cursor.recursive_index=={q["index"]}']
 else:
  code+=f'    let cursor:Cursor=encrypted_cleanup_ranges::begin({q["page"]},{q["last"]},{q["budget"]});\n    let value:RangeStep=encrypted_cleanup_ranges::step(profile,cursor,&words,&ids,&others);\n'
  cursor='value.cursor';extra=[]
 code+=f'    let next:u64={cursor}.next_page; let last:u64={cursor}.last_page; let budget:u64={cursor}.budget;\n'
 checks=[f'value.has_plan=={str(q["has_plan"]).lower()}',f'{cursor}.status in RangeStatus::{q["status"]}',f'next=={q["next"]}',f'last=={q["last"]}',f'budget=={q["remaining"]}',f'value.branch.retire_count=={len(q["retired"])}',f'value.branch.write_mask=={q["write"]}',f'value.branch.visited=={q["visited"]}']+extra
 if q['has_plan']:checks+=['value.branch.outcome in CleanupOutcome::Ready']
 checks += [f'value.branch.words[{i}]=={w}'for i,w in enumerate(q['out'])]+[f'value.branch.frames_to_retire[{i}]=={v}'for i,v in enumerate(q['retired']+[0]*(3-len(q['retired'])))]
 code+='    transition '+' && '.join(checks)+' {true -> (0) _ -> (1)}\n'
 path=f'cases/step-{n:03}.omg';write(HERE/path,prefix+f'// {p["name"]} / {q["recursive"]} / {q["scenario"]}\nmachine test_result()->i32 {{\n'+code+'}\n'+footer)
 fixtures.append(dict(path=path,step=n))
 picks=[('masked-retirement',q['profile']==6 and not q['recursive'] and q['scenario']=='marked-chain',f'value.branch.frames_to_retire[0]=={q["retired"][0]}' if q['retired'] else '',f'value.branch.frames_to_retire[0]=={(q["retired"][0]|p["marker"])}' if q['retired'] else ''),('old-bit-nonempty',q['profile']==10 and not q['recursive'] and q['scenario']=='old-bit-only-leaf','value.branch.retire_count==0','value.branch.retire_count==3'),('partial-upward-retirement',q['profile']==6 and not q['recursive'] and q['scenario']=='middle-neighbor','value.branch.retire_count==1','value.branch.retire_count==3'),('recursive-exclusion',q['profile']==6 and q['recursive'] and q['scenario']=='skip-final','value.has_plan==false','value.has_plan==true'),('huge-stops',q['profile']==2 and not q['recursive'] and q['scenario']=='marked-chain','value.branch.visited==1','value.branch.visited==4'),('repeated-mask-retirement',q['profile']==10 and q['recursive'] and q['scenario']=='old-bit-links',f'value.branch.frames_to_retire[0]=={q["retired"][0]}' if q['retired'] else '',f'value.branch.frames_to_retire[0]=={(q["retired"][0]|(1<<47))}' if q['retired'] else '')]
 for family,selected,old,new in picks:
  if selected:assert code.count(old)==1 and old!=new;controls.append(dict(path=path,family=family,old=old,new=new))
config=[]
for n,p in enumerate(profiles):
 for bit,shared in p['configs']:config.append(f'if profile=={n} {{unsafe {{enable_memory_encryption(MemoryEncryptionConfiguration::{"SharedBit" if shared else "EncryptedBit"}({bit}))}};}}')
source=(HERE/'probe.rs.in').read_text().replace('// GENERATED_CONFIGURATIONS','\n'.join(config)).replace('// GENERATED_CHECKS','\n'.join(rust))
write(HERE/'src/main.rs',source)
write(HERE/'profiles.json',json.dumps(profiles,indent=2)+'\n')
write(HERE/'cases.json',json.dumps(dict(scenarios=worlds,steps=rows),indent=2)+'\n')
write(HERE/'fixtures.json',json.dumps(dict(batches=fixtures,controls=controls),indent=2)+'\n')
print(len(worlds),'whole-tree Rust cases;',len(rows),'Omega cursor/branch bodies;',len(controls),'controls')
