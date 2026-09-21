#!/usr/bin/env python3
"""Execute actual bounded text bodies; record exact immutable source closure."""
import argparse,hashlib,json,os,re,subprocess,tempfile,time
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT
PIN='eaa7993a23623cd8fabf45350340479c5c9c7879'
RUNNER=Path('/tmp/cathedral-acpi-execution-checked/release/cathedral-acpi-checked-runner')
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def snapshot():
 paths=list(HERE.glob('*.py'))+[HERE/'reference.rs',HERE/'reference.Cargo.lock',HERE/'reference-verification.json',HERE/'cases.json',HERE/'comparison.json']
 paths += [ROOT/'source/libraries/acpi/aml'/name for name in ['build.omg','model.omg','names.omg','bytes.omg','name_text.omg']]
 paths += [ROOT/'source/libraries/acpi/interpreter/build.omg']
 paths += [ROOT/'tools/ports/acpi/interpreter/execution'/name for name in ['checked_runner.rs','runner.Cargo.lock']]
 return {str(path.relative_to(ROOT)):sha(path)for path in sorted(paths)}
def build_text():return 'machine build(builder:&mut Build){builder.application("cathedral-name-text-tests");builder.freestanding=true;builder.depend_as("aml",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/aml')+'"});}'
def validate(output,names):
 assert 'CHECKED authored package and dependency bodies;'in output
 actual=re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=',output,re.M)
 assert len(actual)==len(names)
 for (name,expected,observed),selection in zip(actual,names):assert name+'='+expected==selection and expected==observed

def main():
 p=argparse.ArgumentParser();p.add_argument('--runner',type=Path,default=RUNNER);p.add_argument('--case',action='append');p.add_argument('--record',default='checked-verification.json');a=p.parse_args()
 rows=fixtures.cases();assert json.loads((HERE/'cases.json').read_text())==rows
 if a.case:rows=[r for r in rows if r['name']in a.case];assert len(rows)==len(a.case)
 text,names=fixtures.render(rows);inputs=snapshot();binary=sha(a.runner);start=time.monotonic()
 with tempfile.TemporaryDirectory(prefix='cathedral-name-text-checked-')as d:
  work=Path(d);(work/'main.omg').write_text(text);(work/'build.omg').write_text(build_text())
  result=subprocess.run([str(a.runner),str(work/'main.omg'),str(work/'build'),*names],capture_output=True,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
  output=result.stdout+result.stderr;print(output,flush=True);assert result.returncode==0,output
 validate(output,names);assert inputs==snapshot() and binary==sha(a.runner),'inputs changed'
 record=dict(stage='actual checked interpreter bodies; no native or namespace evaluation',omega_revision=PIN,input_sha256=inputs,runner_sha256=binary,runner_path=str(a.runner),scope='selected'if a.case else'full',cases=[r['name']for r in rows],positive_count=len(rows),control_count=len(rows),source_sha256=hashlib.sha256(text.encode()).hexdigest(),elapsed_seconds=round(time.monotonic()-start,3),output=output)
 (HERE/a.record).write_text(json.dumps(record,indent=2,sort_keys=True)+'\n');print('PASS',len(rows),'checked pairs')
if __name__=='__main__':main()
