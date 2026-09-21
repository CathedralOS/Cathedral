#!/usr/bin/env python3
"""Representative actual const evaluation with changed expectation bodies."""
import argparse,hashlib,json,subprocess,tempfile,time
from pathlib import Path
import generate,check
HERE=generate.HERE;ROOT=generate.ROOT
COMPILER=Path('/tmp/cathedral-omega-eaa7993/release/omega')
def chosen():
 rows=generate.cases();tests=[]
 predicates=[
  lambda r:r['profile']=='e0' and r['op']=='containing' and r['start']==1 and r['size']==4096,
  lambda r:r['profile']=='e47_s48_e47' and r['op']=='from_start' and r['start']==1<<48 and r['size']==4096,
  lambda r:r['profile']=='e47' and r['op']=='arithmetic' and r['start']==(1<<47)-4096 and r['index']==1 and not r['reverse'],
  lambda r:r['profile']=='e0' and r['op']=='iterator' and r['inclusive'] and not r['reverse'] and r['start']==0 and r['index']==0 and r['size']==4096,
  lambda r:r['op']=='iterator' and r['failed'] and (r['out_start']!=r['start'] or r['out_end']!=r['end']),
  lambda r:r['profile']=='e47' and r['op']=='iterator' and r['inclusive'] and not r['reverse'] and r['start']==r['end'] and r['start']>1<<48 and r['index']==0 and r['size']==4096 and not r['failed'],
 ]
 for predicate in predicates:tests.append(next(r for r in rows if predicate(r)))
 return tests

def main():
 p=argparse.ArgumentParser();p.add_argument('--record',type=Path);a=p.parse_args();before=check.snapshot();results=[];start=time.monotonic()
 for row in chosen():
  group=dict(profile=row['profile'],cases=[row]);observations=[]
  for control in [False,True]:
   source=generate.IMPORTS+generate.HELPERS+generate.render(group,control)+'''const TEST_RESULT:i32=test_result();
machine require_ok(value:i32) requires value==0;{}
data Main{}
machine Main::main(&mut self){require_ok(TEST_RESULT);}
'''
   with tempfile.TemporaryDirectory(prefix='cathedral-encrypted-frame-const-')as directory:
    work=Path(directory);(work/'main.omg').write_text(source);(work/'build.omg').write_text((HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/'))
    result=subprocess.run([str(COMPILER),'--check',str(work/'main.omg')],capture_output=True,text=True);output=result.stdout+result.stderr
    if control:assert result.returncode and 'cannot prove requires contract'in output and '1 == 0'in output,output
    else:assert result.returncode==0,output
    observations.append(dict(control=control,fixture_sha256=hashlib.sha256(source.encode()).hexdigest(),output=output))
   print('PASS const case',row['case_id'],'control',control,flush=True)
  results.append(dict(case=row,observations=observations))
 assert before==check.snapshot(),'Source drift during const evaluation'
 if a.record:a.record.write_text(json.dumps(dict(format='cathedral-encrypted-frames-const-v1',stage='Omega constant evaluation and requires checking; native/hardware not run',compiler_sha256=hashlib.sha256(COMPILER.read_bytes()).hexdigest(),source_sha256=before,cases=results,elapsed_seconds=round(time.monotonic()-start,3)),indent=2,sort_keys=True)+'\n')
if __name__=='__main__':main()
