#!/usr/bin/env python3
"""Replay source, generated-module, binary and observed-result receipt bindings."""
import argparse
import json
from pathlib import Path
import check


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('record', type=Path, nargs='?', default=check.HERE/'checked-verification.json')
    parser.add_argument('--require-binaries', action='store_true')
    parser.add_argument('--require-complete', action='store_true')
    args = parser.parse_args()
    record = json.loads(args.record.read_text())
    assert record['omega_revision'] == check.PIN and record['exit_code'] == 0 and record['source_unchanged']
    assert record['input_sha256'] == check.snapshot()
    assert record['groups'] == [selection['group'] for selection in record['groups_selected']]
    assert len(record['groups']) == len(set(record['groups']))
    if args.require_complete:
        assert record['scope']=='full' and record['groups']==check.GROUPS
    count=0
    expected_batches=[]
    assert record['batch_size']>0 and 1<=record['workers']<=3
    for selection in record['groups_selected']:
        rows = check.rows(selection['group'])
        by_name = {row['name']:row for row in rows}
        assert len(selection['cases']) == len(set(selection['cases']))
        if record['scope'] == 'full':
            assert selection['cases'] == [row['name'] for row in rows]
        selected=[by_name[name] for name in selection['cases']]
        count+=len(selected)
        expected_batches.extend((selection['group'],selected[index:index+record['batch_size']]) for index in range(0,len(selected),record['batch_size']))
    assert len(expected_batches)==len(record['batches'])
    for (group,rows),batch in zip(expected_batches,record['batches']):
        assert batch['group']==group and batch['cases']==[row['name'] for row in rows]
        source,names=check.module_source(group,rows)
        assert check.text_sha(source)==batch['source_sha256'] and names==batch['selections']
        assert check.text_sha(check.driver_source([group]))==batch['driver_sha256']
        assert batch['exit_code']==0 and batch['source_unchanged']
        check.validate(batch['output'],names)
    assert record['positive_count']==record['control_count']==count
    if args.require_complete:
        assert count==sum(len(check.rows(group)) for group in check.GROUPS)
    assert record['batch_elapsed_seconds_sum']==round(sum(batch['elapsed_seconds'] for batch in record['batches']),3)
    build = check.build_text(Path(record['execution_root']))
    assert build == record['build_source'] and check.text_sha(build) == record['build_sha256']
    if args.require_binaries:
        assert check.sha(Path(record['runner_path'])) == record['runner_sha256']
    print('PASS', count, 'exact checked pairs;', len(record['input_sha256']), 'source hashes')


if __name__ == '__main__':
    main()
