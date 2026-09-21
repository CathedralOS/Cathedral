#!/usr/bin/env python3
"""Check retirement witnesses and retain exact generated/input/binary evidence."""
import argparse, hashlib, json, os, re, subprocess, tempfile, time
from pathlib import Path
import fixtures

ROOT,HERE=fixtures.ROOT,fixtures.HERE
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def digest(text):return hashlib.sha256(text.encode()).hexdigest()
def snapshot():
    paths=list((ROOT/'source/libraries/acpi').rglob('*.omg'))
    paths += [HERE/'fixtures.py',HERE/'check.py',fixtures.COMPARATOR,HERE.parent/'execution/checked_runner.rs',HERE.parent/'execution/runner.Cargo.lock']
    return {str(p.relative_to(ROOT)):sha(p) for p in sorted(set(paths))}
def build():
    text='machine build(builder:&mut Build){builder.package("concat-execution");builder.freestanding=true;'
    for alias,path in [('aml','source/libraries/acpi/aml'),('execution','source/libraries/acpi/interpreter/execution'),('integer_helpers','source/libraries/acpi/interpreter'),('pipeline','source/libraries/acpi/pipeline')]:
        text+=f'builder.depend_as("{alias}",Source::Path {{location:"{ROOT/path}"}});'
    return text+'}\n'
def validate(record):
    assert record['exit_code']==0 and record['source_unchanged']
    assert record['output'].count('CHECKED authored package and dependency bodies;')==1
    actual=re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=',record['output'],re.M)
    assert [name+'='+want for name,want,got in actual]==record['selections']
    assert all(want==got for name,want,got in actual)
def verify(path):
    record=json.loads(path.read_text());rows,source,names=fixtures.render(record['match'])
    assert record['source_sha256']==snapshot()
    assert record['runner_sha256']==sha(Path(record['runner']))
    assert record['fixture_sha256']==digest(source) and record['selections']==names
    assert record['cases']==[r['name']for r in rows]
    assert record['build_source']==build() and record['build_sha256']==digest(build())
    validate(record);print('PASS exact current inputs/body/selections/binary:',path)
def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--match',default='');p.add_argument('--record',type=Path)
    p.add_argument('--verify',type=Path)
    p.add_argument('--runner',type=Path,default=Path('/tmp/cathedral-field-protocol-target/release/cathedral-acpi-checked-runner'))
    args=p.parse_args()
    if args.verify:return verify(args.verify)
    rows,source,names=fixtures.render(args.match);inputs=snapshot();binary=sha(args.runner);start=time.monotonic()
    with tempfile.TemporaryDirectory(prefix='cathedral-concat-retirement-') as name:
        work=Path(name);(work/'main.omg').write_text(source);(work/'build.omg').write_text(build())
        command=[str(args.runner),str(work/'main.omg'),str(work/'build'),*names]
        process=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
        lines=[]
        for line in process.stdout:print(line,end='',flush=True);lines.append(line)
        code=process.wait()
        record=dict(production_root=str(ROOT),match=args.match,runner=str(args.runner),runner_sha256=binary,source_sha256=inputs,source_unchanged=inputs==snapshot() and binary==sha(args.runner),fixture_sha256=digest(source),build_source=build(),build_sha256=digest(build()),cases=[r['name']for r in rows],selections=names,command=command,exit_code=code,output=''.join(lines),elapsed_seconds=time.monotonic()-start,native_execution=False)
        if args.record:args.record.write_text(json.dumps(record,indent=2)+'\n')
        validate(record);print('PASS',len(rows),'pairs in',round(record['elapsed_seconds'],3),'seconds')
if __name__=='__main__':main()
