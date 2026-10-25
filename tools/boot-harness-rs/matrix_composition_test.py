"""Boot the independent three-peer matrix and check its real IPC boundaries."""
from datetime import datetime, timezone
import os
from pathlib import Path
import shutil
import subprocess
import sys

import protocol_test
import run as harness


def main(omega=False, upstream=False):
    profile = Path(__file__).with_name('profiles') / ('omega-matrix-upstream.json' if upstream else 'omega-matrix.json' if omega else 'matrix-composition.json')
    subprocess.run([sys.executable, str(Path(__file__).with_name('run.py')),
                    '--profile', str(profile), '--release', '--build-only'], check=True)
    output = harness.BUILD / profile.stem / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    shutil.copytree(harness.BUILD / (profile.stem + '-interactive/esp'), output / 'esp')
    qemu = harness.find_qemu()
    command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
    command += harness.firmware_args(qemu, output)
    command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
    protocol_test.run(command, output, 120, subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
                      ('Cathedral Omega upstream: nested attenuation, revocation and subtree reclamation passed' if upstream else 'Cathedral alternative matrix: scoped acquisition, interrupted handoffs, rollback and subtree reclamation passed'))


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--omega', action='store_true', help='run the same proof against the Omega matrix')
    parser.add_argument('--upstream', action='store_true', help='run Omega with a real parent and three nested peers')
    args = parser.parse_args()
    main(args.omega, args.upstream)
