#!/usr/bin/env python3
"""Reproduce source binding and public/mirrored Rust witnesses; optional const proof."""
import argparse,hashlib,json,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
OMEGA=Path('/tmp/cathedral-omega-eaa7993/release/omega')
def run(*args):subprocess.run([str(x)for x in args],cwd=ROOT,check=True)
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--host-only',action='store_true');p.add_argument('--omega',type=Path,default=OMEGA);a=p.parse_args()
 for script in ['generate_reference.py','generate.py','generate_extras.py','generate_inventory.py']:run(sys.executable,HERE/script,'--check')
 # Private geometry was retained verbatim; changes demand a fresh reconciliation.
 lib=ROOT/'source/libraries/x86_64'
 baseline=(lib/'cleanup_ranges.omg').read_text();profile=(lib/'encrypted_cleanup_ranges.omg').read_text()
 assert baseline[baseline.index('// Once a selected link'):]==profile[profile.index('// Once a selected link'):]
 run('cargo','+nightly-2026-09-04','build','--manifest-path',HERE/'Cargo.toml','--locked','--offline')
 for n in range(len(json.loads((HERE/'profiles.json').read_text()))):run(HERE/'target/debug/cathedral-x86-64-encrypted-cleanup-witness',n)
 print('PASS 181 actual public mapped calls + 203 adapted private recursive calls in 11 isolated profiles',flush=True)
 if a.host_only:return
 # A current profile-sensitive success body and a changed expected retirement
 # exercise the constant evaluator under the unchanged success contract.
 metadata=json.loads((HERE/'fixtures.json').read_text());control=next(c for c in metadata['controls'] if c['family']=='masked-retirement')
 run(a.omega,'--check',HERE/control['path'])
 with tempfile.TemporaryDirectory(prefix='cathedral-encrypted-cleanup-const-control-')as temp:
  d=Path(temp);source=(HERE/control['path']).read_text();assert source.count(control['old'])==1
  (d/'main.omg').write_text(source.replace(control['old'],control['new']))
  (d/'build.omg').write_text((HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/'))
  result=subprocess.run([str(a.omega),'--check',str(d/'main.omg')],cwd=ROOT,text=True,capture_output=True)
  output=result.stdout+result.stderr
  assert result.returncode and 'cannot prove requires contract' in output and '1 == 0' in output,output
 print('PASS current representative const body and changed-body rejection; Omega SHA256',hashlib.sha256(a.omega.read_bytes()).hexdigest(),flush=True)
if __name__=='__main__':main()
