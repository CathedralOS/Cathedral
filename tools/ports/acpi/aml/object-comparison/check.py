#!/usr/bin/env python3
"""Primary-rule vectors and actual checked Omega bodies, including negative controls."""
import argparse,hashlib,json,os,re,subprocess,tempfile,time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
RUNNER=Path('/tmp/cathedral-acpi-execution-checked/release/cathedral-acpi-checked-runner')
COMPILER=Path('/tmp/cathedral-omega-eaa7993/release/omega')
PIN='eaa7993a23623cd8fabf45350340479c5c9c7879'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
from fixtures import cases,IMPORTS,HELPERS,body
def authored(rows):
 text=IMPORTS+HELPERS+'data Suite{}\n';names=[]
 for row in rows:
  for control in [False,True]:
   name='Suite::'+row['name']+('_control'if control else'_positive');names.append(name+'='+str(int(control)));text+='machine '+name+'(&mut self)->i32{'+body(row,control)+'}\n'
 return text,names
def build(root=ROOT):return 'machine build(builder:&mut Build){builder.application("cathedral-object-comparison-checks");builder.freestanding=true;builder.depend_as("aml",Source::Path {location:"'+str(root/'source/libraries/acpi/aml')+'"});builder.depend_as("integer_helpers",Source::Path {location:"'+str(root/'source/libraries/acpi/interpreter')+'"});}'
SOURCE=['source/libraries/acpi/aml/build.omg', 'source/libraries/acpi/aml/byte_storage.omg', 'source/libraries/acpi/aml/bytes.omg', 'source/libraries/acpi/aml/model.omg', 'source/libraries/acpi/aml/names.omg', 'source/libraries/acpi/aml/namespace.omg', 'source/libraries/acpi/aml/object_conversions.omg', 'source/libraries/acpi/aml/object_references.omg', 'source/libraries/acpi/interpreter/buffer_fields.omg', 'source/libraries/acpi/interpreter/build.omg', 'source/libraries/acpi/interpreter/conversions.omg', 'source/libraries/acpi/interpreter/integers.omg', 'source/libraries/acpi/interpreter/string_numbers.omg', 'source/libraries/acpi/interpreter/implicit_integer.omg', 'source/libraries/acpi/interpreter/implicit_strings.omg', 'source/libraries/acpi/aml/implicit_conversions.omg', 'source/libraries/acpi/interpreter/byte_comparison.omg', 'source/libraries/acpi/aml/object_comparison.omg']
def snapshot():
 paths=[HERE/n for n in ['check.py','fixtures.py','cases.json']]+[ROOT/p for p in SOURCE]+[ROOT/'tools/ports/acpi/interpreter/execution'/n for n in ['checked_runner.rs','runner.Cargo.lock']]
 return {str(p.relative_to(ROOT)):sha(p)for p in paths}
def validate(out,names):
 assert out.count('CHECKED authored package and dependency bodies;')==1
 actual=re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=',out,re.M);assert len(actual)==len(names)
 for (name,want,got),selection in zip(actual,names):assert name+'='+want==selection and want==got
