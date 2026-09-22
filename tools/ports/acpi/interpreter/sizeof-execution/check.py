#!/usr/bin/env python3
"""Check real Omega SizeOf opcode assertions and changed-expectation controls."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time
from types import SimpleNamespace

HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
RUNNER=Path('/tmp/cathedral-field-protocol-target/release/cathedral-acpi-checked-runner')
PIN='eaa7993a23623cd8fabf45350340479c5c9c7879'
DIRECTORIES={'execution':HERE,'pipeline':ROOT/'tools/ports/acpi/pipeline','generic':HERE.parent/'generic-execution','queries':ROOT/'tools/ports/acpi/aml/object-queries','objecttype':HERE.parent/'object-type-execution','objectdecoder':HERE.parent/'object-type-execution'}
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def text_sha(text):return hashlib.sha256(text.encode()).hexdigest()
def module(group):
    directory=DIRECTORIES[group];sys.path.insert(0,str(directory))
    spec=importlib.util.spec_from_file_location('query_opcode_'+group,directory/'fixtures.py');value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);sys.path.pop(0)
    if group=='objecttype':return SimpleNamespace(cases=value.execution_cases,render=value.base.render,full_renderer=True)
    if group=='objectdecoder':return SimpleNamespace(cases=value.decoder_cases,render=value.render_decoder,full_renderer=True)
    return value

def source(group,rows,fixture):
    if group=='generic' or getattr(fixture,'full_renderer',False):return fixture.render(rows)
    text=fixture.IMPORTS+getattr(fixture,'HELPERS','')+'\ndata Suite {}\n';names=[]
    for row in rows:
        for control in [False,True]:
            entry='Suite::'+row['name']+('_control' if control else '_positive')
            text+=fixture.render(row,control,entry);names.append(entry+'='+str(int(control)))
    return text,names

SUITES={'execution':'SizeOfExecutionSuite','pipeline':'SizeOfPipelineSuite','generic':'SizeOfGenericSuite','queries':'SizeOfMetadataSuite','objecttype':'SizeOfPriorObjectTypeSuite','objectdecoder':'SizeOfPriorObjectDecoderSuite'}

def module_source(group,rows,fixture):
    body,names=source(group,rows,fixture);receiver=SUITES[group]
    assert all(name.startswith('Suite::') for name in names)
    body='module authored_'+group+';\n'+re.sub(r'\bSuite\b',receiver,body)
    return body,[name.replace('Suite::',receiver+'::',1) for name in names]

def driver_source(groups):return ''.join('use authored_'+group+';\n' for group in groups)

def build_text(group='execution',root=ROOT):
    text='machine build(builder:&mut Build){builder.application("cathedral-sizeof-opcode-tests");builder.freestanding=true;'
    dependencies=[('aml','aml')] if group=='queries' else [('aml','aml'),('pipeline','pipeline'),('execution','interpreter/execution'),('integer_helpers','interpreter')]
    for alias,folder in dependencies:
        text+='builder.depend_as("'+alias+'",Source::Path {location:"'+str(root/'source/libraries/acpi'/folder)+'"});'
    return text+'}\n'

def snapshot(groups):
    paths=set((ROOT/'source/libraries/acpi').rglob('*.omg'))
    paths.update(path for path in HERE.glob('*.py') if path.name!='reference.py')
    paths.add(HERE/'toolchain.json');paths.add(HERE/'baseline.json');paths.add(ROOT/'source/libraries/acpi/interpreter/execution/sizeof-inventory.json')
    paths.add(HERE.parent/'generic-execution/focused/bridge-atomicity/main.omg');paths.add(HERE.parent/'generic-execution/decoder_fixtures.py');paths.add(HERE.parent/'execution/fixtures.py');paths.add(HERE.parent/'execution/checked_runner.rs');paths.add(HERE.parent/'execution/runner.Cargo.lock')
    for group in groups:
        paths.update(DIRECTORIES[group].glob('*.py'))
        paths.update(DIRECTORIES[group].glob('*-cases.json'))
        if (DIRECTORIES[group]/'cases.json').exists():paths.add(DIRECTORIES[group]/'cases.json')
    paths.discard(HERE/'reference.py')
    return {str(path.relative_to(ROOT)):sha(path) for path in sorted(paths)}

def validate(output,names):
    assert 'CHECKED authored package and dependency bodies;' in output,output
    actual=re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=',output,re.M)
    assert len(actual)==len(names),output
    for (name,expected,observed),selection in zip(actual,names):assert name+'='+expected==selection and expected==observed,output

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--group',choices=DIRECTORIES,action='append');p.add_argument('--match',default='');p.add_argument('--runner',type=Path,default=RUNNER);p.add_argument('--workers',type=int,default=1);p.add_argument('--batch-size',type=int,default=1000);p.add_argument('--combine',action='store_true');p.add_argument('--record',type=Path,default=HERE/'verification.json');args=p.parse_args()
    groups=args.group or ['execution'];assert 1<=args.workers<=4 and args.batch_size>0 and len(groups)==len(set(groups))
    inputs=snapshot(groups);binary=sha(args.runner);tasks=[]
    for group in groups:
        fixture=module(group);rows=[row for row in fixture.cases() if any(part in row['name'] for part in args.match.split(','))];assert rows,group
        tasks.extend((group,rows[at:at+args.batch_size],fixture) for at in range(0,len(rows),args.batch_size))
    start=time.monotonic()
    def run(task):
        group,rows,fixture=task;text,names=source(group,rows,fixture);build=build_text(group);started=time.monotonic()
        with tempfile.TemporaryDirectory(prefix='cathedral-sizeof-opcodes-') as directory:
            work=Path(directory);(work/'main.omg').write_text(text);(work/'build.omg').write_text(build)
            process=subprocess.run([str(args.runner),str(work/'main.omg'),str(work/'build'),*names],capture_output=True,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
        output=process.stdout+process.stderr
        unchanged=inputs==snapshot(groups) and binary==sha(args.runner)
        value=dict(group=group,cases=[row['name'] for row in rows],selections=names,source_sha256=text_sha(text),build_text=build,build_sha256=text_sha(build),output=output,exit_code=process.returncode,source_unchanged=unchanged,elapsed_seconds=round(time.monotonic()-started,3))
        print('BATCH',group,len(rows),'pairs; exit',process.returncode,flush=True);return value
    if args.combine:
        assert args.workers==1 and len(tasks)==len(groups), 'combined mode requires one worker and one complete chunk per group'
        names=[];modules=[];driver=driver_source(groups);build=build_text();started=time.monotonic()
        with tempfile.TemporaryDirectory(prefix='cathedral-sizeof-combined-') as directory:
            work=Path(directory);(work/'main.omg').write_text(driver);(work/'build.omg').write_text(build)
            for group,rows,fixture in tasks:
                body,entries=module_source(group,rows,fixture);(work/('authored_'+group+'.omg')).write_text(body)
                modules.append(dict(group=group,cases=[row['name'] for row in rows],source_sha256=text_sha(body),selections=entries));names.extend(entries)
            process=subprocess.run([str(args.runner),str(work/'main.omg'),str(work/'build'),*names],capture_output=True,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
        output=process.stdout+process.stderr
        unchanged=inputs==snapshot(groups) and binary==sha(args.runner)
        batches=[dict(modules=modules,main_text=driver,main_sha256=text_sha(driver),build_text=build,build_sha256=text_sha(build),selections=names,output=output,exit_code=process.returncode,source_unchanged=unchanged,elapsed_seconds=round(time.monotonic()-started,3))]
        print('BATCH combined',sum(len(rows) for _,rows,_ in tasks),'pairs; exit',process.returncode,flush=True)
    else:
        with ThreadPoolExecutor(max_workers=args.workers) as pool:batches=list(pool.map(run,tasks))
    count=sum(len(rows) for _,rows,_ in tasks)
    unchanged=inputs==snapshot(groups) and binary==sha(args.runner) and all(batch['source_unchanged'] for batch in batches)
    record=dict(stage='checked interpreter; authored Omega bodies with changed-expectation controls; native/hardware not run',omega_revision=PIN,execution_root=str(ROOT),runner_path=str(args.runner.resolve()),runner_sha256=binary,input_sha256=inputs,groups=groups,scope='selected' if args.match else 'full',positive_count=count,control_count=count,workers=args.workers,combined=args.combine,batches=batches,exit_code=0 if all(batch['exit_code']==0 for batch in batches) else 1,source_unchanged=unchanged,elapsed_seconds=round(time.monotonic()-start,3))
    args.record.parent.mkdir(parents=True,exist_ok=True)
    args.record.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
    assert unchanged,'inputs changed during execution; see retained receipt'
    for batch in batches:
        assert batch['exit_code']==0,batch['output']
        validate(batch['output'],batch['selections'])
    print('PASS',count,'SizeOf opcode/regression pairs')
if __name__=='__main__':main()
