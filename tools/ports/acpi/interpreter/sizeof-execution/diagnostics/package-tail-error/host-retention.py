#!/usr/bin/env python3
"""Host-only SizeOf receipt tests; no Omega, Rust, or provider execution.

Synthetic runner outputs and receipts live only in a TemporaryDirectory and are
deleted before this driver returns. The optional report contains host-test
results and source hashes, never a checked-execution receipt.
"""
import argparse
import contextlib
import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
from unittest.mock import patch


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def run(root):
    root = root.resolve()
    here = root / 'tools/ports/acpi/interpreter/sizeof-execution'
    sys.path.insert(0, str(here))
    check = load('check', here / 'check.py')
    verify = load('verify_record', here / 'verify_record.py')
    bound_before = check.snapshot(list(check.DIRECTORIES))
    selected_before = check.snapshot(['execution'])
    selected_changed = dict(selected_before)
    selected_changed[next(iter(selected_changed))] = '0' * 64
    scenarios = []
    rejected = []

    with tempfile.TemporaryDirectory(prefix='sizeof-host-only-receipts-') as directory:
        for combined in (False, True):
            for mode in ('compile_failure', 'entry_failure', 'invalid_output',
                         'success', 'source_changed'):
                record_path = Path(directory) / (str(combined) + '-' + mode + '.json')
                argv = ['check.py', '--group', 'execution', '--match',
                        'size_malformed_package', '--record', str(record_path)]
                if combined:
                    argv.append('--combine')
                calls = {'transport': 0, 'snapshot': 0}

                def transport(command, **kwargs):
                    calls['transport'] += 1
                    output = 'CHECKED authored package and dependency bodies; native publication NOT requested\n'
                    for selection in command[3:]:
                        name, expected = selection.rsplit('=', 1)
                        output += ('PASS ' + name + ' expected=' + expected +
                                   ' observed=' + expected +
                                   ' error=None usage=HOST_TEST_ONLY\n')
                    if mode == 'compile_failure':
                        return SimpleNamespace(returncode=2, stdout='',
                                               stderr='HOST_TEST_ONLY compile diagnostic\n')
                    if mode == 'entry_failure':
                        return SimpleNamespace(returncode=1,
                                               stdout=output.replace('PASS ', 'FAIL ', 1), stderr='')
                    if mode == 'invalid_output':
                        return SimpleNamespace(returncode=0,
                                               stdout='HOST_TEST_ONLY malformed output\n', stderr='')
                    return SimpleNamespace(returncode=0, stdout=output, stderr='')

                def changed_snapshot(groups):
                    calls['snapshot'] += 1
                    return selected_before if calls['snapshot'] == 1 else selected_changed

                failed = False
                with contextlib.ExitStack() as stack:
                    stack.enter_context(patch.object(sys, 'argv', argv))
                    stack.enter_context(patch.object(check.subprocess, 'run', side_effect=transport))
                    stack.enter_context(contextlib.redirect_stdout(io.StringIO()))
                    if mode == 'source_changed':
                        stack.enter_context(patch.object(check, 'snapshot', side_effect=changed_snapshot))
                    try:
                        check.main()
                    except AssertionError:
                        failed = True
                assert calls['transport'] == 1, (combined, mode, calls)
                assert failed == (mode != 'success'), (combined, mode)
                retained = json.loads(record_path.read_text())
                assert len(retained['batches']) == 1
                expected_runner_exit = 2 if mode == 'compile_failure' else 1 if mode == 'entry_failure' else 0
                assert retained['batches'][0]['exit_code'] == expected_runner_exit
                assert retained['exit_code'] == (1 if expected_runner_exit else 0)
                assert retained['source_unchanged'] is (mode != 'source_changed')
                assert retained['batches'][0]['source_unchanged'] is (mode != 'source_changed')
                if mode == 'success':
                    with contextlib.redirect_stdout(io.StringIO()):
                        verify.checked(record_path, True)
                    for field in ('exit_code', 'source_unchanged'):
                        for scope in ('record', 'batch'):
                            changed = copy.deepcopy(retained)
                            target = changed if scope == 'record' else changed['batches'][0]
                            target[field] = 1 if field == 'exit_code' else False
                            record_path.write_text(json.dumps(changed))
                            with contextlib.redirect_stdout(io.StringIO()):
                                try:
                                    verify.checked(record_path, True)
                                except AssertionError:
                                    pass
                                else:
                                    raise AssertionError('Accepted mutated ' + scope + ' ' + field)
                            rejected.append(dict(combined=combined, scope=scope, field=field))
                scenarios.append(dict(combined=combined, mode=mode,
                                      receipt_retained=True, checker_rejected=failed,
                                      simulated_runner_exit=expected_runner_exit,
                                      source_unchanged=retained['source_unchanged']))

    assert len(scenarios) == 10 and len(rejected) == 8
    assert check.snapshot(list(check.DIRECTORIES)) == bound_before
    return dict(stage='Host-only transport and verifier tests; no Omega execution evidence',
                execution_validation=False, synthetic_receipts_retained=False,
                root=str(root), driver_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                input_sha256=bound_before, source_unchanged=True,
                transport_scenarios=scenarios, verifier_mutation_rejections=rejected)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path('/tmp/cathedral-sizeof-corrected'))
    parser.add_argument('--record', type=Path)
    args = parser.parse_args()
    record = run(args.root)
    if args.record:
        args.record.write_text(json.dumps(record, indent=2, sort_keys=True) + '\n')
    print('PASS 10 host-only retention scenarios and 8 verifier mutations;',
          len(record['input_sha256']), 'inputs unchanged; no Omega execution')


if __name__ == '__main__':
    main()
