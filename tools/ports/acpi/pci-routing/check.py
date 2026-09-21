#!/usr/bin/env python3
"""Actual pinned public _PRT decoding/routing with forbidden-service traps."""
import argparse,collections,hashlib,json,re,subprocess,tempfile
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=HERE.parents[3];UP=ROOT/'reference_code/rust-osdev/acpi'
PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5';TARGET=Path('/tmp/cathedral-acpi-pci-routing')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def inputs():
 return {str(p.relative_to(ROOT)):sha(p)for p in sorted(HERE.rglob('*'))if p.is_file()and not any(s in p.parts for s in ['target','__pycache__'])and p.name!='observations.json'}
def irq(text):
 if not text.startswith('ok:'):return dict(outcome=text)
 match=re.fullmatch(r'ok:IrqDescriptor \{ is_consumer: (true|false), trigger: (Edge|Level), polarity: (ActiveHigh|ActiveLow), is_shared: (true|false), is_wake_capable: (true|false), irqs: \[([^\]]*)\] \}',text)
 assert match,text
 consumer,trigger,polarity,shared,wake,numbers=match.groups()
 return dict(outcome='success',is_consumer=consumer=='true',trigger=trigger,polarity=polarity,is_shared=shared=='true',is_wake_capable=wake=='true',irqs=[int(x.strip())for x in numbers.split(',')if x.strip()])
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--write',action='store_true');args=parser.parse_args()
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()==PIN
 assert not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=UP,text=True).strip()
 subprocess.run(['python3',str(HERE/'fixtures.py'),'--check'],check=True,cwd=ROOT)
 before=inputs()
 subprocess.run(['cargo','+nightly-2026-09-04','build','--offline','--locked','--release','--manifest-path',str(HERE/'Cargo.toml'),'--target-dir',str(TARGET)],check=True,cwd=ROOT)
 binary=TARGET/'release/cathedral-acpi-pci-routing';observed={};counts=collections.Counter()
 with tempfile.TemporaryDirectory(prefix='cathedral-public-prt-')as directory:
  for row in fixtures.cases():
   data=bytes.fromhex(row['aml_hex']);path=Path(directory)/(row['name']+'.aml');path.write_bytes(data)
   command=[str(binary),str(path),row['prt_path'],*(','.join(map(str,q))for q in row['queries'])]
   run=subprocess.run(command,capture_output=True,text=True,timeout=5)
   assert run.returncode==0,(row['name'],run.returncode,run.stderr)
   values=dict(line.split('\t',1)for line in run.stdout.replace(str(ROOT)+'/','').splitlines())
   assert values.get('forbidden_calls')=='0',(row['name'],'attempted host service',values)
   assert values.get('created_mutexes')=='1',(row['name'],'unexpected constructor identity',values)
   routes={key:irq(value)for key,value in values.items()if key.startswith('route:')}
   decode=values.get('decode','not reached')
   classification='not_reached'if 'decode'not in values else'success'if decode.startswith('ok:')else'panic'if decode=='panic'else'error'
   counts['load_'+('success'if values.get('load')=='ok'else'panic'if values.get('load_or_probe')=='panic'else'error')]+=1
   counts['decode_'+classification]+=1
   for result in routes.values():counts['route_'+('success'if result['outcome']=='success'else'panic'if result['outcome']=='panic'else'error')]+=1
   observed[row['name']]=dict(table_sha256=sha(path),observed=values,normalized_routes=routes,strict_expectation=row['strict_expectation'])
   print(row['name'],classification,json.dumps(routes,sort_keys=True),flush=True)
 assert inputs()==before,'harness changed during run'
 # Concrete canaries are independent of the golden-output file.
 assert observed['direct_all_pins']['normalized_routes']['route:0']['irqs']==[40]
 assert observed['first_duplicate_wildcard']['normalized_routes']['route:0']['irqs']==[41]
 assert observed['gsi_4294967296']['normalized_routes']['route:0']['irqs']==[0]
 assert observed['direct_length_0']['observed']['decode']=='panic'
 assert observed['link_index_string']['normalized_routes']['route:0']['irqs']==[10]
 assert observed['captured_source_scope_name']['normalized_routes']['route:0']['irqs']==[11]
 assert observed['captured_source_scope_string']['normalized_routes']['route:0']['irqs']==[10]
 assert observed['name_parent']['normalized_routes']['route:0']['outcome'].startswith('error:NameNotAbsolute')
 assert observed['vendor_then_irq_index0']['normalized_routes']['route:0']['irqs']==[10]
 assert observed['two_irqs_index1']['normalized_routes']['route:0']['outcome']=='error:UnexpectedResourceType'
 record=dict(format='cathedral-public-prt-observations-v1',stage='actual public Rust Interpreter::new/load_table and PciRoutingTable::from_prt_path/route; no Omega execution or hardware',upstream_revision=PIN,rustc=subprocess.check_output(['rustc','+nightly-2026-09-04','--version'],text=True).strip(),binary_sha256=sha(binary),source_sha256=before,upstream_source_sha256={str(p.relative_to(UP)):sha(p)for p in sorted((UP/'src').rglob('*.rs'))},counts=dict(counts),rows=observed)
 path=HERE/'observations.json'
 if args.write:path.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==record,'public _PRT observation/source drift'
 print(len(observed),'public _PRT fixtures:',json.dumps(dict(counts),sort_keys=True))
if __name__=='__main__':main()
