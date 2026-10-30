"""Validate real chooser directory pages, owner progress, staged images and refunds."""
import argparse
from datetime import datetime, timezone
from pathlib import Path
import subprocess

from build import ROOT, prepare
from run_test import boot


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path)
    args = parser.parse_args()
    image = prepare(ROOT / 'source/distribution/applications/chooser/native', args.compiler)
    output = ROOT / 'build/omega-user/chooser-control' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    cargo = ROOT / 'build/boot-harness-rs/cargo'
    with (output / 'owner-build.log').open('w') as log:
        subprocess.run(['cargo', 'build', '--locked', '--release', '--package', 'cathedral-channels-lab',
                        '--features', 'omega-chooser', '--target', 'x86_64-unknown-none',
                        '--target-dir', str(cargo)], cwd=ROOT / 'source-rs',
                       stdout=log, stderr=subprocess.STDOUT, check=True)
    owner = cargo / 'x86_64-unknown-none/release/cathedral-channels-lab'
    composition = dict(CATHEDRAL_LAUNCH_COUNT='1', CATHEDRAL_CHANNEL_LIMITS='16,3',
                       CATHEDRAL_CPU_LIMITS='0,0', CATHEDRAL_INIT_PRIVATE_PAGES='32',
                       CATHEDRAL_INIT_SHARED_PAGES='32', CATHEDRAL_LAUNCH_0_ELF=str(image),
                       CATHEDRAL_LAUNCH_0_ARGUMENT='1', CATHEDRAL_LAUNCH_0_CLOCK='1',
                       CATHEDRAL_LAUNCH_0_PRIVATE_PAGES='20', CATHEDRAL_LAUNCH_0_SHARED_PAGES='4')

    def verify(stream, directory, text):
        if 'Omega chooser catalogue/control/images PASS\n' not in text or 'initial program stopped: Some(Returned(0))' not in text:
            raise RuntimeError(f'chooser control witness failed: {directory / "serial.log"}')

    boot(owner, output / 'boot', True, 90, composition=composition, verifier=verify)
    print(f'Omega chooser catalogue/control/images PASS: {output}', flush=True)


if __name__ == '__main__':
    main()
