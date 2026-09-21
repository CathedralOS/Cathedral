#!/usr/bin/env python3
"""Retain actual public generic AML results; these are not Omega test passes."""
import argparse,json,os,subprocess,tempfile
from pathlib import Path
import check as host
import generic_fixtures

def main():
 parser=argparse.ArgumentParser();parser.add_argument('--write',action='store_true');args=parser.parse_args()
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=host.UP,text=True).strip()==host.PIN
 assert not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=host.UP,text=True).strip()
 subprocess.run(['cargo','+nightly-2026-09-04','build','--offline','--locked','--release','--manifest-path',str(host.HERE/'Cargo.toml'),'--target-dir',str(host.TARGET)],check=True,cwd=host.ROOT)
 binary=host.TARGET/'release/cathedral-acpi-public-execution';rows={}
 with tempfile.TemporaryDirectory(prefix='cathedral-public-generic-')as directory:
  for row in generic_fixtures.cases():
   path=Path(directory)/(row['name']+'.aml');path.write_bytes(bytes.fromhex(row['table_hex']))
   run=subprocess.run([str(binary),str(path),str(row['revision']),'',*row['after']],env={**os.environ,'CATHEDRAL_GENERIC_DESCRIBE':'1'},capture_output=True,text=True,timeout=5)
   assert run.returncode==0,(row['name'],run.stderr)
   observed=dict(line.split('\t',1)for line in run.stdout.replace(str(host.ROOT)+'/','').splitlines())
   assert observed['forbidden_calls']=='0' and observed['created_mutexes']=='1',(row['name'],observed)
   assert 'observation-limit' not in str(observed),(row['name'],observed)
   rows[row['name']]={**row,'observed':observed}
   print(row['name'],json.dumps(observed,sort_keys=True),flush=True)
 sources=[Path(__file__),host.HERE/'generic_fixtures.py',host.HERE/'check.py',host.HERE/'src/main.rs',host.HERE/'Cargo.toml',host.HERE/'Cargo.lock',host.CASES.with_name('fixtures.py')]
 record={'format':'cathedral-public-generic-aml-v1','stage':'actual public pinned Rust loader/evaluator; Omega integration and semantic comparison pending','upstream_revision':host.PIN,'rustc':subprocess.check_output(['rustc','+nightly-2026-09-04','--version'],text=True).strip(),'binary_sha256':host.sha(binary),'input_sha256':{str(p.resolve().relative_to(host.ROOT)):host.sha(p)for p in sources},'upstream_source_sha256':{str(p.relative_to(host.UP)):host.sha(p)for p in sorted((host.UP/'src').rglob('*.rs'))},'rows':rows}
 path=host.HERE/'generic-observations.json'
 if args.write:path.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==record,'generic public fixture/source/output drift'
 print(len(rows),'actual public generic observations; no Omega result claimed')
if __name__=='__main__':main()
