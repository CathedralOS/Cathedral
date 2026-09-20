#!/usr/bin/env python3
"""Bounded source-only probes of observed instruction catalog limits."""
from pathlib import Path
import subprocess
import sys
ROOT=Path(__file__).resolve().parents[3]
OMEGA=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else ROOT.parent/'Omega/target/release/omega'
for name,expected in [('port8',None),('port16','requires an exact `u8` writable place'),('invlpg','unknown asm instruction `invlpg`')]:
    path=ROOT/'tools/ports/x86_64/catalog-probes'/f'{name}.omg'
    result=subprocess.run([str(OMEGA),'--check',str(path)],cwd=ROOT,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    if (expected is None and result.returncode) or (expected is not None and (result.returncode==0 or expected not in result.stdout)):
        raise SystemExit(name+' unexpected result:\n'+result.stdout)
    print('PASS '+name+(': source checks; no I/O executed' if expected is None else ': expected source rejection: '+expected),flush=True)
