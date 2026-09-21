#!/usr/bin/env python3
"""Observe finite existing bytecode fixtures through actual pinned public APIs."""
import argparse,hashlib,importlib.util,json,subprocess,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[3]
CASES=ROOT/'tools/ports/acpi/interpreter/execution/cases.json'
spec=importlib.util.spec_from_file_location('execution_fixtures',CASES.with_name('fixtures.py'))
fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5';UP=ROOT/'reference_code/rust-osdev/acpi'
TARGET=Path('/tmp/cathedral-acpi-public-execution')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def omission(row):
 if 'profile_patch'in row:return 'Cathedral storage/span/capacity admission profile, not encoded AML'
 if 'omega_setup'in row:return 'Direct Cathedral namespace/definition edits require a separate public-API fixture'
 if row['name']in ['loop_budget','recursive_depth','store_then_limit']:return 'Unbounded pinned loop/recursion has no corresponding Cathedral fuel/depth mechanism'
 if row['error']in ['UnresolvedService','UnresolvedSynchronization','UnresolvedRegion']:return 'Requires explicitly absent host service or region model'
 if row['name']in ['zero_budget','over_budget']:return 'Caller-supplied Cathedral work admission is not an upstream API parameter'
 return None
def table(row):
 data=bytearray()
 for name,value in row['globals'].items():
  assert isinstance(value,int);data+=b'\x08'+fixture.name(name)+fixture.integer(value)
 for name,method in {'MAIN':{'flags':row['flags'],'bytes':row['main']},**row['methods']}.items():
  data+=fixture.pkg(0x14,fixture.name(name)+bytes([method['flags']])+bytes.fromhex(method['bytes']))
 for alias,target in row['aliases'].items():data+=b'\x06'+fixture.name(target)+fixture.name(alias)
 return bytes(data)
def expected(row):
 result='integer:'+str(row['expected'])if row['expected']is not None else'uninitialized'
 return dict(result=result,**{'after:'+n:'integer:'+str(v)for n,v in row['after'].items()})
def main():
 p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');a=p.parse_args()
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()==PIN
 assert not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=UP,text=True).strip()
 assert json.loads(CASES.read_text())['cases']==fixture.cases()
 subprocess.run(['cargo','+nightly-2026-09-04','build','--offline','--locked','--release','--manifest-path',str(HERE/'Cargo.toml'),'--target-dir',str(TARGET)],check=True,cwd=ROOT)
 binary=TARGET/'release/cathedral-acpi-public-execution';observed={};omitted={};agreements=[];differences=[]
 with tempfile.TemporaryDirectory(prefix='cathedral-public-aml-')as directory:
  for row in fixture.cases():
   reason=omission(row)
   if reason:omitted[row['name']]=reason;continue
   path=Path(directory)/(row['name']+'.aml');path.write_bytes(table(row))
   command=[str(binary),str(path),'1'if row['bits']==32 else'2',','.join(map(str,row['args'])),*row['after']]
   try:
    run=subprocess.run(command,capture_output=True,text=True,timeout=5)
    assert run.returncode==0,(row['name'],run.returncode,run.stderr)
    values=dict(line.split('\t',1)for line in run.stdout.replace(str(ROOT)+'/', '').splitlines())
    assert values['forbidden_calls']=='0',(row['name'],'attempted host service')
    assert values['created_mutexes']=='1',(row['name'],'unexpected synchronization construction')
   except subprocess.TimeoutExpired:values={'timeout':'5 seconds; process killed, no completed observation'}
   observed[row['name']]={'table_hex':path.read_bytes().hex(),'table_sha256':sha(path),'observed':values,'omega_expected':{'outcome':row['error'],**expected(row)}}
   equal=row['error']=='Success'and values.get('load')=='ok'and all(values.get(k)==v for k,v in expected(row).items())
   (agreements if equal else differences).append(row['name'])
   print(row['name'],json.dumps(values,sort_keys=True),flush=True)
 sources=[HERE/'Cargo.toml',HERE/'Cargo.lock',HERE/'src/main.rs',HERE/'check.py',CASES,CASES.with_name('fixtures.py')]
 record={'format':'cathedral-public-aml-execution-v1','stage':'actual pinned public Rust Interpreter::new/load_table/evaluate on host; no Omega native or hardware execution','upstream_revision':PIN,'rustc':subprocess.check_output(['rustc','+nightly-2026-09-04','--version'],text=True).strip(),'binary_sha256':sha(binary),'source_sha256':{str(p.relative_to(ROOT)):sha(p)for p in sources},'upstream_source_sha256':{str(p.relative_to(UP)):sha(p)for p in sorted((UP/'src').rglob('*.rs'))},'rows':observed,'successful_value_agreements':agreements,'different_or_error_observations':differences,'omitted':omitted}
 path=HERE/'observations.json'
 if a.write:path.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==record,'public observation/source drift'
 print(len(observed),'public observations;',len(agreements),'successful value/state agreements;',len(differences),'differences/errors;',len(omitted),'explicitly omitted')
if __name__=='__main__':main()
