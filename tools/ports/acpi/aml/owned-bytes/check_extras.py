#!/usr/bin/env python3
import argparse,hashlib,json,os,subprocess,tempfile
from pathlib import Path
import extras,fixtures,check
HERE=fixtures.HERE;ROOT=fixtures.ROOT
p=argparse.ArgumentParser();p.add_argument('--const',dest='constant',action='store_true');a=p.parse_args();before=check.snapshot();source_files={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for base in ['aml','interpreter','pipeline']for p in(ROOT/'source/libraries/acpi'/base).rglob('*.omg')};rows=extras.cases();records=[]
for name in ['extras.py','check_extras.py']:source_files[str((HERE/name).relative_to(ROOT))]=hashlib.sha256((HERE/name).read_bytes()).hexdigest()
with tempfile.TemporaryDirectory(prefix='cathedral-owned-byte-extras-')as directory:
 work=Path(directory);(work/'build.omg').write_text('machine build(builder:&mut Build){builder.application("owned-byte-extras");builder.freestanding=true;builder.depend_as("aml",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/aml')+'"});builder.depend_as("helpers",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/interpreter')+'"});builder.depend_as("pipeline",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/pipeline')+'"});}\n')
 if a.constant:
  rows=[r for r in rows if r['name']in ['length_buffer','string_index_write','field_revalidates_shrunken_backing']]
  for row in rows:
   for control in [False,True]:
    text=fixtures.IMPORTS+extras.render(row,control)+'''const TEST_RESULT:i32=test_result();
machine require_ok(value:i32) requires value==0;{}
data Main{}
machine Main::main(&mut self){require_ok(TEST_RESULT);}
''';(work/'main.omg').write_text(text);run=subprocess.run(['/tmp/cathedral-omega-eaa7993/release/omega','--check',str(work/'main.omg')],capture_output=True,text=True);output=run.stdout+run.stderr
    good=run.returncode!=0 and'cannot prove requires contract'in output and'1 == 0'in output if control else run.returncode==0
    assert good,(row['name'],control,output);records.append(dict(case=row['name'],control=control,source_sha256=hashlib.sha256(text.encode()).hexdigest(),output=output));print('PASS const',row['name'],control,flush=True)
 else:
  text=extras.IMPORTS+extras.HELPERS+'data Extras{}\n';selections=[]
  for row in rows:
   for control in [False,True]:
    name='Extras::'+row['name']+('_control'if control else'_positive');text+=extras.render(row,control,name);selections.append(name+'='+str(int(control)))
  (work/'main.omg').write_text(text);run=subprocess.run([str(check.RUNNER),str(work/'main.omg'),str(work/'build'),*selections],capture_output=True,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'));print(run.stdout+run.stderr);assert run.returncode==0;records.append(dict(source_sha256=hashlib.sha256(text.encode()).hexdigest(),output=run.stdout))
assert before==check.snapshot();assert all(hashlib.sha256((ROOT/p).read_bytes()).hexdigest()==h for p,h in source_files.items())
(HERE/('const-verification.json'if a.constant else'extras-verification.json')).write_text(json.dumps(dict(stage='constant evaluation and checked requires'if a.constant else'checked-interpreter actual bodies; returned ID outlives host helper activation, not generic AML Return execution',omega_revision='eaa7993a23623cd8fabf45350340479c5c9c7879',runner_sha256=hashlib.sha256((Path('/tmp/cathedral-omega-eaa7993/release/omega')if a.constant else check.RUNNER).read_bytes()).hexdigest(),cases=[r['name']for r in rows],scenario_count=len(rows),control_count=len(rows),source_sha256={**before,**source_files},records=records),indent=2,sort_keys=True)+'\n')
