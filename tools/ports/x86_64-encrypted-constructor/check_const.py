#!/usr/bin/env python3
"""Selected compact fixture bodies evaluated as constants under zero contracts."""
import hashlib,json,subprocess,tempfile
from pathlib import Path
from check_interpreted import HERE,ROOT,snapshot
compiler=Path('/tmp/cathedral-omega-eaa7993/release/omega');before=snapshot();runs=[]
for profile in [0,9]:
 for control in [False,True]:
  name=f'p{profile}_'+('control'if control else'positive')
  with tempfile.TemporaryDirectory(prefix='cathedral-constructor-compact-const-')as directory:
   work=Path(directory);body=(HERE/'compact_suite.omg').read_text();old=f'machine Suite::{name}(&mut self)';assert body.count(old)==1;body=body.replace(old,'machine scenario()')+"\nconst RESULT:i32=scenario();\nmachine require_success(value:i32) requires value==0; {}\ndata Main{}\nmachine Main::main(&mut self){require_success(RESULT);}\n"
   (work/'main.omg').write_text(body);(work/'build.omg').write_text((HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/'))
   run=subprocess.run([str(compiler),'--check',str(work/'main.omg')],capture_output=True,text=True);output=run.stdout+run.stderr
   passed=run.returncode!=0 and 'cannot prove requires contract'in output and '1 == 0'in output if control else run.returncode==0
   assert passed,(name,output);runs.append(dict(name=name,control=control,source_sha256=hashlib.sha256(body.encode()).hexdigest(),output=output));print('PASS',name,flush=True)
assert snapshot()==before
(HERE/'constant-verification.json').write_text(json.dumps(dict(stage='actual constant evaluation of compact helper bodies; native not run',compiler_sha256=hashlib.sha256(compiler.read_bytes()).hexdigest(),source_sha256=before,runs=runs),indent=2)+'\n')
