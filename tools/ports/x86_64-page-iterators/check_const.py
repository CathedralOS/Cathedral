#!/usr/bin/env python3
"""One direct gap-crossing constant body and a mutated expected cursor control."""
import hashlib,json,subprocess,tempfile,time
from pathlib import Path
import check,generate

def source(control):
 r=generate.rows()[352]
 expected=r['out_start']+(r['size'] if control else 0)
 return f'''// SPDX-License-Identifier: MIT OR Apache-2.0
use x86::page_iterators;
use x86::encrypted_frames::IteratorResult;
use x86::addresses::NumberResult;
machine test_result()->i32 {{
 let actual:IteratorResult=page_iterators::step({r['start']},{r['end']},{r['size']},true,{r['index']},false);
 let good:bool=actual.start=={expected} && actual.end=={r['out_end']} && actual.failed;
 transition actual.selection {{ NumberResult::Rejected -> result(good) _ -> (1) }}
 state result(good:bool)->i32 {{transition good {{true -> (0) _ -> (1)}}}}
}}
'''+generate.FOOT

def main():
 subprocess.run(['python3',str(generate.HERE/'check.py'),'--host-only'],check=True,cwd=generate.ROOT)
 before=check.snapshot();started=time.monotonic();outputs=[];hashes=[]
 with tempfile.TemporaryDirectory(prefix='cathedral-page-iterator-const-') as temp:
  work=Path(temp);(work/'build.omg').write_text((generate.HERE/'build.omg').read_text().replace('../../../source/',str(generate.ROOT/'source')+'/'))
  for control in [False,True]:
   body=source(control);hashes.append(hashlib.sha256(body.encode()).hexdigest());(work/'main.omg').write_text(body)
   result=subprocess.run([str(check.OMEGA),'--check',str(work/'main.omg')],cwd=generate.ROOT,text=True,capture_output=True)
   output=result.stdout+result.stderr
   assert (result.returncode!=0 and 'cannot prove requires contract' in output and '1 == 0' in output) if control else result.returncode==0,output
   label='body_control' if control else 'positive';outputs.append(label+'\n'+output);print('PASS constant',label,flush=True)
 assert before==check.snapshot(),'selected source/harness inputs changed'
 record=dict(format='cathedral-page-iterators-v1',stage='actual Omega constant evaluation; native not run',command='python3 tools/ports/x86_64-page-iterators/check_const.py',omega_revision='eaa7993a23623cd8fabf45350340479c5c9c7879',binary_sha256=hashlib.sha256(check.OMEGA.read_bytes()).hexdigest(),source_sha256=before,suite_sha256=hashes,case_count=1,group_count=1,control_count=1,observation_id=352,elapsed_seconds=round(time.monotonic()-started,3),output=''.join(outputs))
 (generate.HERE/'constant-verification.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
if __name__=='__main__':main()
