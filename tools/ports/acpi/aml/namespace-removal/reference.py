import argparse,hashlib,json,subprocess,tempfile
from pathlib import Path
import fixtures
HERE=fixtures.HERE;CANONICAL=fixtures.ROOT;UP=CANONICAL/'reference_code/rust-osdev/acpi';PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');a=p.parse_args();assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()==PIN
 upstream={str(p.relative_to(UP)):sha(p)for p in sorted((UP/'src').rglob('*.rs'))}
 for name in ['Cargo.toml','LICENCE-MIT','LICENCE-APACHE']:upstream[name]=sha(UP/name)
 for name,h in upstream.items():assert hashlib.sha256(subprocess.check_output(['git','show',PIN+':'+name],cwd=UP)).hexdigest()==h
 with tempfile.TemporaryDirectory(prefix='cathedral-namespace-removal-public-')as d:
  work=Path(d);(work/'Cargo.toml').write_text('[package]\nname="cathedral-namespace-removal-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={path="'+str(UP)+'"}\n[[bin]]\nname="reference"\npath="'+str(HERE/'reference.rs')+'"\n');(work/'Cargo.lock').write_bytes((HERE/'reference.Cargo.lock').read_bytes());subprocess.run(['cargo','+nightly-2026-09-04','build','--offline','--locked','--release','--manifest-path',str(work/'Cargo.toml'),'--target-dir','/tmp/cathedral-namespace-removal-reference'],check=True)
 binary=Path('/tmp/cathedral-namespace-removal-reference/release/reference');observations=[]
 for row in fixtures.cases()[:11]+[next(x for x in fixtures.cases()if x['name']=='relative_path')]:
  target=(''if row['name']=='relative_path'else'\\')+row['target'];out=subprocess.check_output([str(binary),row['name'],target],text=True);obs=dict(line.split('\t',1)for line in out.splitlines());assert(obs['result']=='ok')==(row['outcome']=='Success')
  before_levels=set(obs['before_levels'].split(','));before_values=set(obs['before_values'].split(','));after_levels=set(obs['after_levels'].split(','));after_values=set(obs['after_values'].split(','))
  if 'kept'in row:
   expected_levels={x for x in before_levels if x!=target and not x.startswith(target+'.')};expected_values={x for x in before_values if not x.split(':')[0].startswith(target+'.')}
  else:expected_levels=before_levels;expected_values=before_values
  assert after_levels==expected_levels and after_values==expected_values,obs
  assert obs['same_path_object']==obs['alias_survives']=='true'
  observations.append(dict(case=row['name'],target=target,observed=obs))
 value=dict(pin=PIN,upstream_sha256=upstream,source_sha256={p.name:sha(p)for p in [HERE/'reference.py',HERE/'reference.rs',HERE/'reference.Cargo.lock',HERE/'fixtures.py']},binary_sha256=sha(binary),observations=observations)
 path=HERE/'public-verification.json'
 if a.write:path.write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==value
 print('PASS',len(observations),'actual public Namespace removal observations')
if __name__=='__main__':main()
