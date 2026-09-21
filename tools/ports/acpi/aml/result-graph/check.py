#!/usr/bin/env python3
"""Check the current result-graph fixture bodies and retain exact input identity."""
import argparse, hashlib, json, os, re, subprocess, tempfile, time
from pathlib import Path
import fixtures
HERE,ROOT=fixtures.HERE,fixtures.ROOT
RUNNER=Path('/tmp/cathedral-acpi-generic-checked/release/cathedral-acpi-checked-runner')
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def digest(value):return hashlib.sha256(value.encode()).hexdigest()
def snapshot():
    paths=list((ROOT/'source/libraries/acpi/aml').glob('*.omg'))+list((ROOT/'source/libraries/acpi/interpreter').glob('*.omg'))
    paths+=list(HERE.glob('*.py'))+[HERE/'cases.json']
    return {str(p.relative_to(ROOT)):sha(p) for p in sorted(paths)}
def build():return f'machine build(builder:&mut Build){{builder.package("result-graph-check");builder.freestanding=true;builder.depend_as("aml",Source::Path {{location:"{ROOT}/source/libraries/acpi/aml"}});}}\n'
def validate(output,selections):
    assert output.count('CHECKED authored package and dependency bodies;')==1
    rows=re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=',output,re.M)
    assert len(rows)==len(selections),output
    assert all(name+'='+want==selected and want==got for (name,want,got),selected in zip(rows,selections))
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--match',default='');p.add_argument('--runner',type=Path,default=RUNNER);p.add_argument('--record',type=Path);p.add_argument('--verify',type=Path);a=p.parse_args()
    if a.verify:
        r=json.loads(a.verify.read_text());rows=[v for v in fixtures.cases() if r['match'] in v['name']];source,names=fixtures.render(rows)
        assert r['execution_root']==str(ROOT) and r['source_sha256']==snapshot() and r['source_unchanged']
        assert r['build_source']==build() and r['build_sha256']==digest(build()) and r['fixture_sha256']==digest(source)
        assert r['selections']==names and r['runner_sha256']==sha(Path(r['runner'])) and r['exit_code']==0
        validate(r['output'],names);print('PASS exact current-input receipt',a.verify);return
    rows=[v for v in fixtures.cases() if a.match in v['name']];assert rows
    source,names=fixtures.render(rows);before=snapshot();binary=sha(a.runner);started=time.monotonic()
    with tempfile.TemporaryDirectory(prefix='cathedral-result-graph-') as d:
        work=Path(d);(work/'main.omg').write_text(source);(work/'build.omg').write_text(build())
        command=[str(a.runner),str(work/'main.omg'),str(work/'build'),*names]
        run=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
        output=[]
        for line in run.stdout:
            print(line if len(line)<=2000 else line[:1200]+f' ... ({len(line)} characters retained in receipt)\n',end='',flush=True);output.append(line)
        code=run.wait();text=''.join(output);unchanged=before==snapshot() and binary==sha(a.runner)
        record=dict(execution_root=str(ROOT),match=a.match,cases=[v['name']for v in rows],source_sha256=before,source_unchanged=unchanged,build_source=build(),build_sha256=digest(build()),fixture_sha256=digest(source),runner=str(a.runner),runner_sha256=binary,command=command,selections=names,exit_code=code,output=text,elapsed_seconds=time.monotonic()-started,stage='checked interpreter',native_execution=False)
        if a.record:a.record.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
        assert unchanged,'Source or runner changed during execution'
        if code:raise SystemExit(code)
        validate(text,names)
        print('PASS',len(rows),'graph behavior/control pairs')
if __name__=='__main__':main()
