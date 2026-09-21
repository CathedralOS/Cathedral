#!/usr/bin/env python3
"""Execute actual bounded write assembly bodies against independent interval vectors."""
import argparse,hashlib,json,os,re,subprocess,tempfile,time
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT
PIN='eaa7993a23623cd8fabf45350340479c5c9c7879'
RUNNER=Path('/tmp/cathedral-acpi-execution-checked/release/cathedral-acpi-checked-runner')
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def snapshot():
 paths=list(HERE.glob('*.py'))+[HERE/'reference.rs',HERE/'reference.Cargo.lock',HERE/'reference-verification.json',HERE/'cases.json',HERE/'comparison.json']
 paths += list((ROOT/'source/libraries/acpi/field_writes').glob('*.omg'))
 paths += [ROOT/'source/libraries/acpi/field_access'/name for name in ['build.omg','model.omg','geometry.omg','chunks.omg']]
 paths += [ROOT/'source/libraries/acpi/aml'/name for name in ['build.omg','model.omg']]
 paths += [ROOT/'source/libraries/acpi/aml/fields'/name for name in ['build.omg','field_model.omg','flags.omg']]
 paths += [ROOT/'source/libraries/acpi/interpreter'/name for name in ['build.omg','integers.omg','buffer_fields.omg']]
 paths += [ROOT/'tools/ports/acpi/interpreter/execution'/name for name in ['checked_runner.rs','runner.Cargo.lock']]
 return {str(path.relative_to(ROOT)):sha(path)for path in sorted(paths)}
def build_text():
 text='machine build(builder:&mut Build){builder.application("cathedral-field-writes-tests");builder.freestanding=true;'
 for alias,folder in [('writes','field_writes'),('access','field_access'),('fields','aml/fields'),('integers','interpreter'),('aml','aml')]:text+='builder.depend_as("'+alias+'",Source::Path {location:"'+str(ROOT/'source/libraries/acpi'/folder)+'"});'
 return text+'}'
def validate(output,names):
 assert 'CHECKED authored package and dependency bodies;'in output
 actual=re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=',output,re.M)
 assert len(actual)==len(names)
 for (name,expected,observed),selection in zip(actual,names):assert name+'='+expected==selection and expected==observed

def main():
 p=argparse.ArgumentParser();p.add_argument('--runner',type=Path,default=RUNNER);p.add_argument('--case',action='append');p.add_argument('--batch-size',type=int,default=20);p.add_argument('--workers',type=int,default=3);p.add_argument('--record',default='checked-verification.json');a=p.parse_args()
 assert 1<=a.workers<=3 and a.batch_size>0
 rows=fixtures.cases();assert json.loads((HERE/'cases.json').read_text())==rows
 if a.case:rows=[r for r in rows if r['name']in a.case];assert len(rows)==len(a.case)
 inputs=snapshot();binary=sha(a.runner);start=time.monotonic();build_source=build_text()
 def run_batch(selected):
  batch_start=time.monotonic();text,names=fixtures.render(selected)
  assert inputs==snapshot() and binary==sha(a.runner),'inputs changed before batch'
  with tempfile.TemporaryDirectory(prefix='cathedral-field-writes-checked-')as d:
   work=Path(d);(work/'main.omg').write_text(text);(work/'build.omg').write_text(build_source)
   result=subprocess.run([str(a.runner),str(work/'main.omg'),str(work/'build'),*names],capture_output=True,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
   output=result.stdout+result.stderr;assert result.returncode==0,output
  validate(output,names);assert inputs==snapshot() and binary==sha(a.runner),'inputs changed after batch'
  return dict(cases=[r['name']for r in selected],source_sha256=hashlib.sha256(text.encode()).hexdigest(),elapsed_seconds=round(time.monotonic()-batch_start,3),output=output)
 selected_batches=[rows[i:i+a.batch_size]for i in range(0,len(rows),a.batch_size)];batches=[]
 with ThreadPoolExecutor(max_workers=a.workers)as pool:
  for batch in pool.map(run_batch,selected_batches):
   batches.append(batch);print(batch['output'],flush=True)
 assert inputs==snapshot() and binary==sha(a.runner),'inputs changed'
 record=dict(stage='actual checked interpreter bodies; no native or namespace evaluation',omega_revision=PIN,input_sha256=inputs,runner_sha256=binary,runner_path=str(a.runner),scope='selected'if a.case else'full',cases=[r['name']for r in rows],positive_count=len(rows),control_count=len(rows),elapsed_seconds=round(time.monotonic()-start,3),batch_elapsed_seconds_sum=round(sum(b['elapsed_seconds']for b in batches),3),workers=a.workers,build_sha256=hashlib.sha256(build_source.encode()).hexdigest(),batches=batches)
 (HERE/a.record).write_text(json.dumps(record,indent=2,sort_keys=True)+'\n');print('PASS',len(rows),'checked pairs')
if __name__=='__main__':main()
