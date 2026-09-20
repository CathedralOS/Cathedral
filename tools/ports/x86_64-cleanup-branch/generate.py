#!/usr/bin/env python3
"""Original bounded cleanup branch fixtures and pinned full-table witnesses."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
rows=[]
for depth in range(4):
 for word in ([0,512,128,(4096*(depth+1))|129] if depth<3 else [0,512,128,1]):
  for others in ([0,2,4,8] if depth<3 else range(16)):
   words=[4097,8193,12289,0];words[depth]=word
   out=words.copy();current=0
   while current<3 and out[current]&1 and not out[current]&128:current+=1
   visited=current+1;retired=[];mask=0
   while current>0 and out[current]==0 and not others&(1<<current):
    retired.append(4096*current);out[current-1]=0;mask|=1<<(current-1);current-=1
   rows.append(dict(words=words,others=others,out=out,retired=retired,write=mask,visited=visited))
def arr(values):return '['+','.join(map(str,values))+']'
def write(path,text):
 if '--check' in sys.argv:assert path.read_text()==text,('stale',path)
 else:path.write_text(text)
prefix='// SPDX-License-Identifier: MIT OR Apache-2.0\nuse x86::cleanup_branch;\nuse x86::cleanup_branch::CleanupPlan;\nuse x86::cleanup_branch::CleanupOutcome;\n'
blocks=[];rust=[]
for i,r in enumerate(rows):
 others=[str(bool(r['others']&(1<<n))).lower() for n in range(4)]
 checks=[f'result.words[{j}]=={word}' for j,word in enumerate(r['out'])]
 checks += [f'result.frames_to_retire[{j}]=={frame}' for j,frame in enumerate(r['retired']+[0]*(3-len(r['retired']))) ]
 checks += [f'result.retire_count=={len(r["retired"])}',f'result.write_mask=={r["write"]}',f'result.visited=={r["visited"]}','result.outcome in CleanupOutcome::Ready']
 body=f'let words:[u64;4]={arr(r["words"])}; let ids:[u64;4]=[16384,4096,8192,12288]; let others:[bool;4]={arr(others)}; let result:CleanupPlan=cleanup_branch::plan(&words,&ids,&others,0); transition '+ ' && '.join(checks)+' { true -> (0) _ -> (1) }'
 blocks.append(f'machine cleanup_{i}()->i32 {{ {body} }}')
 rust.append(f'assert_eq!(observe({arr(r["words"])},{r["others"]}),({arr(r["out"])},vec!{arr(r["retired"])}),"case {i}");')
write(HERE/'cases.json',json.dumps(rows,indent=2)+'\n')
write(HERE/'main.omg',prefix+'\n'.join(blocks)+'\ndata Main{}\nmachine Main::main(&mut self){}\n')
write(HERE/'src/main.rs',(HERE/'probe.rs.in').read_text().replace('// GENERATED_CHECKS','\n'.join(rust)))
print(len(rows),'cleanup branch cases')
