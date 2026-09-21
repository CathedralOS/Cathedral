#!/usr/bin/env python3
"""Public interpreter witnesses for already-encoded owned-pipeline fixtures."""
import argparse,importlib.util,json,re,subprocess,tempfile
from pathlib import Path
import check as host
HERE=host.HERE;ROOT=host.ROOT
SOURCE=ROOT/'tools/ports/acpi/pipeline/fixtures.py'
spec=importlib.util.spec_from_file_location('pipeline_fixtures',SOURCE)
pipeline=importlib.util.module_from_spec(spec);spec.loader.exec_module(pipeline)
SELECTED=['load_run_add','nested_method','cross_scope_alias','alias_original_rebound','method_redeclaration']
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--write',action='store_true');args=parser.parse_args()
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=host.UP,text=True).strip()==host.PIN
 subprocess.run(['cargo','+nightly-2026-09-04','build','--offline','--locked','--release','--manifest-path',str(HERE/'Cargo.toml'),'--target-dir',str(host.TARGET)],check=True,cwd=ROOT)
 rows={row['name']:row for row in pipeline.cases()};observations={}
 with tempfile.TemporaryDirectory(prefix='cathedral-public-pipeline-')as directory:
  for name in SELECTED:
   row=rows[name];prefix=row['body'].split('let prepared: Prepared =',1)[0]
   size=int(re.search(r'prepare_program\(input,(\d+),',row['body'])[1]);data=bytearray(size)
   for index,value in re.findall(r'input\[(\d+)\] = (\d+);',prefix):data[int(index)]=int(value)
   expected=int(re.search(r'result.value.number == (\d+)',row['check'])[1]);path=Path(directory)/(name+'.aml');path.write_bytes(data)
   run=subprocess.run([str(host.TARGET/'release/cathedral-acpi-public-execution'),str(path),'1',''],capture_output=True,text=True,timeout=5);assert run.returncode==0,run.stderr
   value=dict(line.split('\t',1)for line in run.stdout.replace(str(ROOT)+'/','').splitlines())
   assert value['forbidden_calls']=='0'and value['created_mutexes']=='1'
   observations[name]={'table_hex':data.hex(),'observed':value,'omega_expected_integer':expected,'agrees':value.get('result')=='integer:'+str(expected)}
   print(name,json.dumps(value,sort_keys=True),flush=True)
 paths=[Path(__file__),HERE/'check.py',HERE/'src/main.rs',HERE/'Cargo.toml',HERE/'Cargo.lock',SOURCE,ROOT/'tools/ports/acpi/pipeline/cases.json']
 record={'format':'cathedral-public-aml-pipeline-v1','stage':'actual public Rust load_table/evaluate; no Omega native or hardware execution','upstream_revision':host.PIN,'source_sha256':{str(p.resolve().relative_to(ROOT)):host.sha(p)for p in paths},'binary_sha256':host.sha(host.TARGET/'release/cathedral-acpi-public-execution'),'rows':observations,'agreements':sum(x['agrees']for x in observations.values())}
 path=HERE/'pipeline-observations.json'
 if args.write:path.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==record,'public pipeline observation/source drift'
 print(len(observations),'public pipeline observations;',record['agreements'],'value agreements')
if __name__=='__main__':main()
