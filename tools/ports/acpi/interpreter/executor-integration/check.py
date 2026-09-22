#!/usr/bin/env python3
"""Execute original body pairs in bounded packages with full dependency checks."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time
import fixtures

HERE, ROOT = fixtures.HERE, fixtures.ROOT
PIN = 'eaa7993a23623cd8fabf45350340479c5c9c7879'
RUNNER = Path('/tmp/cathedral-field-protocol-target/release/cathedral-acpi-checked-runner')
GROUPS = fixtures.GROUPS


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def text_sha(source):
    return hashlib.sha256(source.encode()).hexdigest()


def snapshot():
    paths=set((ROOT/'source/libraries/acpi').rglob('*.omg'))
    folders=[HERE,HERE.parent/'logical-execution',HERE.parent/'to-string-execution',
             HERE.parent/'concat-execution',HERE.parent/'field-write-execution',
             HERE.parent/'mid-execution',HERE.parent.parent/'pipeline/field-writes']
    for folder in folders:
        paths.update(folder.glob('*.py'))
        paths.update(folder.glob('*-cases.json'))
        if (folder/'toolchain.json').exists():paths.add(folder/'toolchain.json')
    paths.update({HERE.parent/'execution'/name for name in ['fixtures.py','checked_runner.rs','runner.Cargo.lock']})
    paths.update({fixtures.GENERIC/name for name in ['fixtures.py','decoder_fixtures.py','focused/bridge-atomicity/main.omg']})
    paths.update({fixtures.NAMED/'fixtures.py',HERE.parent/'to-integer-execution/fixtures.py',
                  HERE.parent.parent/'pipeline/fixtures.py',HERE.parent.parent/'pipeline/field-writes/rejection-helpers.omg'})
    return {str(path.relative_to(ROOT)):sha(path) for path in sorted(paths)}


def rows(group):
    return fixtures.rows(group)


def module_source(group, selected):
    source, entries = fixtures.render_rows(group, selected)
    receiver = ''.join(part.title() for part in group.split('_'))+'IntegratedSuite'
    source = 'module authored_'+group+';\n'+re.sub(r'\bSuite\b', receiver, source)
    return source, [entry.replace('Suite::', receiver+'::', 1) for entry in entries]


def driver_source(groups):
    return ''.join('use authored_'+group+';\n' for group in groups)


def plan_batches(selections, batch_size, combine_groups=False):
    """Pack case names in order; each group keeps its own unchanged module."""
    assert batch_size > 0 and selections
    assert len({item['group'] for item in selections}) == len(selections)
    batches, pending, used = [], [], 0
    for selection in selections:
        group, cases = selection['group'], selection['cases']
        assert group in GROUPS and cases and len(cases) == len(set(cases))
        offset = 0
        while offset < len(cases):
            chosen = cases[offset:offset + batch_size - used]
            pending.append(dict(group=group, cases=chosen))
            offset += len(chosen)
            used += len(chosen)
            if used == batch_size:
                batches.append(pending)
                pending, used = [], 0
        if pending and not combine_groups:
            batches.append(pending)
            pending, used = [], 0
    if pending:
        batches.append(pending)
    return batches


def render_package(plan):
    assert plan and len({item['group'] for item in plan}) == len(plan)
    sources, modules, entries = {}, [], []
    for selection in plan:
        group, names = selection['group'], selection['cases']
        by_name = {row['name']: row for row in rows(group)}
        assert names and len(names) == len(set(names))
        source, selected = module_source(group, [by_name[name] for name in names])
        sources['authored_'+group+'.omg'] = source
        modules.append(dict(group=group, cases=names, source_sha256=text_sha(source), selections=selected))
        entries.extend(selected)
    assert len(entries) == len(set(entries))
    return sources, modules, entries, driver_source([item['group'] for item in plan])


def build_text(root=ROOT):
    source = 'machine build(builder:&mut Build){builder.application("cathedral-executor-integration");builder.freestanding=true;'
    for alias, folder in [('aml','aml'),('execution','interpreter/execution'),('integer_helpers','interpreter'),('pipeline','pipeline'),('write_values','field_writes'),('access','field_access')]:
        source += 'builder.depend_as("'+alias+'",Source::Path {location:"'+str(root/'source/libraries/acpi'/folder)+'"});'
    return source+'}'


def validate(output, entries):
    assert output.count('CHECKED authored package and dependency bodies; native publication NOT requested') == 1
    assert not re.search(r'^FAIL ', output, re.M)
    actual = re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=', output, re.M)
    assert len(re.findall(r'^PASS ', output, re.M)) == len(actual), 'Malformed extra PASS line'
    assert len(actual) == len(entries)
    assert [name+'='+expected for name,expected,observed in actual] == entries
    assert all(expected == observed for name,expected,observed in actual)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--group', choices=GROUPS, action='append')
    parser.add_argument('--match', default='')
    parser.add_argument('--runner', type=Path, default=RUNNER)
    parser.add_argument('--record', type=Path, default=HERE/'checked-verification.json')
    parser.add_argument('--batch-size', type=int, default=10)
    parser.add_argument('--workers', type=int, default=1)
    parser.add_argument('--combine-groups', action='store_true',
                        help='Share one compiler check across groups, up to batch-size pairs per package.')
    args = parser.parse_args()
    toolchain=json.loads((HERE/'toolchain.json').read_text())
    omega=Path(toolchain['omega_source'])
    assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=omega,text=True).strip()==PIN
    assert not subprocess.check_output(['git','status','--porcelain'],cwd=omega,text=True).strip()
    assert sha(args.runner)==toolchain['sha256'][str(args.runner)]
    groups = args.group or GROUPS
    assert len(groups) == len(set(groups))
    assert args.batch_size>0 and 1<=args.workers<=3
    before, binary = snapshot(), sha(args.runner)
    selections = []
    for group in groups:
        selected = [row for row in rows(group) if not args.match or row['name'] in args.match.split(',')]
        assert selected
        selections.append(dict(group=group,cases=[row['name'] for row in selected]))
    if args.match:
        assert set(args.match.split(',')) == {name for item in selections for name in item['cases']}, 'Unknown selected case'
    batches = plan_batches(selections, args.batch_size, args.combine_groups)
    build = build_text()
    started = time.monotonic()
    def run_batch(batch):
        batch_started=time.monotonic()
        sources,modules,entries,driver=render_package(batch)
        assert before==snapshot() and binary==sha(args.runner), 'Inputs changed before batch'
        with tempfile.TemporaryDirectory(prefix='cathedral-executor-integration-') as directory:
            work=Path(directory)
            (work/'main.omg').write_text(driver)
            (work/'build.omg').write_text(build)
            for name,source in sources.items():
                (work/name).write_text(source)
            run=subprocess.run([str(args.runner),str(work/'main.omg'),str(work/'build'),*entries],
                               capture_output=True,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
        unchanged=before==snapshot() and binary==sha(args.runner)
        return dict(modules=modules,
                    driver_sha256=text_sha(driver),selections=entries,exit_code=run.returncode,
                    source_unchanged=unchanged,elapsed_seconds=round(time.monotonic()-batch_started,3),output=run.stdout+run.stderr)
    completed=[]
    with ThreadPoolExecutor(max_workers=args.workers) as pool:
        for batch in pool.map(run_batch,batches):
            completed.append(batch)
            print('BATCH',','.join(item['group'] for item in batch['modules']),len(batch['selections'])//2,
                  'pairs; exit',batch['exit_code'],';',batch['elapsed_seconds'],'seconds',flush=True)
            if batch['exit_code']:
                print(batch['output'],flush=True)
    unchanged = before == snapshot() and binary == sha(args.runner)
    assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=omega,text=True).strip()==PIN
    assert not subprocess.check_output(['git','status','--porcelain'],cwd=omega,text=True).strip()
    count=sum(len(selection['cases']) for selection in selections)
    record = dict(format_version=2,stage='checked interpreter; complete authored and dependency bodies; changed-expectation controls',
                  omega_revision=PIN, execution_root=str(ROOT), groups=groups, scope='selected' if args.match else 'full',
                  input_sha256=before, runner_path=str(args.runner.resolve()), runner_sha256=binary,
                  source_unchanged=unchanged, native_execution=False,
                  exit_code=0 if all(batch['exit_code']==0 for batch in completed) else 1,
                  groups_selected=selections,batches=completed,batch_size=args.batch_size,workers=args.workers,
                  combine_groups=args.combine_groups,
                  positive_count=count,control_count=count,build_source=build,build_sha256=text_sha(build),
                  elapsed_seconds=round(time.monotonic()-started,3),
                  batch_elapsed_seconds_sum=round(sum(batch['elapsed_seconds'] for batch in completed),3))
    args.record.parent.mkdir(parents=True, exist_ok=True)
    args.record.write_text(json.dumps(record, indent=2, sort_keys=True)+'\n')
    assert unchanged, 'Inputs changed during verification'
    for batch in completed:
        assert batch['source_unchanged'], 'Inputs changed during batch'
    assert record['exit_code']==0, 'Compilation or execution failed; see retained diagnostic'
    for batch in completed:
        validate(batch['output'],batch['selections'])
    print('PASS', count, 'checked pairs;', record['elapsed_seconds'], 'seconds', flush=True)


if __name__ == '__main__':
    main()
