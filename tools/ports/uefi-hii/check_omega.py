#!/usr/bin/env python3
"""Run live Omega source checks and positive/negative semantic helper fixtures."""
from pathlib import Path
import json
import subprocess
import sys
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
OMEGA=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else ROOT.parent/'Omega/target/release/omega'
if not OMEGA.is_file():raise SystemExit('Omega release binary absent: '+str(OMEGA))
modules=sorted({r['module'] for r in json.loads((HERE/'schema.json').read_text())})+['hii_layouts']
roots=['source/contracts/uefi/raw/'+m+'.omg' for m in modules]+['source/libraries/uefi/hii_helpers.omg','tools/ports/uefi-hii/main.omg']
for root in roots:
    result=subprocess.run([str(OMEGA),'--check',root],cwd=ROOT,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    if result.returncode:raise SystemExit(root+' FAILED\n'+result.stdout)
    print('PASS '+root,flush=True)
negative=subprocess.run([str(OMEGA),'--check','tools/ports/uefi-hii/negative.omg'],cwd=ROOT,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
if negative.returncode==0 or '0 + 1 == 0' not in negative.stdout:raise SystemExit('negative control did not fail expected evaluated contract:\n'+negative.stdout)
print('PASS negative control rejected evaluated 0 + 1 == 0; helper bodies executed, no native/firmware execution',flush=True)
