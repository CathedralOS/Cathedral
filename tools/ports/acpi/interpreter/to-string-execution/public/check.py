#!/usr/bin/env python3
"""Pinned public ToString observations using the unchanged inert service-trap harness."""
import argparse,hashlib,json,os,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;sys.path.insert(0,str(HERE.parent));import fixtures
ROOT=fixtures.ROOT;PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5';PROBE=ROOT/'tools/ports/acpi/aml-public-execution'
DEFAULT_TOOLCHAIN=Path('/Users/zcanann/.rustup/toolchains/nightly-2026-09-04-aarch64-apple-darwin/bin')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def snapshot():
 paths=[Path(__file__),HERE.parent/'fixtures.py',HERE.parent/'execution-cases.json',HERE.parent/'toolchain.json',PROBE/'src/main.rs',PROBE/'Cargo.toml',PROBE/'Cargo.lock',HERE.parent.parent/'mid-execution/fixtures.py',fixtures.NAMED/'fixtures.py',fixtures.GENERIC/'fixtures.py',fixtures.GENERIC/'decoder_fixtures.py',HERE.parent.parent/'execution/fixtures.py',HERE.parent.parent/'to-integer-execution/fixtures.py',HERE.parent.parent.parent/'pipeline/fixtures.py']
 return {str(p.relative_to(ROOT)):sha(p)for p in paths}
def selection():
 selected=[];omitted={}
 for row in fixtures.execution_cases():
  if row['setup']:omitted[row['name']]='Direct canonical store edit has no equivalent public API fixture.'
  elif row['name']=='debug_target':omitted[row['name']]='Debug callback lies outside this service-free observation profile.'
  else:selected.append(row)
 return selected,omitted
def expected(row):
 return {'result':('integer:'+str(row['number'])if row['kind']==1 else ('buffer:'if row['kind']==2 else 'string:')+row['bytes']),**{'after:'+key:'integer:'+str(value) for key,value in row['after'].items()}}
def verify(record,require_binary):
 assert record['source_sha256']==snapshot() and record['source_unchanged'] and record['upstream_revision']==PIN
 rows,omitted=selection();assert omitted==record['omitted'];assert list(record['rows'])==sorted(r['name']for r in rows)
 upstream=Path(record['upstream_source']);assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=upstream,text=True).strip()==PIN
 assert all(sha(upstream/path)==value for path,value in record['upstream_sha256'].items())
 for row in rows:
  observation=record['rows'][row['name']];assert observation['table_hex']==row['table'] and observation['bits']==row['bits'] and observation['omega_outcome']==row['error'] and observation['omega_expected']==expected(row)
  values=dict(line.split('\t',1)for line in observation['stdout'].splitlines());assert values==observation['observed'] and observation['exit_code']==0
  assert values['forbidden_calls']=='0' and values['created_mutexes']=='1'
  agrees=row['error']=='Success' and values.get('load')=='ok' and all(values.get(key)==value for key,value in expected(row).items())
  assert agrees==observation['agrees_on_value_and_state']
 assert record['agreements']==sum(v['agrees_on_value_and_state']for v in record['rows'].values())
 if require_binary:assert sha(Path(record['binary']))==record['binary_sha256']
 print('PASS',len(rows),'public observations;',record['agreements'],'value/state agreements;',len(omitted),'explicit omissions')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--acpi-source',type=Path,default=ROOT/'reference_code/rust-osdev/acpi');p.add_argument('--toolchain',type=Path,default=DEFAULT_TOOLCHAIN);p.add_argument('--record',type=Path,default=HERE/'verification.json');p.add_argument('--verify',action='store_true');p.add_argument('--require-binary',action='store_true');a=p.parse_args()
 if a.verify:return verify(json.loads(a.record.read_text()),a.require_binary)
 upstream=a.acpi_source.resolve();assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=upstream,text=True).strip()==PIN
 assert not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=upstream,text=True).strip()
 bound=snapshot();original={str(p.relative_to(upstream)):sha(p)for p in sorted((upstream/'src').rglob('*.rs'))};original['Cargo.toml']=sha(upstream/'Cargo.toml')
 manifest=(PROBE/'Cargo.toml').read_text().replace('../../../../reference_code/rust-osdev/acpi',str(upstream));cargo=a.toolchain/'cargo';rustc=a.toolchain/'rustc';toolenv=dict(os.environ,RUSTC=str(rustc));observations={};rows,omitted=selection()
 with tempfile.TemporaryDirectory(prefix='cathedral-to-string-public-')as d:
  work=Path(d);(work/'src').mkdir();(work/'src/main.rs').write_bytes((PROBE/'src/main.rs').read_bytes());(work/'Cargo.toml').write_text(manifest);(work/'Cargo.lock').write_bytes((PROBE/'Cargo.lock').read_bytes())
  target=Path('/tmp/cathedral-to-string-public-target');command=[str(cargo),'build','--offline','--locked','--release','--jobs','2','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)]
  built=subprocess.run(command,env=toolenv,cwd=upstream,capture_output=True,text=True);assert built.returncode==0,built.stdout+built.stderr
  binary=target/'release/cathedral-acpi-public-execution';binary_hash=sha(binary)
  for row in rows:
   aml=work/(row['name']+'.aml');aml.write_bytes(bytes.fromhex(row['table']));run=subprocess.run([str(binary),str(aml),'1'if row['bits']==32 else'2','',*row['after']],capture_output=True,text=True,timeout=5,env=dict(os.environ,CATHEDRAL_GENERIC_DESCRIBE='1'))
   assert run.returncode==0,(row['name'],run.stderr);values=dict(line.split('\t',1)for line in run.stdout.splitlines());assert values['forbidden_calls']=='0' and values['created_mutexes']=='1',(row['name'],values)
   agrees=row['error']=='Success' and values.get('load')=='ok' and all(values.get(key)==value for key,value in expected(row).items())
   observations[row['name']]=dict(table_hex=row['table'],bits=row['bits'],omega_outcome=row['error'],omega_expected=expected(row),observed=values,stdout=run.stdout,stderr=run.stderr,exit_code=run.returncode,agrees_on_value_and_state=agrees)
  assert binary_hash==sha(binary)
 assert bound==snapshot() and all(sha(upstream/path)==value for path,value in original.items())
 record=dict(stage='Pinned public Interpreter load/evaluate; no native Omega or hardware claim',upstream_revision=PIN,upstream_source=str(upstream),upstream_sha256=original,source_sha256=bound,source_unchanged=True,binary=str(binary),binary_sha256=binary_hash,build_source=manifest,build_command=command,build_output=built.stdout+built.stderr,rustc=subprocess.check_output([str(rustc),'-Vv'],text=True),cargo=subprocess.check_output([str(cargo),'-V'],text=True),rows=observations,omitted=omitted,agreements=sum(v['agrees_on_value_and_state']for v in observations.values()))
 a.record.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n');verify(json.loads(a.record.read_text()),True)
if __name__=='__main__':main()
