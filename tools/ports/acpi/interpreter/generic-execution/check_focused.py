#!/usr/bin/env python3
"""Run one retained actual-Omega boundary suite with changed-body controls.

Uses an existing isolated checked interpreter; never builds or replaces a runner.
Each receipt binds both positive/control selection and the full source snapshot.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
DEFAULT_RUNNER = Path('/tmp/cathedral-acpi-generic-checked/release/cathedral-acpi-checked-runner')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def snapshot(group):
    paths = sorted((ROOT / 'source/libraries/acpi').rglob('*.omg'))
    paths += [Path(__file__).resolve()] + sorted(group.glob('*'))
    return {str(p.relative_to(ROOT)): sha(p) for p in paths if p.is_file()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('group', choices=sorted(p.name for p in (HERE / 'focused').iterdir() if p.is_dir()))
    parser.add_argument('--runner', type=Path, default=DEFAULT_RUNNER)
    parser.add_argument('--record', type=Path, required=True)
    args = parser.parse_args()
    group = HERE / 'focused' / args.group
    profile = json.loads((group / 'profile.json').read_text())
    selections = (group / 'selections.txt').read_text().splitlines()
    before = snapshot(group)
    runner_before = sha(args.runner)
    started = time.time()
    with tempfile.TemporaryDirectory(prefix='cathedral-generic-focused-') as directory:
        fixture = Path(directory)
        (fixture / 'main.omg').write_bytes((group / 'main.omg').read_bytes())
        build = 'machine build(builder:&mut Build){builder.package("generic-focused");builder.freestanding=true;'
        for alias, relative in [('aml', 'source/libraries/acpi/aml'), ('execution', 'source/libraries/acpi/interpreter/execution'), ('integer_helpers', 'source/libraries/acpi/interpreter'), ('pipeline', 'source/libraries/acpi/pipeline')]:
            build += f'builder.depend_as("{alias}",Source::Path {{location:"{ROOT / relative}"}});'
        (fixture / 'build.omg').write_text(build + '}\n')
        command = [str(args.runner), str(fixture / 'main.omg'), str(fixture / 'build'), *selections]
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
                                   env=dict(os.environ, OMEGA_INTERP_STEP_BUDGET='10000000'))
        lines = []
        for line in process.stdout:
            print(line, end='', flush=True)
            lines.append(line)
        code = process.wait()
        unchanged = before == snapshot(group) and runner_before == sha(args.runner)
        receipt = dict(execution_root=str(ROOT.resolve()), stage='checked interpreter', native_execution=False, group=args.group,
                       scenarios=profile['pairs'], controls=profile['pairs'], exit_code=code,
                       started_unix=started, elapsed_seconds=time.time() - started,
                       source_unchanged=unchanged, source_sha256=before,
                       runner_sha256=runner_before, fixture_sha256=sha(fixture / 'main.omg'),
                       build_sha256=sha(fixture / 'build.omg'), build_source=build+'}\n',
                       selections=selections, command=command, output=''.join(lines))
        args.record.parent.mkdir(parents=True, exist_ok=True)
        args.record.write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
        assert unchanged, 'Source, fixture, or runner changed during verification'
        raise SystemExit(code)


if __name__ == '__main__':
    main()