def selected(rows):return [next(r for r in rows if r['name']==n)for n in ['matrix_64_1_4','lex_5a_4141','unsigned_32_4294967296_0']]
def const_source(row,control):return IMPORTS+HELPERS+'machine oc_constant_result()->i32{'+body(row,control)+'}\nconst RESULT:i32=oc_constant_result();\nmachine require_ok(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_ok(RESULT);}\n'
def checked_batch(batch):
 text,selections=authored(batch);start=time.monotonic()
 with tempfile.TemporaryDirectory(prefix='cathedral-object-comparison-batch-')as d:
  work=Path(d);(work/'build.omg').write_text(build());(work/'main.omg').write_text(text)
  r=subprocess.run([str(RUNNER),str(work/'main.omg'),str(work/'build'),*selections],capture_output=True,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'));out=r.stdout+r.stderr;assert r.returncode==0,out;validate(out,selections)
 return dict(cases=[r['name']for r in batch],source_sha256=hashlib.sha256(text.encode()).hexdigest(),output=out,seconds=round(time.monotonic()-start,3))
def run():
 rows=cases();assert json.loads((HERE/'cases.json').read_text())==rows;inputs=snapshot();runner=sha(RUNNER);compiler=sha(COMPILER);source,names=authored(rows);proofs=[];batches=[];start=time.monotonic();completed=0
 with ThreadPoolExecutor(max_workers=3)as pool:
  for batch in pool.map(checked_batch,[rows[i:i+10]for i in range(0,len(rows),10)]):
   batches.append(batch);completed+=len(batch['cases']);print('PASS',completed,'/',len(rows),'checked pairs',flush=True)
 with tempfile.TemporaryDirectory(prefix='cathedral-object-comparison-const-')as d:
  work=Path(d);(work/'build.omg').write_text(build())
  for row in selected(rows):
   for control in [False,True]:
    text=const_source(row,control);(work/'main.omg').write_text(text);r=subprocess.run([str(COMPILER),'--check',str(work/'main.omg')],capture_output=True,text=True);output=r.stdout+r.stderr
    if control:assert r.returncode!=0 and 'cannot prove requires contract'in output and '1 == 0'in output,output
    else:assert r.returncode==0,output
    proofs.append(dict(case=row['name'],control=control,source_sha256=hashlib.sha256(text.encode()).hexdigest(),output=output));print('PASS const',row['name'],control,flush=True)
 assert inputs==snapshot()and runner==sha(RUNNER)and compiler==sha(COMPILER)
 (HERE/'verification.json').write_text(json.dumps(dict(omega_revision=PIN,input_sha256=inputs,runner_sha256=runner,compiler_sha256=compiler,source_sha256=hashlib.sha256(source.encode()).hexdigest(),execution_root=str(ROOT),build_sha256=hashlib.sha256(build().encode()).hexdigest(),workers=3,batch_seconds_sum=round(sum(b['seconds']for b in batches),3),positive_count=len(rows),control_count=len(rows),batches=batches,constant_proofs=proofs,seconds=round(time.monotonic()-start,3)),indent=2,sort_keys=True)+'\n')
def verify():
 rows=cases();assert json.loads((HERE/'cases.json').read_text())==rows;r=json.loads((HERE/'verification.json').read_text());source,names=authored(rows)
 assert r['workers']==3 and r['batch_seconds_sum']==round(sum(b['seconds']for b in r['batches']),3)
 assert r['build_sha256']==hashlib.sha256(build(Path(r['execution_root'])).encode()).hexdigest()
 assert r['omega_revision']==PIN and r['input_sha256']==snapshot()and r['runner_sha256']==sha(RUNNER)and r['compiler_sha256']==sha(COMPILER)
 assert r['positive_count']==r['control_count']==len(rows)and r['source_sha256']==hashlib.sha256(source.encode()).hexdigest();assert len(r['constant_proofs'])==6
 seen=[];lookup={row['name']:row for row in rows}
 for batch in r['batches']:
  text,selections=authored([lookup[n]for n in batch['cases']]);assert batch['source_sha256']==hashlib.sha256(text.encode()).hexdigest();validate(batch['output'],selections);seen+=batch['cases']
 assert seen==[row['name']for row in rows]
 for row,pair in zip(selected(rows),zip(r['constant_proofs'][::2],r['constant_proofs'][1::2])):
  for control,proof in zip([False,True],pair):
   assert proof['case']==row['name']and proof['control']==control and proof['source_sha256']==hashlib.sha256(const_source(row,control).encode()).hexdigest()
   assert ('cannot prove requires contract'in proof['output']and'1 == 0'in proof['output'])if control else'compiled 'in proof['output']
 print('PASS',len(rows),'checked pairs and 3 const pairs; exact current hashes')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--write-cases',action='store_true');p.add_argument('--verify-record',action='store_true');a=p.parse_args()
 if a.write_cases:(HERE/'cases.json').write_text(json.dumps(cases(),indent=2,sort_keys=True)+'\n')
 elif a.verify_record:verify()
 else:run()
