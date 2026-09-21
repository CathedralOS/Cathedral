#!/usr/bin/env python3
"""Check the portable strict PCI routing package and execute actual Omega bodies."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import time
import generate
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
DEFAULT_RUNNER = '/tmp/cathedral-acpi-execution-checked/release/cathedral-acpi-checked-runner'
OMEGA_REVISION = 'eaa7993a23623cd8fabf45350340479c5c9c7879'
def sha(path):
 return hashlib.sha256(path.read_bytes()).hexdigest()
def inputs():
 paths = set()
 for folder in ['pci_routing', 'aml', 'resources', 'interpreter']:
  paths.update((ROOT/'source/libraries/acpi'/folder).glob('*.omg'))
 paths.update((ROOT/'source/libraries/acpi').glob('*.omg'))
 paths.update(HERE.glob('*.py'))
 paths.add(HERE/'build.omg')
 for name in ['interpreter/execution/checked_runner.rs', 'interpreter/execution/runner.Cargo.lock', 'pci-routing/fixtures.json', 'pci-routing/observations.json']:
  paths.add(ROOT/'tools/ports/acpi'/name)
 return {str(path.relative_to(ROOT)): sha(path) for path in sorted(paths)}
def validate_output(output, selections):
 assert 'CHECKED authored package and dependency bodies;' in output
 rows = re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=', output, re.M)
 assert len(rows) == len(selections), (len(rows), len(selections))
 for (name, expected, observed), selection in zip(rows, selections):
  assert name+'='+expected == selection and expected == observed, (selection, name, expected, observed)
 assert not re.search(r'^(FAIL|ERROR)\b', output, re.M)
def main():
 parser = argparse.ArgumentParser()
 parser.add_argument('--runner', default=DEFAULT_RUNNER)
 parser.add_argument('--case', action='append')
 parser.add_argument('--record', default='checked-verification.json')
 args = parser.parse_args()
 runner = Path(args.runner).resolve()
 source, selections = generate.render(args.case)
 assert selections and len(selections) % 2 == 0
 path = HERE/'run-suite.omg'; path.write_text(source)
 before = inputs(); binary = sha(runner); start = time.monotonic()
 command = [str(runner), str(path), str(HERE/'build'), *selections]
 process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, env=dict(os.environ, OMEGA_INTERP_STEP_BUDGET='10000000'))
 lines = []
 for line in process.stdout:
  print(line, end='', flush=True); lines.append(line)
 assert process.wait() == 0, 'checked source/body failure'
 output = ''.join(lines); validate_output(output, selections)
 assert inputs() == before and sha(runner) == binary, 'source/harness/runner changed during check'
 record = dict(format='cathedral-acpi-pci-routing-checked-v1', stage='actual checked interpreter bodies; no native or hardware execution', scope='full' if args.case is None else 'selected', cases=args.case, case_count=len(selections)//2, control_count=len(selections)//2, source_sha256=before, suite_sha256=hashlib.sha256(source.encode()).hexdigest(), runner_sha256=binary, runner_path=str(runner), omega_revision=OMEGA_REVISION, elapsed_seconds=round(time.monotonic()-start,3), selections=selections, output=output)
 (HERE/args.record).write_text(json.dumps(record, indent=2, sort_keys=True)+'\n')
 print('PASS', len(selections)//2, 'strict positives and changed-body controls')
if __name__ == '__main__': main()
