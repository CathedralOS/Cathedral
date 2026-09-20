#!/usr/bin/env python3
"""Pinned mapper witnesses for detached leaf and child-creation decisions."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;MASK=0x000ffffffffff000;MAX=2**64-1
rows=[]
for size in [4096,2**21,2**30]:
 for word in [0,128,512,1,129,0x40000001,0x40000081,0x40001081]:
  for op in ['unmap','update','translate']:
   frame=word&MASK;kind='Ready';out=word;write=False;payload=frame
   if op=='unmap':
    if not word&1:kind='PageNotMapped'
    elif bool(word&128)!=(size!=4096):kind='ParentHugePage'
    elif frame%size:kind='InvalidFrameAddress'
    else:out=0;write=True
   elif op=='translate':
    if word==0:kind='PageNotMapped'
    elif frame%size:kind='InvalidFrameAddress'
   else:
    if word==0:kind='PageNotMapped'
    else:out=frame|0x8000000000000003|(128 if size!=4096 else 0);write=True
   rows.append(dict(op=op,word=word,size=size,frame=0x80000000,flags=0x8000000000000003,kind=kind,out=out,write=write,payload=payload))
 for word,flags in [(0,3),(0,128),(0,0),(512,3),(1,3),(1,128)]:
  kind='AlreadyMapped' if word else 'InvalidHugeInput' if size==4096 and flags&128 else 'Ready'
  out=word if kind!='Ready' else 0x80000000|flags|(128 if size!=4096 else 0)
  rows.append(dict(op='map',word=word,size=size,frame=0x80000000,flags=flags,kind=kind,out=out,write=kind=='Ready',payload=0x80000000))
 for flags in [0,0x2003,MAX]:
  rows.append(dict(op='update',word=0x40000001,size=size,frame=0x80000000,flags=flags,kind='Ready',out=0x40000000|flags|(128 if size!=4096 else 0),write=True,payload=0x40000000))
children=[]
for word,flags,supply in [(0,3,0x1000),(0,3,None),(0,0,0x1000),(0,128,0x1000),(0x1001,0,None),(0x1001,3,None),(0x1001,129,None),(0x1000,0,None),(0x1000,1,None),(0x1081,2,None),(0x1001,0x2000,None),(0,128,None),(0x1001,0,1)]:
 out=word;write=False;zero=False;kind='Ready'
 if word==0:
  if supply is None:kind='AllocationFailed'
  elif flags&128:kind='InvalidHugeInput'
  else:out=supply|flags;write=True;zero=True
 else:write=flags!=0 and ((word&~MASK)&flags)!=flags;out=word|flags if write else word
 if kind=='Ready':
  if not out&1:kind='MissingPresent'
  elif out&128:kind='HugeParent'
 children.append(dict(word=word,flags=flags,supply=supply,kind=kind,out=out,write=write,zero=zero,address=out&MASK))
prefix='// SPDX-License-Identifier: MIT OR Apache-2.0\nuse x86::mapping_plans;\nuse x86::mapping_plans::LeafPlan;\nuse x86::mapping_plans::ChildPlan;\nuse x86::mapping_plans::FrameSupply;\n'
blocks=[]
for i,r in enumerate(rows):
 calls={'map':f'map_leaf({r["word"]},{r["frame"]},{r["size"]},{r["flags"]})','unmap':f'unmap_leaf({r["word"]},{r["size"]})','update':f'update_leaf_flags({r["word"]},{r["size"]},{r["flags"]})','translate':f'translate_leaf({r["word"]},{r["size"]})'}
 body=f'let value:LeafPlan=mapping_plans::{calls[r["op"]]}; let common:bool=value.word == {r["out"]} && value.write == {str(r["write"]).lower()};\n'
 if r['kind'] in ['Ready','InvalidFrameAddress','AlreadyMapped']:
  field={'Ready':'frame','InvalidFrameAddress':'address','AlreadyMapped':'requested_frame'}[r['kind']]
  body+=f'transition value {{ LeafPlan::{r["kind"]} {{ {field} }} -> payload(common,{field}) _ -> (1) }} state payload(common:bool,number:u64)->i32 {{ transition common && number == {r["payload"]} {{ true -> (0) _ -> (1) }} }}'
 else:body+=f'transition common && value in LeafPlan::{r["kind"]} {{ true -> (0) _ -> (1) }}'
 blocks.append(f'machine leaf_{i}()->i32 {{ {body} }}')
for i,r in enumerate(children):
 supply='FrameSupply::Unavailable' if r['supply'] is None else f'FrameSupply::Supplied {{ address:{r["supply"]} }}'
 body=f'let value:ChildPlan=mapping_plans::prepare_child({r["word"]},{r["flags"]},{supply}); let common:bool=value.word == {r["out"]} && value.write == {str(r["write"]).lower()};\n'
 if r['kind']=='Ready':body+=f'transition value {{ ChildPlan::Ready {{ address,zero_child }} -> payload(common,address,zero_child) _ -> (1) }} state payload(common:bool,address:u64,zero:bool)->i32 {{ transition common && address == {r["address"]} && zero == {str(r["zero"]).lower()} {{ true -> (0) _ -> (1) }} }}'
 else:body+=f'transition common && value in ChildPlan::{r["kind"]} {{ true -> (0) _ -> (1) }}'
 blocks.append(f'machine child_{i}()->i32 {{ {body} }}')
def write(path,text):
 if '--check' in sys.argv:assert path.read_text()==text,('stale fixture',path)
 else:path.write_text(text)
write(HERE/'cases.json',json.dumps(dict(leaves=rows,children=children),indent=2)+'\n')
write(HERE/'main.omg',prefix+'\n'.join(blocks)+'\ndata Main{}\nmachine Main::main(&mut self){}\n')
rust=(HERE/'probe.rs.in').read_text()
kind={'Ready':0,'PageNotMapped':1,'ParentHugePage':2,'InvalidFrameAddress':3,'AlreadyMapped':4,'InvalidHugeInput':5}
checks=[]
for r in rows:
 typ={4096:'Size4KiB',2**21:'Size2MiB',2**30:'Size1GiB'}[r['size']]
 payload=r['payload'] if r['kind'] in ['Ready','InvalidFrameAddress','AlreadyMapped'] else 0
 # update_flags returns only a flush token; observe the original frame separately.
 checks.append(f'assert_eq!(leaf::<{typ}>("{r["op"]}",{r["word"]},{r["frame"]},{r["flags"]}),({kind[r["kind"]]},{r["out"]},{payload}));')
child_kind={'Ready':0,'AllocationFailed':1,'MissingPresent':2,'HugeParent':3,'InvalidHugeInput':4}
for r in children:
 supply='None' if r['supply'] is None else f'Some({r["supply"]})'
 checks.append(f'assert_eq!(child({r["word"]},{r["flags"]},{supply}),({child_kind[r["kind"]]},{r["out"]},{str(r["zero"] and r["kind"]=="Ready").lower()}));')
write(HERE/'src/main.rs',rust.replace('// GENERATED_CHECKS','\n'.join(checks)))
print(len(rows),'leaf cases;',len(children),'child cases')
