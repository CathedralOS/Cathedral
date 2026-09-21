import argparse,hashlib,json,os,re,subprocess,tempfile,time
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT;PIN='eaa7993a23623cd8fabf45350340479c5c9c7879'
RUNNER=Path('/tmp/cathedral-acpi-execution-checked/release/cathedral-acpi-checked-runner');COMPILER=Path('/tmp/cathedral-omega-eaa7993/release/omega')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def snapshot():
 paths=list(HERE.glob('*.py'))+[HERE/n for n in ['cases.json','reference.rs','reference.Cargo.lock','public-verification.json']]+[ROOT/'source/libraries/acpi/aml'/n for n in ['build.omg','model.omg','names.omg','bytes.omg','namespace_removal.omg']]+[ROOT/'source/libraries/acpi/interpreter/build.omg']+[ROOT/'tools/ports/acpi/interpreter/execution'/n for n in ['checked_runner.rs','runner.Cargo.lock']]
 return {str(p.relative_to(ROOT)):sha(p)for p in sorted(paths)}
def build():return 'machine build(builder:&mut Build){builder.application("cathedral-namespace-removal-tests");builder.freestanding=true;builder.depend_as("aml",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/aml')+'"});}'
def validate(output,names):
 assert 'CHECKED authored package and dependency bodies;'in output
 actual=re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=',output,re.M);assert len(actual)==len(names)
 for (name,expected,observed),selection in zip(actual,names):assert name+'='+expected==selection and expected==observed

def selected():
 rows={r['name']:r for r in fixtures.cases()};return [rows[n]for n in ['subtree_preserves_same_path_object','bad_count_18446744073709551615']]
def const_source(row,control):
 helpers,_=fixtures.helpers();return fixtures.IMPORTS+helpers+'machine test_result()->i32{'+fixtures.body(row,control)+'}\nconst RESULT:i32=test_result();\nmachine require_ok(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_ok(RESULT);}\n'
def run():
 rows=fixtures.cases();assert json.loads((HERE/'cases.json').read_text())==rows;inputs=snapshot();runner=sha(RUNNER);compiler=sha(COMPILER);text,names=fixtures.render(rows);start=time.monotonic();proofs=[]
 with tempfile.TemporaryDirectory(prefix='cathedral-namespace-removal-check-')as d:
  work=Path(d);(work/'build.omg').write_text(build());(work/'main.omg').write_text(text)
  r=subprocess.run([str(RUNNER),str(work/'main.omg'),str(work/'build'),*names],capture_output=True,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'));output=r.stdout+r.stderr;assert r.returncode==0,output;validate(output,names);print('PASS',len(rows),'checked pairs',flush=True)
  for row in selected():
   for control in [False,True]:
    source=const_source(row,control);(work/'main.omg').write_text(source);r=subprocess.run([str(COMPILER),'--check',str(work/'main.omg')],capture_output=True,text=True);out=r.stdout+r.stderr
    if control:assert r.returncode!=0 and 'cannot prove requires contract'in out and '1 == 0'in out,out
    else:assert r.returncode==0,out
    proofs.append(dict(case=row['name'],control=control,source_sha256=hashlib.sha256(source.encode()).hexdigest(),output=out));print('PASS const',row['name'],control,flush=True)
 assert inputs==snapshot() and runner==sha(RUNNER) and compiler==sha(COMPILER)
 value=dict(omega_revision=PIN,input_sha256=inputs,runner_sha256=runner,compiler_sha256=compiler,source_sha256=hashlib.sha256(text.encode()).hexdigest(),positive_count=len(rows),control_count=len(rows),output=output,constant_proofs=proofs,seconds=round(time.monotonic()-start,3))
 (HERE/'verification.json').write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')
def verify():
 r=json.loads((HERE/'verification.json').read_text());rows=fixtures.cases();text,names=fixtures.render(rows);assert r['omega_revision']==PIN and r['input_sha256']==snapshot();assert r['runner_sha256']==sha(RUNNER)and r['compiler_sha256']==sha(COMPILER)
 assert r['positive_count']==r['control_count']==len(rows);assert r['source_sha256']==hashlib.sha256(text.encode()).hexdigest();validate(r['output'],names);assert len(r['constant_proofs'])==2*len(selected())
 for row,pair in zip(selected(),zip(r['constant_proofs'][::2],r['constant_proofs'][1::2])):
  for control,proof in zip([False,True],pair):
   assert proof['case']==row['name']and proof['control']==control and proof['source_sha256']==hashlib.sha256(const_source(row,control).encode()).hexdigest()
   assert ('cannot prove requires contract'in proof['output']and'1 == 0'in proof['output'])if control else'compiled 'in proof['output']
 public=json.loads((HERE/'public-verification.json').read_text());assert len(public['observations'])==12
 print('PASS',len(rows),'checked pairs,2const pairs,12bound public observations; current hashes')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--verify-record',action='store_true');a=p.parse_args()
 if a.verify_record:verify()
 else:run()
