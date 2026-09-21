#!/usr/bin/env python3
"""Actual public immutable WrappedObject probes; never forge ObjectToken."""
import argparse,hashlib,json,shutil,subprocess,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];UP=ROOT/'reference_code/rust-osdev/acpi';PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args()
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()==PIN
 hashes={}
 for path in ['src/aml/object.rs','src/aml/mod.rs','Cargo.toml','LICENCE-MIT','LICENCE-APACHE']:
  expected=subprocess.check_output(['git','show',PIN+':'+path],cwd=UP);assert expected==(UP/path).read_bytes();hashes[path]=hashlib.sha256(expected).hexdigest()
 with tempfile.TemporaryDirectory(prefix='cathedral-object-references-rust-')as directory:
  work=Path(directory);(work/'Cargo.toml').write_text(f'[package]\nname="cathedral-acpi-object-references"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={{path="{UP}"}}\n[[bin]]\nname="reference"\npath="{HERE}/reference.rs"\n');cargo=shutil.which('mbx')or'cargo';lock=HERE/'reference.Cargo.lock'
  if lock.exists():(work/'Cargo.lock').write_bytes(lock.read_bytes())
  else:
   assert not a.check,'Missing pinned harness lock'
   subprocess.run([cargo,'generate-lockfile','--offline','--manifest-path',str(work/'Cargo.toml')],cwd=ROOT.parent/'Omega',check=True);lock.write_bytes((work/'Cargo.lock').read_bytes())
  run=subprocess.run([cargo,'run','--offline','--locked','--release','--manifest-path',str(work/'Cargo.toml'),'--target-dir','/tmp/cathedral-object-references-rust','--bin','reference'],cwd=ROOT.parent/'Omega',capture_output=True,text=True);assert run.returncode==0,run.stderr
 rows=[json.loads(line)for line in run.stdout.splitlines()];record=dict(stage='actual host Rust public unwrap and immutable clone/identity observations; no unsafe token, mutable object access, AML opcode or hardware execution',upstream_revision=PIN,upstream_sha256=hashes,rustc=subprocess.check_output(['rustc','--version'],cwd=ROOT.parent/'Omega',text=True).strip(),source_sha256={p.name:hashlib.sha256(p.read_bytes()).hexdigest()for p in [HERE/'reference.py',HERE/'reference.rs',lock]},observations=rows)
 text=json.dumps(record,indent=2,sort_keys=True)+'\n';out=HERE/'reference-verification.json'
 if a.check:assert out.read_text()==text,'Reference observation/source drift'
 else:out.write_text(text)
 print('PASS',len(rows),'actual immutable public Rust observations')
if __name__=='__main__':main()
