#!/usr/bin/env python3
"""Validate retained source, generated body, controls and public observations."""
import argparse
from collections import Counter
import json
from pathlib import Path

import check
import reference


def checked(path, require_binaries):
    record=json.loads(path.read_text())
    assert record['exit_code']==0 and record['source_unchanged'] is True
    assert record['batches'] and all(batch['exit_code']==0 and batch['source_unchanged'] is True for batch in record['batches'])
    toolchain=json.loads((check.HERE/'toolchain.json').read_text())
    assert toolchain['omega_revision']==record['omega_revision']==check.PIN
    runner_hashes=[digest for path,digest in toolchain['sha256'].items() if Path(path).name in ['cathedral-acpi-checked-runner','cathedral-acpi-checked-runner.exe']]
    assert runner_hashes==[record['runner_sha256']]
    assert record['input_sha256']==check.snapshot(record['groups']), 'source closure differs'
    selected={group:[] for group in record['groups']}
    if record.get('combined'):
        assert len(record['batches'])==1
        batch=record['batches'][0];names=[]
        assert [module['group'] for module in batch['modules']]==record['groups']
        for module in batch['modules']:
            group=module['group'];fixture=check.module(group);by_name={row['name']:row for row in fixture.cases()}
            body,entries=check.module_source(group,[by_name[name] for name in module['cases']],fixture)
            assert check.text_sha(body)==module['source_sha256'] and entries==module['selections']
            names.extend(entries);selected[group].extend(module['cases'])
        driver=check.driver_source(record['groups']);build=check.build_text(root=Path(record['execution_root']))
        assert driver==batch['main_text'] and check.text_sha(driver)==batch['main_sha256']
        assert build==batch['build_text'] and check.text_sha(build)==batch['build_sha256']
        assert names==batch['selections'];check.validate(batch['output'],names)
    for batch in [] if record.get('combined') else record['batches']:
        group=batch['group'];fixture=check.module(group)
        by_name={row['name']:row for row in fixture.cases()}
        body,names=check.source(group,[by_name[name] for name in batch['cases']],fixture)
        build=check.build_text(group,Path(record['execution_root']))
        assert check.text_sha(body)==batch['source_sha256']
        assert build==batch['build_text'] and check.text_sha(build)==batch['build_sha256']
        assert names==batch['selections']
        check.validate(batch['output'],names)
        selected[group].extend(batch['cases'])
    for group,names in selected.items():
        assert len(names)==len(set(names))
        if record['scope']=='full':assert names==[row['name'] for row in check.module(group).cases()]
    count=sum(map(len,selected.values()))
    assert count==record['positive_count']==record['control_count']
    if require_binaries:assert check.sha(Path(record['runner_path']))==record['runner_sha256']
    print('PASS',path.name,count,'exact checked pairs and source/body bindings')


def public():
    record=json.loads((check.HERE/'reference-verification.json').read_text())
    assert record['pin']==reference.PIN
    assert record['source_sha256']=={str(path.relative_to(check.ROOT)):check.sha(path) for path in reference.source_inputs()}
    rows=reference.public_cases();assert len(rows)==len(record['rows'])
    for retained,authored in zip(record['rows'],rows):
        assert retained['name']==authored['name'] and retained['aml_hex']==authored['aml_hex']
        assert retained['revision']==authored['revision'] and retained['expected']==authored['expected']
        assert retained['profile_outcome']==authored['error']
        assert retained['category']==reference.validate_observed(authored,retained['observed'])
    assert record['counts']==dict(Counter(row['category'] for row in record['rows']))
    upstream=check.ROOT/'reference_code/rust-osdev/acpi'
    if upstream.exists():assert all(check.sha(upstream/name)==digest for name,digest in record['upstream_sha256'].items())
    inventory=json.loads((check.ROOT/'source/libraries/acpi/interpreter/execution/sizeof-inventory.json').read_text())
    assert inventory['upstream']['revision']==reference.PIN
    for name,metadata in inventory['files'].items():
        assert metadata['sha256']==record['upstream_sha256'][name]
        if upstream.exists():
            lines=(upstream/name).read_text().splitlines()
            for key,symbol in metadata['symbols'].items():
                assert lines[int(key.split(':',1)[0])-1].strip()==symbol['anchor']
                for target in symbol.get('targets',[]):assert target['anchor'] in (check.ROOT/target['path']).read_text()
    print('PASS',len(rows),'public observations and exact pinned symbol anchors')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--require-binaries',action='store_true')
    parser.add_argument('--public-only',action='store_true')
    args=parser.parse_args()
    if not args.public_only:
        primary=check.HERE/'verification.json';record=json.loads(primary.read_text())
        checked(primary,args.require_binaries)
        if record.get('combined'):
            assert record['groups']==list(check.DIRECTORIES) and record['scope']=='full'
        else:checked(check.HERE/'regressions.json',args.require_binaries)
    public()


if __name__=='__main__':main()
