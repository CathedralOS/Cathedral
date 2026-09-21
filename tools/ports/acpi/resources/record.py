#!/usr/bin/env python3
"""Verify current evidence hashes, then optionally refresh the release manifest."""
import argparse,collections,hashlib,json,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[3]
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def encode(data):return json.dumps(data,indent=2,sort_keys=True)+'\n'
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--write',action='store_true');a=p.parse_args()
 subprocess.run(['python3',str(HERE/'generate.py'),'--check'],cwd=ROOT,check=True)
 subprocess.run(['python3',str(HERE/'map_inventory.py'),'--check'],cwd=ROOT,check=True)
 subprocess.run(['python3',str(ROOT/'tools/ports/inventory.py'),'check',str(ROOT/'source/libraries/acpi/resources/inventory.json'),'--checkout',str(ROOT/'reference_code/rust-osdev/acpi')],cwd=ROOT,check=True)
 cases=json.loads((HERE/'cases.json').read_text());checked=json.loads((HERE/'checked-verification.json').read_text());const=json.loads((HERE/'constant-verification.json').read_text());observations=json.loads((HERE/'observations.json').read_text())
 assert checked['format']=='cathedral-resource-checked-batches-v1'
 assert checked['scenario_count']==checked['control_count']==len(cases)
 assert set(checked['fixture_sha256'])==set(cases)
 for name,h in checked['fixture_sha256'].items():assert digest(HERE/'cases'/f'{name}.omg')==h,name
 for name,h in checked['source_sha256'].items():assert digest(ROOT/name)==h,name
 closure=hashlib.sha256()
 for name in sorted(checked['source_sha256']):closure.update(name.encode());closure.update(b'\0');closure.update((ROOT/name).read_bytes())
 assert const['selected_source_sha256']==closure.hexdigest()
 recorded_subset_hash=closure.hexdigest();recorded_sources=dict(checked['source_sha256'])
 audit=json.loads((HERE/'source-closure-audit.json').read_text());complete_sources=dict(recorded_sources)
 for name,update in audit['harness_updates'].items():assert digest(ROOT/name)==update['future_canonical_harness_sha256']
 for name,h in audit['supplemental_source_sha256'].items():
  assert digest(ROOT/name)==h
  assert hashlib.sha256(subprocess.check_output(['git','show','4b95485:'+name],cwd=ROOT)).hexdigest()==h
  assert audit['git_head_header_sha256']==audit['first_checkpoint_header_sha256']==h
  complete_sources[name]=h
 closure=hashlib.sha256()
 for name in sorted(complete_sources):closure.update(name.encode());closure.update(b'\0');closure.update((ROOT/name).read_bytes())
 assert const['scenario_count']==const['control_count']==len(const['results'])
 for result in const['results']:
  assert digest(HERE/'cases'/f'{result["name"]}.omg')==result['fixture_sha256']
  assert 'compiled 'in result['positive_output']
  assert 'cannot prove requires contract'in result['negative_output']and'1 == 0'in result['negative_output']
 union=set()
 for batch in checked['batches']:
  path=ROOT/batch['path'];assert digest(path)==batch['sha256'];r=json.loads(path.read_text());assert r['source_sha256']==checked['source_sha256'];assert r['runner_sha256']==checked['runner_sha256'];assert r['omega_revision']==checked['omega_revision']
  assert digest(HERE/'checked_runner.rs')==r['runner_source_sha256'];assert digest(HERE/'runner.Cargo.lock')==r['cargo_lock_sha256']
  assert r['scenario_count']==r['control_count']==len(r['fixture_sha256'])==batch['cases']
  for name,h in r['fixture_sha256'].items():
   assert name not in union;union.add(name);assert checked['fixture_sha256'][name]==h
   for suffix,value in [('positive',0),('control',1)]:assert f'PASS ResourceSuite::{name.replace("-","_")}_{suffix} expected={value} observed={value} error=None' in r['output']
 assert union==set(cases)
 counts=collections.Counter(o['result']for o in observations.values());agreements=sum(not m['template']and m['expected']['outcome']=='Success'and m['expected'].get('norm')is not None for m in cases.values())
 source=ROOT/'reference_code/rust-osdev/acpi/src/aml/resource.rs';revision='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
 refs=[source]+[HERE/n for n in ['Cargo.lock','Cargo.toml','corpus.py','generate.py','observations.json','probe.rs.in','src/main.rs']]
 reference={'kind':'actual public Rust resource_descriptor_list on synthetic Object::Buffer inputs; no private mirror','command':'python3 tools/ports/acpi/resources/generate.py --check','pin':revision,'upstream_url':'https://github.com/rust-osdev/acpi','rustc':subprocess.check_output(['rustc','+nightly-2026-09-04','--version'],text=True).strip(),'primary_spec':'https://uefi.org/specs/ACPI/6.6/06_Device_Configuration.html#connection-descriptors','observations':len(observations),'normalized_supported_agreements':agreements,'observed_ok':counts['ok'],'observed_error':counts['error'],'observed_panic':counts['panic'],'sha256':{str(f.relative_to(ROOT)):digest(f)for f in refs},'policy':'Reference catch_unwind records pin panics; bounded Omega parser requires explicit outcomes. Current revision, reserved flags, source profile and offset differences are documented in PORT.md.'}
 refpath=HERE/'reference-verification.json'
 if a.write:refpath.write_text(encode(reference))
 else:assert refpath.read_text()==encode(reference),'stale reference record'
 inventory=json.loads((ROOT/'source/libraries/acpi/resources/inventory.json').read_text());anchor_counts=collections.Counter(row['disposition']for f in inventory['files'].values()for row in f['symbols'].values())
 artifacts=[]
 for base in [ROOT/'source/libraries/acpi/resources',HERE]:
  for f in base.rglob('*'):
   if f.is_file()and not any(part in ['target','__pycache__','history']for part in f.relative_to(base).parts)and f!=HERE/'verification.json':artifacts.append(f)
 manifest={'format':'cathedral-acpi-resources-connections-v2','status':'tested bounded pin-supported descriptor families including GPIO/I2C; other defined families explicitly Unsupported','upstream_revision':revision,'upstream_resource_sha256':digest(source),'omega_revision':checked['omega_revision'],'omega_compiler_sha256':const['omega_compiler_sha256'],'checked_runner_sha256':checked['runner_sha256'],'source_hash_algorithm':'SHA-256 over sorted repository-relative path + NUL + file bytes','selected_source_sha256':recorded_subset_hash,'source_closure_sha256':closure.hexdigest(),'source_files_sha256':complete_sources,'original_run_source_files_sha256':recorded_sources,'source_closure_audit':'source-closure-audit.json; original recorded subsets preserved, unchanged transitive header explicitly supplemented','inventory':dict(anchor_counts,anchors=sum(anchor_counts.values()),files=len(inventory['files'])),'validation':{'actual_public_rust':reference,'checked_interpreter':{'record':'checked-verification.json','positives':len(cases),'changed_body_controls':len(cases),'batches':len(checked['batches']),'elapsed_seconds':checked['elapsed_seconds'],'command':checked['command']},'constant_evaluation':{'record':'constant-verification.json','positives':const['scenario_count'],'changed_body_controls':const['control_count'],'command':const['command']}},'boundaries':['4096 initialized bytes, not an ACPI maximum','all actual pin-supported families represented; other defined families retain Unsupported envelope','strict documented EndTag, connection/source/revision profile','spans/raw numeric facts do not confer GPIO/I2C/DMA/IO/mapping or namespace authority','no native ABI or firmware execution'],'historical_record':'history/first-slice-4b95485/verification.json applies only to first checkpoint','artifacts_sha256':{str(f.relative_to(ROOT)):digest(f)for f in sorted(artifacts)}}
 path=HERE/'verification.json'
 if a.write:path.write_text(encode(manifest))
 else:assert path.read_text()==encode(manifest),'stale release manifest'
 print(f'PASS release hashes: {len(cases)} checked pairs, {const["scenario_count"]} constant pairs, {len(observations)} public Rust observations.')
if __name__=='__main__':main()
