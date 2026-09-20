#!/usr/bin/env python3
"""Whole raw UEFI validation; report known compiler gaps without claiming ABI success."""
import argparse
import hashlib
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
RAW = ROOT / 'source/contracts/uefi/raw'
sys.path.insert(0, str(ROOT / 'tools/ports'))
import vectors
import inventory


def run(arguments):
    print('+ ' + ' '.join(str(arg) for arg in arguments), flush=True)
    subprocess.run([str(arg) for arg in arguments], cwd=ROOT, check=True)


def reflection_gap(omega, module, maximum):
    result = subprocess.run([str(omega), '--check', str(RAW / (module + '.omg'))], cwd=ROOT,
                            text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    actual = re.findall(r'error: cannot prove index `(\d+)` is within length 32', result.stdout)
    all_errors = re.findall(r'^\s*error:', result.stdout, re.M)
    if result.returncode == 0:
        raise ValueError(module + ': recorded reflection blocker no longer reproduces; review status and add positive coverage')
    if actual != [str(index) for index in range(32, maximum)] or len(all_errors) != len(actual):
        raise ValueError(module + ': unexpected compiler failure:\n' + result.stdout)
    print(f'CONFIRMED BLOCKER {module}: reflection indices32..{maximum - 1}; this is not a passing layout test', flush=True)


def consumer_gap(omega, path, markers, error_count):
    result = subprocess.run([str(omega), '--check', path], cwd=ROOT,
                            text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    errors = re.findall(r'^\s*error:', result.stdout, re.M)
    if result.returncode == 0 or len(errors) != error_count or any(marker not in result.stdout for marker in markers):
        raise ValueError(path + ': recorded consumer blocker changed; review status:\n' + result.stdout)
    print('CONFIRMED BLOCKER ' + path + '; selected-layout consumer did not pass', flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--omega', type=Path, default=ROOT.parent / 'Omega/target/release/omega')
    scope = parser.add_mutually_exclusive_group()
    scope.add_argument('--omega-only', action='store_true', help='run Omega checks after separately completed upstream checks')
    scope.add_argument('--host-only', action='store_true', help='skip all Omega checks explicitly')
    args = parser.parse_args()
    python = sys.executable
    if not args.omega_only:
        run([python, 'tools/ports/uefi-conformance/inventory.py'])
        for path in sorted(RAW.glob('*.vectors.json')):
            count = len(vectors.validate(inventory.read_json(path))['measurements'])
            print(f'{path.name}: {count} expected vectors valid; Omega ABI comparison NOT RUN', flush=True)
        for slice_name in ['tables', 'storage', 'tcg', 'machine']:
            run([python, f'tools/ports/uefi-{slice_name}/check.py'])
        run([python, 'tools/ports/uefi-machine/check.py', '--slice', 'shell'])
        for slice_name in ['console', 'network', 'hii']:
            run([python, f'tools/ports/uefi-{slice_name}/measure.py'])
            run([python, f'tools/ports/uefi-{slice_name}/check_transcription.py'])
        for slice_name in ['console', 'hii']:
            run(['cargo', 'test', '--locked', '--manifest-path', f'tools/ports/uefi-{slice_name}/Cargo.toml'])
    if args.host_only:
        print('ALL Omega source/semantic/layout checks SKIPPED by --host-only')
        return
    omega = args.omega.resolve()
    print('Omega binary SHA-256:', hashlib.sha256(omega.read_bytes()).hexdigest(), flush=True)
    for name in ['main', 'fixed_plans']:
        run([omega, '--check', f'tools/ports/uefi-conformance/{name}.omg'])
    run([python, 'tools/ports/uefi-scalars/check.py', '--omega', omega])
    for slice_name in ['tables', 'storage']:
        run([python, f'tools/ports/uefi-{slice_name}/semantic-check.py', '--omega', omega])
    # Raw declarations and fixed policies are checked by the combined roots above.
    # These fixtures force evaluation of real helper bodies and reject a changed assertion.
    for slice_name in ['console', 'hii']:
        run([omega, '--check', f'tools/ports/uefi-{slice_name}/main.omg'])
        negative = subprocess.run([str(omega), '--check', f'tools/ports/uefi-{slice_name}/negative.omg'], cwd=ROOT,
                                  text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        if negative.returncode == 0 or 'cannot prove requires contract' not in negative.stdout or '0 + 1 == 0' not in negative.stdout:
            raise ValueError(slice_name + ': negative control did not reject evaluated result:\n' + negative.stdout)
        print(slice_name + ': semantic fixture passes; changed assertion rejected', flush=True)
    run([python, 'tools/ports/uefi-network/check.py', '--omega', omega])
    run([python, 'tools/ports/uefi-conformance/behavior_controls.py', '--omega', omega])
    for module, maximum in [('tables_layouts', 45), ('shell_layouts', 46), ('network_large_layouts', 34)]:
        reflection_gap(omega, module, maximum)
    for path, field, count in [
        ('console/layout_projection', 'InputKeyX64Layout<InputKey>::scan_code', 2),
        ('network/layout_projection', 'NetworkStatisticsX64Layout<NetworkStatistics>::rx_total_frames', 1),
        ('network/layout_union_view', 'PxeBaseCodePacketX64Layout<PxeBaseCodePacket>::raw', 1),
        ('hii/layout_projection', 'HiiDateX64Layout<HiiDate>::year', 2),
    ]:
        consumer_gap(omega, 'tools/ports/uefi-' + path + '.omg', ['selects private data', field], count)
    consumer_gap(omega, 'tools/ports/uefi-console/layout_probe.omg',
                 ['PlacedField::read', 'PlacedField::take', 'PlacedField::write', 'expected 0 callable generic parameter(s), got 1'], 3)
    for path in ['console/layout_type_only', 'console/layout_array_probe', 'hii/layout_probe']:
        run([omega, '--check', 'tools/ports/uefi-' + path + '.omg'])
    print('Source, semantic and upstream checks complete. Recorded reflection failures reproduced.')
    print('Emitted Omega ABI comparison, native callbacks, firmware execution and Cathedral integration NOT RUN.')


if __name__ == '__main__':
    main()
