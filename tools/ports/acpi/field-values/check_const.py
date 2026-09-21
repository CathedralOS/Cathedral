#!/usr/bin/env python3
"""Representative actual read assembly bodies evaluated as constants, with body controls."""
import argparse,hashlib,json,subprocess,tempfile,time
from pathlib import Path
import fixtures,check
HERE=fixtures.HERE
DEFAULT_COMPILER=Path('/tmp/cathedral-omega-eaa7993/release/omega')
def selected():
 names=['geometry_2_1_17','pattern_4_4_33','count_capacity_1_18446744073709551615'];rows=fixtures.cases();return [next(r for r in rows if r['name']==name)for name in names]

def source(row,control):return fixtures.HEAD+'machine test_result()->i32 {\n'+fixtures.body(row,control)+'}\nconst TEST_RESULT:i32=test_result();\nmachine require_ok(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_ok(TEST_RESULT);}\n'
def main():
 p=argparse.ArgumentParser();p.add_argument('--compiler',type=Path,default=DEFAULT_COMPILER);a=p.parse_args();inputs=check.snapshot();binary=check.sha(a.compiler);proofs=[];start=time.monotonic()
 for row in selected():
  for control in [False,True]:
   text=source(row,control)
   with tempfile.TemporaryDirectory(prefix='cathedral-field-values-const-')as d:
    work=Path(d);(work/'build.omg').write_text(check.build_text());(work/'main.omg').write_text(text)
    result=subprocess.run([str(a.compiler),'--check',str(work/'main.omg')],capture_output=True,text=True);output=result.stdout+result.stderr
   if control:assert result.returncode!=0 and 'cannot prove requires contract'in output and '1 == 0'in output,output
   else:assert result.returncode==0,output
   proofs.append(dict(case=row['name'],control=control,source_sha256=hashlib.sha256(text.encode()).hexdigest(),output=output));print('PASS constant',row['name'],'control',control,flush=True)
   assert inputs==check.snapshot() and binary==check.sha(a.compiler),'inputs changed'
 (HERE/'const-verification.json').write_text(json.dumps(dict(stage='constant evaluation and requires proof; no native or namespace evaluation',omega_revision=check.PIN,build_sha256=hashlib.sha256(check.build_text().encode()).hexdigest(),input_sha256=inputs,compiler_sha256=binary,compiler_path=str(a.compiler),positive_count=len(selected()),control_count=len(selected()),elapsed_seconds=round(time.monotonic()-start,3),proofs=proofs),indent=2,sort_keys=True)+'\n')
if __name__=='__main__':main()
