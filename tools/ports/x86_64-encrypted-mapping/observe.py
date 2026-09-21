#!/usr/bin/env python3
"""Run every configured profile in a fresh process; never reset global atomics."""
import json,subprocess,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
subprocess.run(['cargo','build','--quiet','--locked','--manifest-path',str(HERE/'Cargo.toml')],check=True)
profiles=json.loads((HERE/'profiles.json').read_text());rows=[]
for n in range(len(profiles)):
 rows+=json.loads(subprocess.check_output([str(HERE/'target/debug/cathedral-encrypted-mapping-reference'),str(n)],text=True))
rows.sort(key=lambda x:x['index']);assert [x['index'] for x in rows]==list(range(len(json.loads((HERE/'cases.json').read_text()))))
p=HERE/'observations.json';s=json.dumps(rows,indent=2)+'\n'
if '--check' in sys.argv:
 if p.read_text()!=s:raise SystemExit('fresh reference observations changed')
else:p.write_text(s)
print('PASS',len(rows),'instrumented observations, including',len(rows)//2,'actual public mapped comparisons in',len(profiles),'fresh profiles')
